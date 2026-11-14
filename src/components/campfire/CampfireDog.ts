import { BufferGeometry, Float32BufferAttribute, Group, Mesh, MeshStandardMaterial, NearestFilter, TextureLoader, Vector3, sRGBEncoding } from 'three'
import { DOG_ABSENT_S, DOG_FIRST_VISIT_S, DOG_STAY_S } from './campfireConfig'
import wolfUrl from '../../assets/campfire/Wolf.png?url'
import wolfParts from '../../assets/campfire/wolfParts.json'
import { smoothRoute, easeTurnRate } from './campfireMotion'

// A blocky dog (wolf model) that visits the campfire: it trots in, turns, lies down next to the local player's seat,
// stays for DOG_STAY_S, gets up, trots away and is gone for DOG_ABSENT_S before it comes back.
//
// Model: "Minecraft - Wolf" by Vincent Yanez (https://sketchfab.com/vinceyanez), CC-BY-4.0, see Wolf.LICENSE.txt.
// The single mesh was split into its boxes (body+mane, head+snout, ears, four legs, tail) in wolfParts.json so the
// parts can move on their own. Units are blocks, the dog faces +Z.

const lerp = (a: number, b: number, t: number) => a + (b - a) * t
const clamp01 = (v: number) => Math.max(0, Math.min(1, v))
const smooth = (v: number) => { const x = clamp01(v); return x * x * (3 - 2 * x) }
function angleDiff(a: number, b: number) {
  let d = (b - a) % (Math.PI * 2)
  if (d > Math.PI) d -= Math.PI * 2
  if (d < -Math.PI) d += Math.PI * 2
  return d
}

// Route in and the lying spot. Verified against the glTF block heights like the seat routes: flat y=35 ground,
// >= 0.4 blocks clear of the logs (row log ends at x=-15, column log starts at x=-14, z=-4).
const ROUTE_IN: [number, number][] = [[-8.5, -6.4], [-14.4, -6.4], [-14.4, -3.8], [-15.3, -3.0], [-15.3, -1.5]]
const LIE_YAW = -Math.PI / 2   // faces the fire

const WALK_SPEED = 1.7
const TURN_SPEED = 2.6

type State = 'absent' | 'arrive' | 'turn' | 'lie' | 'rest' | 'rise' | 'leave'

export class CampfireDog {
  readonly group = new Group()
  state: State = 'absent'

  private readonly groundY: (x: number, z: number) => number
  private readonly disposables: { dispose(): void }[] = []
  private timer = DOG_FIRST_VISIT_S
  private restTimer = 0
  private path: Vector3[] = []
  private wp = 1
  private reverse = false
  private yaw = 0
  private yawRate = 0
  private pos = new Vector3()
  private phase = 0
  private walkAmp = 0
  private lie = 0          // 0 standing .. 1 lying
  private clock = 0
  private sleeping = false
  private nextHeadLift = 8 + Math.random() * 10
  private headLiftT = -1
  private nextTwitch = 3
  private twitchT = -1
  private twitchSide = 1
  private nextWag = 2
  private wagT = -1
  private nextSleep = 60 + Math.random() * 40
  private gy: number | null = null

  private readonly hip = new Group()
  private readonly body: Mesh
  private readonly head: Mesh
  private readonly ears: Mesh[] = []
  private readonly tail: Mesh
  private readonly legs: Mesh[] = []   // FL, FR, HL, HR
  // rest positions of the moving parts (the model's own pivots)
  private readonly headY = wolfParts.parts.head.pivot[1]
  private readonly headZ = wolfParts.parts.head.pivot[2]
  private readonly legY = wolfParts.parts.legFL.pivot[1]

  constructor(groundY: (x: number, z: number) => number) {
    this.groundY = groundY
    this.group.visible = false

    const tex = new TextureLoader().load(wolfUrl)
    tex.magFilter = NearestFilter
    tex.minFilter = NearestFilter
    tex.flipY = false   // the UVs come straight from the glTF
    tex.encoding = sRGBEncoding
    const material = new MeshStandardMaterial({ map: tex, roughness: 1, metalness: 0 })
    this.disposables.push(tex, material)

    /** One model part as a mesh at its pivot; the geometry is stored relative to the pivot. */
    const part = (name: string): Mesh => {
      const d = (wolfParts.parts as Record<string, { pos: number[]; nrm: number[]; uv: number[]; idx: number[]; pivot: number[] }>)[name]
      const g = new BufferGeometry()
      g.setAttribute('position', new Float32BufferAttribute(d.pos, 3))
      g.setAttribute('normal', new Float32BufferAttribute(d.nrm, 3))
      g.setAttribute('uv', new Float32BufferAttribute(d.uv, 2))
      g.setIndex(d.idx)
      this.disposables.push(g)
      const m = new Mesh(g, material)
      m.castShadow = true
      m.position.set(d.pivot[0], d.pivot[1], d.pivot[2])
      return m
    }

    this.body = part('body')
    this.hip.add(this.body)

    // legs: pivot at the hip joint (front left/right, hind left/right = the order the pose code expects)
    for (const n of ['legFR', 'legFL', 'legHR', 'legHL']) {
      const leg = part(n)
      this.hip.add(leg)
      this.legs.push(leg)
    }

    // head with snout, pivot at the neck; ears hang on the head and pivot at their base
    const head = part('head')
    this.head = head
    for (const n of ['earL', 'earR']) {
      const ear = part(n)
      ear.position.sub(head.position)
      head.add(ear)
      this.ears.push(ear)
    }
    this.hip.add(head)

    // tail: pivot at the base, modelled hanging down and back
    this.tail = part('tail')
    this.hip.add(this.tail)

    this.group.add(this.hip)
    this.apply(0)
  }

  /**
   * `playerAtFire`: the local player is at the fire (sitting, or up for a visit), so there is somebody to keep
   * company. `canStart`: false while a visit is going on, so the dog does not walk its route across the visitor's.
   */
  update(dt: number, playerAtFire: boolean, canStart = true): void {
    this.clock += dt
    switch (this.state) {
      case 'absent':
        if (this.timer > 0) this.timer -= dt
        else if (playerAtFire && canStart) this.begin(false)
        break
      case 'arrive':
        if (this.follow(dt)) { this.state = 'turn' }
        break
      case 'turn': {
        const d = angleDiff(this.yaw, LIE_YAW)
        this.yawRate = easeTurnRate(this.yawRate, d, dt, 2.2, TURN_SPEED, 3.5)
        this.yaw += this.yawRate * dt
        this.phase += Math.abs(this.yawRate) * dt * 2.5
        this.walkAmp = lerp(this.walkAmp, Math.min(1, Math.abs(d)), Math.min(1, dt * 8))
        if (Math.abs(d) < 0.03) { this.state = 'lie'; this.walkAmp = 0 }
        break
      }
      case 'lie':
        this.lie = Math.min(1, this.lie + dt / 2.2)
        if (this.lie >= 1) { this.state = 'rest'; this.restTimer = DOG_STAY_S; this.sleeping = false; this.nextSleep = 60 + Math.random() * 40 }
        break
      case 'rest':
        this.restTimer -= dt
        this.idle(dt)
        if (this.restTimer <= 0 || !playerAtFire) { this.state = 'rise'; this.sleeping = false }
        break
      case 'rise':
        this.lie = Math.max(0, this.lie - dt / 1.6)
        if (this.lie <= 0) this.begin(true)
        break
      case 'leave':
        if (this.follow(dt)) {
          this.state = 'absent'
          this.timer = DOG_ABSENT_S
          this.group.visible = false
          this.walkAmp = 0
        }
        break
    }
    this.apply(dt)
  }

  /** Test action: come now (as soon as the player is at the fire and nothing else is going on). */
  callNow(): boolean {
    if (this.state !== 'absent') return false
    this.timer = 0
    return true
  }

  /** Test action: get up and leave. */
  sendAway(): boolean {
    if (this.state !== 'rest' && this.state !== 'lie') return false
    this.state = 'rise'
    this.sleeping = false
    return true
  }

  dispose(): void {
    for (const d of this.disposables) d.dispose()
  }

  // ── movement ─────────────────────────────────────────────────────────────────
  private begin(leaving: boolean): void {
    const pts = smoothRoute(ROUTE_IN.map(([x, z]) => new Vector3(x, 0, z)), 3)
    this.reverse = leaving
    this.path = leaving ? pts.slice().reverse() : pts
    this.wp = 1
    if (!leaving) this.pos.copy(this.path[0])
    else this.pos.copy(pts[pts.length - 1])
    const next = this.path[1]
    this.yaw = leaving ? this.yaw : Math.atan2(next.x - this.pos.x, next.z - this.pos.z)
    this.state = leaving ? 'leave' : 'arrive'
    this.group.visible = true
  }

  private follow(dt: number): boolean {
    const target = this.path[this.wp]
    if (!target) return true
    const dx = target.x - this.pos.x
    const dz = target.z - this.pos.z
    const dist = Math.hypot(dx, dz)
    const diff = angleDiff(this.yaw, Math.atan2(dx, dz))
    this.yawRate = easeTurnRate(this.yawRate, dist > 0.05 ? diff : 0, dt, 3.0, TURN_SPEED, 4.5)
    if (dist > 0.05) this.yaw += this.yawRate * dt
    const step = Math.min(WALK_SPEED * Math.max(0.3, 1 - Math.abs(diff) / 2) * dt, dist)
    if (dist > 1e-4) { this.pos.x += (dx / dist) * step; this.pos.z += (dz / dist) * step }
    this.phase += step * 9
    this.walkAmp = lerp(this.walkAmp, 1, Math.min(1, dt * 8))
    const last = this.wp >= this.path.length - 1
    if (last ? dist - step < 0.02 : dist < 0.25) {
      this.wp++
      if (this.wp >= this.path.length) return true
    }
    return false
  }

  // ── resting behaviour: breathing, ear twitches, tail wags, looking around, falling asleep ─────
  private idle(dt: number): void {
    const t = this.clock
    this.nextTwitch -= dt
    if (this.nextTwitch <= 0 && this.twitchT < 0) { this.twitchT = 0; this.twitchSide = Math.random() < 0.5 ? 0 : 1 }
    if (this.twitchT >= 0) { this.twitchT += dt; if (this.twitchT > 0.35) { this.twitchT = -1; this.nextTwitch = 2 + Math.random() * 6 } }

    this.nextWag -= dt
    if (this.nextWag <= 0 && this.wagT < 0 && !this.sleeping) { this.wagT = 0 }
    if (this.wagT >= 0) { this.wagT += dt; if (this.wagT > 1.6) { this.wagT = -1; this.nextWag = 4 + Math.random() * 12 } }

    this.nextSleep -= dt
    if (this.nextSleep <= 0) { this.sleeping = !this.sleeping; this.nextSleep = this.sleeping ? 40 + Math.random() * 40 : 50 + Math.random() * 50 }

    if (!this.sleeping) {
      this.nextHeadLift -= dt
      if (this.nextHeadLift <= 0 && this.headLiftT < 0) this.headLiftT = 0
    }
    if (this.headLiftT >= 0) { this.headLiftT += dt; if (this.headLiftT > 3.2) { this.headLiftT = -1; this.nextHeadLift = 10 + Math.random() * 18 } }
    void t
  }

  // ── pose ─────────────────────────────────────────────────────────────────────
  private apply(_dt: number): void {
    const c = this.clock
    const b = smooth(this.lie)
    const [fl, fr, hl, hr] = this.legs

    // the ground raycast is costly: only while the dog actually moves
    if (this.state === 'arrive' || this.state === 'turn' || this.state === 'leave' || this.gy === null) this.gy = this.groundY(this.pos.x, this.pos.z)
    this.group.position.set(this.pos.x, this.gy, this.pos.z)
    this.group.rotation.y = this.yaw

    // body drops to the ground while lying
    const breath = this.state === 'rest' || b > 0.9 ? Math.sin(c * (this.sleeping ? 1.6 : 2.4)) : 0
    this.hip.position.y = -0.32 * b + Math.abs(Math.cos(this.phase)) * 0.03 * this.walkAmp * (1 - b)
    this.body.scale.y = 1 + breath * 0.018

    // legs: trot (diagonal pairs) -> front legs stretched out, hind legs tucked beside the body
    const sw = Math.sin(this.phase) * 0.75 * this.walkAmp * (1 - b)
    fl.rotation.x = lerp(sw, -1.5, b)
    fr.rotation.x = lerp(-sw, -1.5, b)
    hl.rotation.x = lerp(-sw, -1.15, b)
    hr.rotation.x = lerp(sw, -1.15, b)
    hl.rotation.z = lerp(0, 0.45, b)
    hr.rotation.z = lerp(0, -0.45, b)
    for (const leg of this.legs) leg.position.y = lerp(this.legY, this.legY + 0.04, b)

    // head: bobs while trotting, rests on the paws while lying, lifts and looks around now and then
    const lift = this.headLiftT >= 0 ? Math.sin(Math.PI * clamp01(this.headLiftT / 3.2)) : 0
    const sleepDrop = this.sleeping ? 1 : 0
    this.head.position.y = lerp(this.headY, this.headY - 0.23, b) + lift * 0.12 - sleepDrop * 0.025 * b
    this.head.position.z = lerp(this.headZ, this.headZ + 0.07, b)
    this.head.rotation.x = lerp(Math.sin(this.phase * 2) * 0.05 * this.walkAmp, 0.14 + sleepDrop * 0.12, b) - lift * 0.40
    this.head.rotation.y = lift * Math.sin(this.headLiftT * 1.6) * 0.7

    // ears: relaxed, with an occasional flick
    const twitch = this.twitchT >= 0 ? Math.sin(Math.PI * clamp01(this.twitchT / 0.35)) : 0
    this.ears.forEach((ear, i) => {
      const side = i === 0 ? -1 : 1
      ear.rotation.z = -side * (0.10 + b * 0.18) - (i === this.twitchSide ? side * twitch * 0.5 : 0)
      ear.rotation.x = -0.1 * (1 - b) + b * 0.12
    })

    // tail: up and wagging while walking, low and calm lying, with a wag now and then
    const wag = this.wagT >= 0 ? Math.sin(Math.PI * clamp01(this.wagT / 1.6)) : 0
    const wagAmp = Math.max(this.walkAmp * 0.5, wag * 0.9)
    this.tail.rotation.x = lerp(0.8, 0.1, b)
    this.tail.rotation.y = Math.sin(c * 14) * 0.5 * wagAmp
  }
}
