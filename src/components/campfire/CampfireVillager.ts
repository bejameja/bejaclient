import {
  BufferAttribute, BufferGeometry, FrontSide, Group, Mesh, MeshStandardMaterial, NearestFilter, Vector3,
} from 'three'
import { GLTFLoader, type GLTF } from 'three/examples/jsm/loaders/GLTFLoader'
import villagerUrl from '../../assets/campfire/Villager.gltf?url'
import { TRADES, type IconName } from './campfireIcons'
import { easeTurnRate } from './campfireMotion'
import {
  VILLAGER_ENABLED, VILLAGER_EXCHANGES_MAX, VILLAGER_EXCHANGES_MIN, VILLAGER_FIRST_MAX_S, VILLAGER_FIRST_MIN_S,
  VILLAGER_WAIT_MAX_S, VILLAGER_WAIT_MIN_S,
} from './campfireConfig'

// A villager who now and then walks up to the fire and "trades" with the local player: they talk (hrmm), the
// villager shows trade offers in bubbles, the player holds out a hand and nods or shakes the head. Then the villager
// says goodbye and walks away.
//
// Model: "Minecraft - Villager" by Vincent Yanez (https://sketchfab.com/vinceyanez), CC-BY-4.0
// (https://sketchfab.com/3d-models/minecraft-villager-905c2547478f4bc9b4a8a1521966ab81), see
// src/assets/campfire/Villager.LICENSE.txt. The file is one static mesh made of 9 cubes (24 vertices each); they are
// split into parts here so the head can nod and shake, the legs can walk and the robe can swish.
// Units are blocks, the villager faces +Z.

const lerp = (a: number, b: number, t: number) => a + (b - a) * t
const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v))
const rand = (min: number, max: number) => min + Math.random() * (max - min)
function angleDiff(a: number, b: number) {
  let d = (b - a) % (Math.PI * 2)
  if (d > Math.PI) d -= Math.PI * 2
  if (d < -Math.PI) d += Math.PI * 2
  return d
}

// Route in and the talking spot, in front of the local player's seat (fire side of the column log). Checked against
// the glTF block heights like the other routes: flat y=35 ground, >= 0.45 from the logs, >= 1.2 from the lying dog.
const ROUTE_IN: [number, number][] = [[-8.5, -6.4], [-14.4, -6.4], [-14.4, -3.8], [-15.6, -3.1]]

const WALK_SPEED = 1.15
const ACCEL = 1.5        // blocks/s^2, speeding up from a standstill
const BRAKE = 2.4        // slowing down for the end of the route
const EXCHANGE_S = 8
const FAREWELL_S = 3.2
const HEIGHT = 1.95      // blocks; the model is 34 model units tall

/** Chaikin corner cutting: turns the sharp-cornered route into a gentle curve (checked: it stays >= 0.5 from the logs). */
function smoothRoute(pts: Vector3[], iterations = 3): Vector3[] {
  let p = pts
  for (let i = 0; i < iterations; i++) {
    const q = [p[0]]
    for (let j = 0; j < p.length - 1; j++) {
      q.push(p[j].clone().multiplyScalar(0.75).addScaledVector(p[j + 1], 0.25))
      q.push(p[j].clone().multiplyScalar(0.25).addScaledVector(p[j + 1], 0.75))
    }
    q.push(p[p.length - 1])
    p = q
  }
  return p
}

type State = 'absent' | 'arrive' | 'face' | 'talk' | 'leave'

export type PlayerCue = 'listen' | 'offer' | 'nod' | 'shake' | 'wave'
export interface Bubble { key: string; text?: string; give?: IconName; get?: IconName }

interface Exchange { give: IconName; get: IconName; accept: boolean }

// The glTF is tiny but parsed only once.
let modelPromise: Promise<GLTF> | null = null
const loadModel = () => (modelPromise ??= new GLTFLoader().loadAsync(villagerUrl))

/** Copies cube `k` (24 vertices, 36 indices) of `src` into its own geometry. */
function cube(src: BufferGeometry, k: number): BufferGeometry {
  const g = new BufferGeometry()
  for (const name of ['position', 'normal', 'uv'] as const) {
    const a = src.getAttribute(name)
    const n = a.itemSize
    const out = new Float32Array(24 * n)
    for (let i = 0; i < 24; i++) {
      out[i * n] = a.getX(k * 24 + i)
      out[i * n + 1] = a.getY(k * 24 + i)
      if (n > 2) out[i * n + 2] = a.getZ(k * 24 + i)
    }
    g.setAttribute(name, new BufferAttribute(out, n))
  }
  const idx = src.getIndex()!
  const out = new Uint16Array(36)
  for (let i = 0; i < 36; i++) out[i] = idx.getX(k * 36 + i) - k * 24
  g.setIndex(new BufferAttribute(out, 1))
  return g
}

export class CampfireVillager {
  readonly group = new Group()
  state: State = 'absent'
  /** True once the villager wants to come (timer or manual trigger); the scene starts it when the moment is calm. */
  wantsToStart = false
  /** What the villager says / offers right now, for the speech bubble. */
  bubble: Bubble | null = null
  /** What the local player should do right now (listen, hold out a hand, nod, shake the head, wave). */
  playerCue: PlayerCue | null = null

  private readonly groundY: (x: number, z: number) => number
  private readonly disposables: { dispose(): void }[] = []
  private timer = rand(VILLAGER_FIRST_MIN_S, VILLAGER_FIRST_MAX_S)
  private path: Vector3[] = []
  private wp = 1
  private pos = new Vector3()
  private yaw = 0
  private gy = 35
  private phase = 0
  private walkAmp = 0
  private speed = 0          // current walking speed in blocks/s
  private yawRate = 0        // smoothed turning speed, for leaning into turns and leading with the head
  private route: Vector3[] = []
  private readonly seed = Math.random() * 100
  private clock = 0
  private talkT = 0
  private plan: Exchange[] = []
  private faceYaw = 0
  private disposed = false

  // model parts (set once the glTF is loaded)
  private ready = false
  private readonly model = new Group()      // scaled: 1 model unit = HEIGHT / 34 blocks
  private head: Group | null = null
  private robe: Group | null = null
  private body: Mesh | null = null
  private legs: Group[] = []

  constructor(groundY: (x: number, z: number) => number) {
    this.groundY = groundY
    this.group.visible = false
    this.group.add(this.model)
    loadModel().then(gltf => { if (!this.disposed) this.attach(gltf) }).catch(e => console.warn('[Campfire] villager model failed to load', e))
  }

  private attach(gltf: GLTF): void {
    let source: Mesh | null = null
    gltf.scene.updateMatrixWorld(true)
    gltf.scene.traverse(o => { if ((o as Mesh).isMesh) source = o as Mesh })
    if (!source) return
    const src = source as Mesh

    // bake the Sketchfab export's node transforms (Z-up FBX -> Y-up), feet on y = 0
    const geo = src.geometry.clone()
    geo.applyMatrix4(src.matrixWorld)
    geo.computeBoundingBox()
    const bb = geo.boundingBox!
    geo.translate(0, -bb.min.y, 0)
    this.model.scale.setScalar(HEIGHT / (bb.max.y - bb.min.y))

    const material = (src.material as MeshStandardMaterial).clone()
    material.alphaTest = 0.5
    material.transparent = false
    material.side = FrontSide
    material.roughness = 1
    material.metalness = 0
    if (material.map) { material.map.magFilter = NearestFilter; material.map.minFilter = NearestFilter }
    this.disposables.push(material)

    // cube indices in the file: 0 head, 1 robe, 2 + 4 arms, 3 crossed-arms bar, 5 + 6 legs, 7 body, 8 nose
    // (pivot points in model units after the feet-to-zero shift)
    const part = (k: number, pivot: [number, number, number]): Mesh => {
      const g = cube(geo, k)
      g.translate(-pivot[0], -pivot[1], -pivot[2])
      this.disposables.push(g)
      const m = new Mesh(g, material)
      m.castShadow = true
      m.position.set(...pivot)
      return m
    }
    const centre = (k: number): [number, number, number] => {
      const g = cube(geo, k)
      g.computeBoundingBox()
      const c = g.boundingBox!.getCenter(new Vector3())
      g.dispose()
      return [c.x, c.y, c.z]
    }

    this.head = new Group(); this.head.position.set(0, 24, 0)       // neck
    for (const k of [0, 8]) {
      const m = part(k, [0, 24, 0]); m.position.set(0, 0, 0)
      this.head.add(m)
    }
    this.robe = new Group(); this.robe.position.set(0, 12, 0)       // hips
    const robe = part(1, [0, 12, 0]); robe.position.set(0, 0, 0)
    this.robe.add(robe)
    this.body = part(7, centre(7))
    for (const [k, x] of [[5, 2], [6, -2]] as [number, number][]) {
      const leg = new Group(); leg.position.set(x, 12, 0)
      const m = part(k, [x, 12, 0]); m.position.set(0, 0, 0)
      leg.add(m)
      this.legs.push(leg)
    }
    this.model.add(this.head, this.robe, this.body, ...this.legs)
    for (const k of [2, 3, 4]) this.model.add(part(k, [0, 0, 0]))   // crossed arms, static
    this.ready = true
    this.apply()
  }

  /** Test button / debug: come as soon as the scene allows it. */
  requestNow(): void { this.wantsToStart = true }

  get active(): boolean { return this.state !== 'absent' }

  update(dt: number, playerSeated: boolean, playerPos: Vector3 | null): void {
    this.clock += dt
    switch (this.state) {
      case 'absent':
        if (VILLAGER_ENABLED && !this.wantsToStart) {
          this.timer -= dt
          if (this.timer <= 0) this.wantsToStart = true
        }
        break
      case 'arrive':
        if (!playerSeated) { this.startLeaving(); break }
        if (this.follow(dt)) { this.state = 'face' }
        break
      case 'face': {
        if (!playerSeated || !playerPos) { this.startLeaving(); break }
        this.faceYaw = Math.atan2(playerPos.x - this.pos.x, playerPos.z - this.pos.z)
        const d = angleDiff(this.yaw, this.faceYaw)
        const rate = clamp(d * 2.2, -1.5, 1.5)
        this.yawRate = easeTurnRate(this.yawRate, d, dt, 2.2, 1.5, 3.2)
        this.yaw += this.yawRate * dt
        this.speed = lerp(this.speed, 0, Math.min(1, dt * 6))
        this.phase += Math.abs(this.yawRate) * dt * 1.6
        this.walkAmp = lerp(this.walkAmp, Math.min(1, Math.abs(rate) / 2), Math.min(1, dt * 6))
        if (Math.abs(d) < 0.05) { this.state = 'talk'; this.talkT = 0 }
        break
      }
      case 'talk':
        if (!playerSeated) { this.startLeaving(); break }
        if (playerPos) this.yaw += clamp(angleDiff(this.yaw, Math.atan2(playerPos.x - this.pos.x, playerPos.z - this.pos.z)), -0.5, 0.5) * dt * 2
        this.walkAmp = lerp(this.walkAmp, 0, Math.min(1, dt * 5))
        this.yawRate = lerp(this.yawRate, 0, Math.min(1, dt * 5))
        this.speed = 0
        this.talk(dt)
        break
      case 'leave':
        if (this.follow(dt)) {
          this.state = 'absent'
          this.group.visible = false
          this.walkAmp = 0
          this.speed = 0
          this.yawRate = 0
          this.timer = rand(VILLAGER_WAIT_MIN_S, VILLAGER_WAIT_MAX_S)
          this.wantsToStart = false
        }
        break
    }
    this.apply()
  }

  dispose(): void {
    this.disposed = true
    for (const d of this.disposables) d.dispose()
  }

  // ── event flow ───────────────────────────────────────────────────────────────
  /** The scene calls this when it is calm enough to start. */
  begin(): void {
    this.wantsToStart = false
    this.route = smoothRoute(ROUTE_IN.map(([x, z]) => new Vector3(x, 0, z)))
    this.path = this.route
    this.wp = 1
    this.speed = 0
    this.yawRate = 0
    this.pos.copy(this.path[0])
    this.yaw = Math.atan2(this.path[1].x - this.pos.x, this.path[1].z - this.pos.z)
    this.state = 'arrive'
    this.group.visible = true
    // a few trade offers, mostly accepted
    const n = Math.round(rand(VILLAGER_EXCHANGES_MIN, VILLAGER_EXCHANGES_MAX))
    const pool = TRADES.slice().sort(() => Math.random() - 0.5)
    this.plan = Array.from({ length: n }, (_, i) => ({ ...pool[i % pool.length], accept: Math.random() < 0.65 }))
    this.bubble = null
    this.playerCue = null
  }

  private startLeaving(): void {
    this.bubble = null
    this.playerCue = null
    // back the way we came: the points passed so far while still arriving, the whole route from the talking spot
    const route = this.route
    const back = this.state === 'arrive' ? route.slice(0, Math.max(1, this.wp)).reverse() : route.slice(0, -1).reverse()
    this.path = smoothRoute([this.pos.clone(), ...back], 1)
    this.wp = 1
    this.state = 'leave'
  }

  private talk(dt: number): void {
    this.talkT += dt
    const e = Math.floor(this.talkT / EXCHANGE_S)
    if (e >= this.plan.length) {
      const t = this.talkT - this.plan.length * EXCHANGE_S
      if (t >= FAREWELL_S) { this.startLeaving(); this.bubble = null; return }
      this.setBubble('bye', { text: 'Hrmm.' })
      this.playerCue = t > 0.6 ? 'wave' : 'listen'
      return
    }
    const ex = this.plan[e]
    const t = this.talkT - e * EXCHANGE_S
    if (t < 1.6) {
      this.setBubble(`q${e}`, { text: 'Hrmm?' })
      this.playerCue = 'listen'
    } else if (t < 5.2) {
      this.setBubble(`o${e}`, { give: ex.give, get: ex.get })
      this.playerCue = t < 3.2 ? 'offer' : ex.accept ? 'nod' : 'shake'
    } else {
      this.setBubble(`a${e}`, { text: ex.accept ? 'Hrmm!' : 'Hrmm.' })
      this.playerCue = 'listen'
    }
  }

  private setBubble(key: string, b: Omit<Bubble, 'key'>): void {
    if (this.bubble?.key === key) return
    this.bubble = { key, ...b }
  }

  // ── movement ─────────────────────────────────────────────────────────────────
  private follow(dt: number): boolean {
    const target = this.path[this.wp]
    if (!target) return true
    const dx = target.x - this.pos.x
    const dz = target.z - this.pos.z
    const dist = Math.hypot(dx, dz)
    const diff = angleDiff(this.yaw, Math.atan2(dx, dz))
    // turn smoothly (the rate eases in and out instead of jumping)
    this.yawRate = easeTurnRate(this.yawRate, dist > 0.05 ? diff : 0, dt, 3.0, 2.2, 4.5)
    this.yaw += this.yawRate * dt

    // distance left on the route: slow down for the end, speed up gently from a standstill
    let left = dist
    for (let k = this.wp; k < this.path.length - 1; k++) left += this.path[k].distanceTo(this.path[k + 1])
    const straight = clamp(1 - Math.abs(diff) / 1.3, 0, 1)             // nearly stops for sharp turns, pivots first
    const want = WALK_SPEED * straight * clamp(left / 1.3, 0.12, 1)
    this.speed += clamp(want - this.speed, -BRAKE * dt, ACCEL * dt)
    const step = Math.min(this.speed * dt, dist)
    if (dist > 1e-4) { this.pos.x += (dx / dist) * step; this.pos.z += (dz / dist) * step }
    // legs: by distance walked; a little shuffling on the spot while pivoting
    this.phase += step * 6.5 + Math.abs(this.yawRate) * dt * (this.speed < 0.3 ? 1.4 : 0)
    this.walkAmp = lerp(this.walkAmp, clamp(this.speed / WALK_SPEED + Math.abs(this.yawRate) * 0.1, 0, 1), Math.min(1, dt * 6))
    this.gy = lerp(this.gy, this.groundY(this.pos.x, this.pos.z), Math.min(1, dt * 14))
    const last = this.wp >= this.path.length - 1
    if (last ? left < 0.03 : dist < 0.3) {
      this.wp++
      if (this.wp >= this.path.length) return true
    }
    return false
  }

  // ── pose ─────────────────────────────────────────────────────────────────────
  private apply(): void {
    const c = this.clock
    this.group.position.set(this.pos.x, this.gy, this.pos.z)
    this.group.rotation.y = this.yaw
    if (!this.ready || !this.head || !this.robe || !this.body) return

    // walking: legs swing under the robe; the body sways, leans into the walk and into turns, bobs on every step
    const w = this.walkAmp
    const s1 = Math.sin(this.phase)
    const leg = 0.62 * w
    this.legs[0].rotation.x = s1 * leg
    this.legs[1].rotation.x = -s1 * leg
    const lean = clamp(this.speed / WALK_SPEED, 0, 1)
    this.robe.rotation.x = 0.045 * lean + Math.sin(this.phase * 2 + 0.6) * 0.012 * w
    this.robe.rotation.z = -s1 * 0.035 * w - this.yawRate * 0.035 * lean + Math.sin(c * 0.5 + this.seed) * 0.012 * (1 - w)  // sway; weight shift when standing
    this.robe.rotation.y = s1 * 0.05 * w
    this.group.position.y += (Math.abs(Math.cos(this.phase)) * 0.03 - 0.012) * w
    this.body.scale.y = 1 + Math.sin(c * 1.9) * 0.012
    this.body.rotation.copy(this.robe.rotation)

    // head: nods and shakes while talking, small idle movement otherwise
    const w2 = this.walkAmp
    const glance = Math.sin(c * 0.53 + this.seed) * Math.sin(c * 0.29 + this.seed * 2)   // slow, irregular looking around
    let hx = Math.sin(c * 0.9) * 0.02 + Math.sin(this.phase * 2) * 0.018 * w2 - 0.03 * w2
    let hy = Math.sin(c * 0.45) * 0.06 + glance * 0.22 * w2 - Math.sin(this.phase) * 0.04 * w2 + this.yawRate * 0.12 * w2
    let hz = Math.sin(this.phase) * 0.02 * w2
    if (this.state === 'talk') {
      const e = Math.floor(this.talkT / EXCHANGE_S)
      const ex = this.plan[e]
      const t = this.talkT - e * EXCHANGE_S
      if (!ex) {                                        // goodbye: one slow nod
        hx += Math.sin(clamp((this.talkT - this.plan.length * EXCHANGE_S) / FAREWELL_S, 0, 1) * Math.PI * 2) * 0.22
      } else if (t < 1.6) {
        hz = Math.sin(clamp(t / 1.6, 0, 1) * Math.PI) * 0.22                 // curious tilt
      } else if (t < 5.2) {
        hx += Math.sin((t - 1.6) * 2.4) * 0.12                                // considers the offer
      } else if (ex.accept) {
        hx += Math.sin((t - 5.2) * 9) * 0.22 * Math.sin(clamp((t - 5.2) / 2.8, 0, 1) * Math.PI)   // happy nodding
      } else {
        hy += Math.sin((t - 5.2) * 9) * 0.4 * Math.sin(clamp((t - 5.2) / 2.8, 0, 1) * Math.PI)    // no
      }
    }
    // the head follows the body's sway a little, so it does not look glued on
    this.head.rotation.set(hx, hy, hz + this.robe.rotation.z * 0.6)
    this.head.position.x = this.robe.rotation.z * -2
  }
}
