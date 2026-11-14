import { CanvasTexture, Group, Mesh, NearestFilter, sRGBEncoding, Vector3 } from 'three'
import { PlayerObject } from 'skinview3d'
import { inferModelType, loadCapeToCanvas, loadImage, loadSkinToCanvas } from 'skinview-utils'
import {
  FIRE, SEAT_TOP, VISITS_ENABLED, VISIT_RETRY_S, VISIT_STAND_MAX_S, VISIT_STAND_MIN_S, VISIT_WAIT_MAX_S, VISIT_WAIT_MIN_S, type Seat,
} from './campfireConfig'
import { envelope, pickGesture, pickStandGesture, zeroPose, BASE_ARM_X, BASE_ELBOW, BASE_KNEE, BASE_LEG_X, type Gesture, type GestureContext, type Pose } from './campfireGestures'
import type { PlayerCue } from './CampfireVillager'
import { smoothRoute, easeTurnRate } from './campfireMotion'
import { installLimbBendRig, type LimbBendJoints } from '../skin/limbBendRig'

const easeInOut = (t: number) => t * t * (3 - 2 * t)
const lerp = (a: number, b: number, t: number) => a + (b - a) * t
function angleDiff(a: number, b: number) {
  let d = (b - a) % (Math.PI * 2)
  if (d > Math.PI) d -= Math.PI * 2
  if (d < -Math.PI) d += Math.PI * 2
  return d
}

const WALK_SPEED = 1.35   // blocks / s
const TURN_SPEED = 2.4    // rad / s (top turning speed; it eases in and out)
const CLIMB_DURATION = 2.6  // s: climb onto the log, swing around, let the legs dangle
const DESCEND_DURATION = 2.0
const HIP_HEIGHT = 0.75   // standing hip pivot above the feet (12px)
const SEAT_CLEARANCE = 0.08  // legs are 0.25 thick and centred on the hip line - lift a touch so they rest on the log, not in it
const SEAT_FRONT = 0.33      // sit towards the fire-side edge of the log so the shins hang past it instead of through it

type State = 'walk' | 'sitdown' | 'idle' | 'standup' | 'leave' | 'gone' | 'visitOut' | 'visitStand' | 'visitBack'

const rand = (min: number, max: number) => min + Math.random() * (max - min)
const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v))

/** Lets the scene allow one visit at a time. */
export interface VisitGate {
  request(actor: CampfireActor): boolean
  release(actor: CampfireActor): void
}

export interface ActorSkin {
  skinUrl: string | null
  capeUrl: string | null
  model: 'default' | 'slim' | 'auto-detect'
}

/**
 * One character in the campfire scene: a skinview3d PlayerObject (so skins,
 * slim arms and capes behave exactly like everywhere else in the launcher)
 * driven by a small walk → turn → sit → idle → stand → leave state machine.
 */
export class CampfireActor {
  readonly uuid: string
  readonly seatIndex: number
  readonly rig = new Group()        // feet on the ground, 1 unit = 1 block, faces +Z
  readonly player: PlayerObject
  state: State = 'walk'
  /** Positions of the other seated characters, used for "look at a neighbour" gestures. */
  peers: () => Vector3[] = () => []
  /** Set by the scene; without it the character never leaves its seat for a visit. */
  visitGate: VisitGate | null = null
  /** What this character does while the villager talks to it (set by the scene; null = nothing). */
  private chatCue: PlayerCue | null = null
  private lastChatCue: PlayerCue | null = null
  private chatFade = 0
  private readonly chatTarget = new Vector3()

  private seat: Seat
  private readonly groundY: (x: number, z: number) => number
  private skinCanvas = document.createElement('canvas')
  private capeCanvas = document.createElement('canvas')
  private skinTexture = new CanvasTexture(this.skinCanvas)
  private capeTexture = new CanvasTexture(this.capeCanvas)
  private capeLoaded = false
  private skinToken = 0
  private bend: LimbBendJoints | null = null   // knee/elbow joints (see skin/limbBendRig.ts)
  private skinReady = false   // stay hidden until a skin arrived, an empty texture renders solid black

  private path: Vector3[]
  private leavePath: Vector3[] = []
  private wp = 1
  private pos: Vector3
  private yaw: number
  private seatYaw: number
  private sit = 0
  private climbStartOff = 0   // distance of the stand spot from the log centre, along the seat's facing
  private phase = 0
  private stateTime = 0
  private clock = Math.random() * 10

  // Idle gestures (see campfireGestures.ts)
  private readonly upper = new Group()   // hips-up: lean the whole torso, head and arms follow
  private gesture: Gesture | null = null
  private lastGesture: Gesture | null = null
  private gestureT = 0
  private gestureDur = 0
  private gestureCtx: GestureContext = { yawToFire: 0, yawToPeer: null }
  private gestureFade = 0
  private nextGestureIn = 1 + Math.random() * 1.5
  private pose = zeroPose()
  private gy: number

  // Visits to the fire (see VISITS_ENABLED): idle -> standup -> visitOut -> visitStand -> visitBack -> sitdown -> idle
  private visiting = false          // between getting up and starting the climb back; the seat stays reserved
  private holdsGate = false
  private visitIn = rand(VISIT_WAIT_MIN_S, VISIT_WAIT_MAX_S)
  private visitPath: Vector3[] = []      // stand -> lane -> via... -> spot
  private visitBackPath: Vector3[] = []  // spot -> via (reversed) -> lane -> stand
  private visitTimeLeft = 0
  private visitYaw = 0                   // faces the fire
  private yawRate = 0
  private stepAmp = 0                    // stepping while turning on the spot

  constructor(uuid: string, seatIndex: number, seat: Seat, groundY: (x: number, z: number) => number) {
    this.uuid = uuid
    this.seatIndex = seatIndex
    this.seat = seat
    this.groundY = groundY
    this.path = smoothRoute(seat.path.map(p => p.clone()))
    this.leavePath = smoothRoute([...seat.path].reverse().map(p => p.clone()))
    this.pos = this.path[0].clone()
    this.yaw = Math.atan2(this.path[1].x - this.path[0].x, this.path[1].z - this.path[0].z)
    this.seatYaw = Math.atan2(seat.facing.x, seat.facing.z)
    this.gy = groundY(this.pos.x, this.pos.z)

    for (const t of [this.skinTexture, this.capeTexture]) {
      t.magFilter = NearestFilter
      t.minFilter = NearestFilter
      t.encoding = sRGBEncoding
    }
    this.player = new PlayerObject(this.skinTexture, this.capeTexture)
    this.player.backEquipment = null
    this.player.traverse(o => { if ((o as Mesh).isMesh) (o as Mesh).castShadow = true })

    // Torso, head and arms as one group pivoting at the hips (skinview3d has them as siblings).
    const sk = this.player.skin
    this.upper.position.y = -12
    sk.add(this.upper)
    this.upper.attach(sk.body)
    this.upper.attach(sk.head)
    this.upper.attach(sk.rightArm)
    this.upper.attach(sk.leftArm)

    // skinview3d draws in pixels with the figure centred on y=0 (feet at -16).
    this.player.position.y = 16
    this.rig.scale.setScalar(1 / 16)
    this.rig.add(this.player)
    this.apply()
  }

  get leaving(): boolean {
    return this.state === 'gone' || (!this.visiting && (this.state === 'standup' || this.state === 'leave'))
  }

  /** The villager talks to this (seated) character: look at them, hold out a hand, nod, shake the head, wave. */
  setChat(cue: PlayerCue | null, target: Vector3 | null): void {
    this.chatCue = cue
    if (cue) this.lastChatCue = cue
    if (target) this.chatTarget.copy(target)
  }

  /** Test action: get up for a visit now (it still waits for the scene to be calm). */
  requestVisit(): boolean {
    if (this.state !== 'idle') return false
    this.visitIn = 0
    return true
  }

  /** Up from the seat for a visit to the fire. */
  get onVisit(): boolean { return this.visiting }

  /** Head position in world space, for nametag projection. */
  headWorld(out: Vector3): Vector3 {
    return out.set(this.rig.position.x, this.rig.position.y + 2.35, this.rig.position.z)
  }

  async setSkin(skin: ActorSkin): Promise<void> {
    const token = ++this.skinToken
    if (!skin.skinUrl) { this.markReady(); return }
    try {
      const img = await loadImage(skin.skinUrl)
      if (token !== this.skinToken) return
      loadSkinToCanvas(this.skinCanvas, img)
      const model = skin.model === 'auto-detect' ? inferModelType(this.skinCanvas) : skin.model
      this.skinTexture.needsUpdate = true
      this.player.skin.modelType = model
    } catch (e) {
      console.warn('[Campfire] skin load failed', e)
    }
    this.markReady()
    if (!skin.capeUrl) { this.capeLoaded = false; return }
    try {
      const cape = await loadImage(skin.capeUrl)
      if (token !== this.skinToken) return
      loadCapeToCanvas(this.capeCanvas, cape)
      this.capeTexture.needsUpdate = true
      this.capeLoaded = true
    } catch (e) {
      this.capeLoaded = false
      console.warn('[Campfire] cape load failed', e)
    }
  }

  /** Split the limbs into upper/lower halves once the real slim/classic model type is known. */
  private markReady(): void {
    if (!this.bend) {
      this.bend = installLimbBendRig(this.player.skin, !!(this.player.skin as any).slim)
      this.player.traverse(o => { if ((o as Mesh).isMesh) (o as Mesh).castShadow = true })
    }
    this.skinReady = true
  }

  /** Ask a seated actor to get up and walk out the way they came. */
  leave(): void {
    if (this.state === 'gone' || this.state === 'leave') return
    if (this.state === 'standup') {
      // Already climbing down: if that was for a visit, make it a real exit (the normal standup -> leave takes over).
      if (this.visiting) { this.visiting = false; this.releaseGate() }
      return
    }
    if (this.state === 'visitOut' || this.state === 'visitStand' || this.state === 'visitBack') {
      this.leaveFromVisit()
      return
    }
    if (this.state === 'idle' || this.state === 'sitdown') {
      this.releaseGate()
      this.state = 'standup'
      this.stateTime = (1 - this.sit) * DESCEND_DURATION   // continue from the current sit amount
    } else {
      // Still walking in (or just turning): retrace the waypoints already passed.
      this.leavePath = [this.pos.clone(), ...this.path.slice(0, this.wp).reverse()]
      this.state = 'leave'
      this.wp = 1
    }
  }

  /** The member left the party while this character was away from its seat: walk out from where it stands. */
  private leaveFromVisit(): void {
    const seat = this.seat
    const stand = seat.path[seat.path.length - 1]
    const lane = seat.path[seat.path.length - 2]
    let toLane: Vector3[]
    if (this.state === 'visitOut') toLane = this.visitPath.slice(1, this.wp).reverse()           // passed points, back to the lane
    else if (this.state === 'visitStand') toLane = this.visitPath.slice(1, -1).reverse()          // via points back to the lane
    else toLane = this.visitBackPath.slice(this.wp, this.visitBackPath.length - 1)                // what is left before the stand spot
    const rest = seat.path.slice(0, -2).reverse()                                                 // lane -> entrance
    const pts = [this.pos.clone(), ...toLane, lane.clone(), ...rest.map(p => p.clone())]
    this.leavePath = smoothRoute(pts.filter((p, i) => i === 0 || p.distanceToSquared(pts[i - 1]) > 0.0025))
    void stand
    this.visiting = false
    this.releaseGate()
    this.state = 'leave'
    this.wp = 1
  }

  private releaseGate(): void {
    if (this.holdsGate) { this.holdsGate = false; this.visitGate?.release(this) }
  }

  dispose(): void {
    this.releaseGate()
    this.skinToken++
    this.skinTexture.dispose()
    this.capeTexture.dispose()
    this.player.traverse(o => {
      const m = o as Mesh
      if (!m.isMesh) return
      m.geometry.dispose()
      const mat = m.material
      if (Array.isArray(mat)) mat.forEach(x => x.dispose()); else mat.dispose()
    })
  }

  update(dt: number): void {
    this.clock += dt
    this.stateTime += dt

    switch (this.state) {
      case 'walk':
      case 'leave':
      case 'visitOut':
      case 'visitBack':
        this.walk(dt)
        break
      case 'visitStand':
        this.standAtFire(dt)
        break
      case 'sitdown':
        this.sit = Math.min(1, this.stateTime / CLIMB_DURATION)
        if (this.sit >= 1) {
          this.state = 'idle'
          this.stateTime = 0
          this.nextGestureIn = 1 + Math.random() * 1.5
          this.visitIn = rand(VISIT_WAIT_MIN_S, VISIT_WAIT_MAX_S)
          this.releaseGate()
        }
        break
      case 'idle':
        this.sit = 1
        this.tickVisitTimer(dt)
        break
      case 'standup':
        this.sit = Math.max(0, 1 - this.stateTime / DESCEND_DURATION)
        if (this.sit <= 0) {
          if (this.visiting) {
            this.startVisitOut()
          } else {
            this.state = 'leave'
            this.wp = 0
            this.stateTime = 0
            this.leavePath = smoothRoute([...this.seat.path].reverse().map(p => p.clone()))
          }
        }
        break
    }
    this.tickGesture(dt)
    this.apply()
  }

  // ── visits ───────────────────────────────────────────────────────────────────
  private tickVisitTimer(dt: number): void {
    if (!VISITS_ENABLED || !this.visitGate) return
    this.visitIn -= dt
    if (this.visitIn > 0) return
    if (this.visitGate.request(this)) {
      this.holdsGate = true
      this.visiting = true
      this.state = 'standup'
      this.stateTime = 0
    } else {
      this.visitIn = VISIT_RETRY_S
    }
  }

  private startVisitOut(): void {
    const seat = this.seat
    const stand = seat.path[seat.path.length - 1]
    const lane = seat.path[seat.path.length - 2]
    this.visitPath = smoothRoute([stand.clone(), lane.clone(), ...seat.visit.via.map(p => p.clone()), seat.visit.spot.clone()])
    this.state = 'visitOut'
    this.wp = 1
    this.stateTime = 0
  }

  /** Standing at the fire: turn to face it, then gestures until the time is up. */
  private standAtFire(dt: number): void {
    const d = angleDiff(this.yaw, this.visitYaw)
    const rate = easeTurnRate(this.yawRate, d, dt, 2.0, TURN_SPEED, 3.5)
    this.yaw += rate * dt
    this.yawRate = rate
    this.phase += Math.abs(rate) * dt * 1.6          // small steps while turning on the spot
    this.stepAmp = lerp(this.stepAmp, Math.min(1, Math.abs(rate) / 2.5), Math.min(1, dt * 8))
    this.visitTimeLeft -= dt
    if (this.visitTimeLeft > 0) return
    const seat = this.seat
    const stand = seat.path[seat.path.length - 1]
    this.visitBackPath = smoothRoute([this.pos.clone(), ...this.visitPath.slice(1, -1).reverse(), stand.clone()])
    this.state = 'visitBack'
    this.wp = 1
    this.stateTime = 0
    this.stepAmp = 0
  }

  private startGesture(): void {
    const standing = this.state === 'visitStand'
    const g = standing ? pickStandGesture(this.lastGesture) : pickGesture(this.lastGesture)
    this.gesture = g
    this.lastGesture = g
    this.gestureT = 0
    this.gestureDur = g.dur[0] + Math.random() * (g.dur[1] - g.dur[0])
    // seated: relative to the seat's facing; standing: relative to where the body points (towards the fire)
    const from = standing ? this.pos : this.seat.pos
    const facing = standing ? this.yaw : this.seatYaw
    const toFire = Math.atan2(FIRE.x - from.x, FIRE.z - from.z)
    const others = this.peers()
    const peer = others.length ? others[Math.floor(Math.random() * others.length)] : null
    this.gestureCtx = {
      yawToFire: angleDiff(facing, toFire),
      yawToPeer: peer ? angleDiff(facing, Math.atan2(peer.x - from.x, peer.z - from.z)) : null,
    }
  }

  private tickGesture(dt: number): void {
    const idle = this.state === 'idle' || (this.state === 'visitStand' && this.stepAmp < 0.15)   // standing gestures start once the turn is done
    const active = this.state === 'idle' || this.state === 'visitStand'
    this.gestureFade = active ? 1 : Math.max(0, this.gestureFade - dt * 3)
    if (!active && this.gestureFade === 0) this.gesture = null

    if (idle && !this.gesture && !this.chatCue) {
      this.nextGestureIn -= dt
      if (this.nextGestureIn <= 0) this.startGesture()
    }
    if (this.gesture) {
      this.gestureT += dt
      if (this.gestureT >= this.gestureDur) {
        this.gesture = null
        this.nextGestureIn = 0.8 + Math.random() * 2.5
      }
    }
    const o = this.pose
    Object.assign(o, zeroPose())
    if (this.gesture) {
      const e = envelope(this.gestureT / this.gestureDur) * this.gestureFade
      this.gesture.apply(o, this.gestureT, this.gestureDur, e, this.gestureCtx)
    }

    // conversation with the villager: blended over whatever gesture is still fading out
    const chatOn = this.chatCue !== null && this.state === 'idle'
    this.chatFade = chatOn ? Math.min(1, this.chatFade + dt * 2.5) : Math.max(0, this.chatFade - dt * 2.5)
    const cue = this.chatCue ?? this.lastChatCue
    if (this.chatFade > 0 && cue) {
      const c = this.chatPose(cue)
      for (const k of Object.keys(o) as (keyof Pose)[]) o[k] = lerp(o[k], c[k], this.chatFade)
    }
  }

  private chatPose(cue: PlayerCue): Pose {
    const c = this.clock
    const from = this.seat.pos
    const yaw = clamp(angleDiff(this.seatYaw, Math.atan2(this.chatTarget.x - from.x, this.chatTarget.z - from.z)), -1.1, 1.1)
    const p = zeroPose()
    p.headY = yaw
    p.headX = 0.04
    switch (cue) {
      case 'offer':    // hand out, palm up, as if showing something
        p.armRX = -1.3 - BASE_ARM_X; p.elbowR = -0.45; p.armRZ = -0.1
        p.bodyX = 0.05
        break
      case 'nod':
        p.headX = Math.sin(c * 6) * 0.2 + 0.05
        break
      case 'shake':
        p.headY = yaw + Math.sin(c * 8) * 0.35
        break
      case 'wave':
        p.armRX = -2.9 - BASE_ARM_X; p.elbowR = -0.4 + Math.sin(c * 11) * 0.5; p.armRZ = -0.3
        break
      default:         // 'listen'
        break
    }
    return p
  }

  private walk(dt: number): void {
    const list = this.state === 'walk' ? this.path
      : this.state === 'visitOut' ? this.visitPath
      : this.state === 'visitBack' ? this.visitBackPath
      : this.leavePath
    const target = list[this.wp]
    if (!target) { this.finishWalk(); return }
    const dx = target.x - this.pos.x
    const dz = target.z - this.pos.z
    const dist = Math.hypot(dx, dz)
    const want = Math.atan2(dx, dz)
    // Ease into the new heading (no constant-rate robot turn) and keep walking while turning, slower
    // the further off-course: corners become arcs instead of stop-and-pivot.
    const diff = angleDiff(this.yaw, want)
    const rate = easeTurnRate(this.yawRate, dist > 0.05 ? diff : 0, dt, 3.0, TURN_SPEED, 4.5)
    this.yawRate = rate
    if (dist > 0.05) this.yaw += rate * dt
    const speedFactor = Math.max(0.25, 1 - Math.abs(diff) / 2.2)
    const step = Math.min(WALK_SPEED * speedFactor * dt, dist)
    if (dist > 1e-4) { this.pos.x += (dx / dist) * step; this.pos.z += (dz / dist) * step }
    this.phase += step * 5.0
    this.gy = this.groundY(this.pos.x, this.pos.z)
    const last = this.wp >= list.length - 1
    if (last ? dist - step < 0.02 : dist < 0.3) {
      this.wp++
      if (this.wp >= list.length) this.finishWalk()
    }
  }

  private finishWalk(): void {
    if (this.state === 'visitOut') {
      this.state = 'visitStand'
      this.stateTime = 0
      this.visitTimeLeft = rand(VISIT_STAND_MIN_S, VISIT_STAND_MAX_S)
      this.visitYaw = Math.atan2(FIRE.x - this.pos.x, FIRE.z - this.pos.z)
      this.nextGestureIn = 1
      this.stepAmp = 0.5
      return
    }
    if (this.state === 'walk' || this.state === 'visitBack') {
      this.visiting = false
      // The route ends walking straight at the log: climb up from here.
      this.state = 'sitdown'
      this.stateTime = 0
      this.yaw = this.seatYaw + Math.PI
      this.climbStartOff = (this.pos.x - this.seat.pos.x) * this.seat.facing.x + (this.pos.z - this.seat.pos.z) * this.seat.facing.z
    } else { this.state = 'gone'; this.rig.visible = false }
  }

  /**
   * Climb onto the log and sit, as one timeline u = 0..1 (played backwards to climb down):
   *   0 to 0.15     face the log, lean in and reach for its top edge
   *   0.12 to 0.45  pull up: hip rises above the log, legs lift forward over it (right one first), body slides over the edge
   *   0.45 to 0.6   settle onto the log top, still facing the log
   *   0.6 to 0.9    swing around 180 degrees on the log, arms come back to rest
   *   0.88 to 1     knees bend, the shins drop over the fire-side edge and start to dangle
   */
  private applyClimb(u: number): void {
    const sk = this.player.skin
    const b = this.bend
    const c = this.clock
    const seat = this.seat
    const cl = (v: number) => Math.max(0, Math.min(1, v))
    const sm = (v: number) => { const x = cl(v); return x * x * (3 - 2 * x) }
    const bump = (v: number) => Math.sin(Math.PI * cl(v))

    const reach = sm(u / 0.15)
    const pull = sm((u - 0.12) / 0.33)
    const slide = sm((u - 0.15) / 0.3)
    const settle = sm((u - 0.45) / 0.15)
    const swivel = sm((u - 0.6) / 0.3)
    const dangle = sm((u - 0.88) / 0.12)
    const rest = swivel   // arms/head/torso return to the seated pose while turning

    // position: along the seat's facing axis from the stand spot, over the edge, to the seated spot
    const off = lerp(lerp(this.climbStartOff, 0.25, slide), SEAT_FRONT, swivel)
    this.rig.position.x = seat.pos.x + seat.facing.x * off
    this.rig.position.z = seat.pos.z + seat.facing.z * off
    const hip = lerp(lerp(this.gy + HIP_HEIGHT, SEAT_TOP + 0.45, pull), SEAT_TOP + SEAT_CLEARANCE, settle)
    this.rig.position.y = hip - HIP_HEIGHT
    this.yaw = this.seatYaw + Math.PI * (1 - swivel)
    this.rig.rotation.y = this.yaw
    this.rig.visible = this.skinReady

    // legs: thighs lift forward over the log, then the shins drop into the seated dangle
    const thighSeat = BASE_LEG_X + Math.sin(c * 0.7) * 0.03
    const liftR = sm((u - 0.18) / 0.2)
    const liftL = sm((u - 0.26) / 0.2)
    sk.rightLeg.rotation.x = lerp(0, thighSeat, liftR)
    sk.leftLeg.rotation.x = lerp(0, thighSeat, liftL)
    sk.rightLeg.rotation.z = lerp(0, -0.04, liftR)
    sk.leftLeg.rotation.z = lerp(0, 0.04, liftL)
    if (b) {
      const kneeSeat = (a: number) => BASE_KNEE + Math.sin(c * 1.9 + a) * 0.28 + Math.sin(c * 0.7 + a * 2) * 0.1
      b.rightLeg.rotation.x = lerp(0.9 * bump((u - 0.18) / 0.3), kneeSeat(0), dangle)
      b.leftLeg.rotation.x = lerp(0.9 * bump((u - 0.26) / 0.3), kneeSeat(Math.PI), dangle)
      const elbow = -0.6 * bump((u - 0.1) / 0.4)
      b.rightArm.rotation.x = Math.min(0, lerp(elbow, BASE_ELBOW, rest))
      b.leftArm.rotation.x = Math.min(0, lerp(elbow, BASE_ELBOW, rest))
    }

    // arms: reach up to the log top, push down on it, then rest
    const armPush = lerp(lerp(0, -1.7, reach), -0.35, sm((u - 0.15) / 0.35))
    sk.rightArm.rotation.x = lerp(armPush, BASE_ARM_X, rest)
    sk.leftArm.rotation.x = lerp(armPush, BASE_ARM_X, rest)
    sk.rightArm.rotation.z = lerp(0.04, 0.12, rest)
    sk.leftArm.rotation.z = lerp(-0.04, -0.12, rest)

    // torso leans into the climb, then straightens up
    const lean = u < 0.45 ? 0.5 * reach : lerp(0.5, 0.06, sm((u - 0.45) / 0.17))
    this.upper.rotation.x = lean
    this.upper.rotation.z = 0
    sk.head.rotation.x = lerp(0.2 * reach * (1 - settle), 0.08, rest)
    sk.head.rotation.y = 0
    sk.head.rotation.z = 0

    this.player.backEquipment = null
  }

  private apply(): void {
    if (this.sit > 0.0001 && this.sit < 0.9999) { this.applyClimb(this.sit); return }
    const s = easeInOut(this.sit)
    const c = this.clock
    const sk = this.player.skin

    // Slide from the stand spot back onto the log while the hip drops to log-top height.
    const seatX = this.seat.pos.x + this.seat.facing.x * SEAT_FRONT
    const seatZ = this.seat.pos.z + this.seat.facing.z * SEAT_FRONT
    this.rig.position.x = lerp(this.pos.x, seatX, s)
    this.rig.position.z = lerp(this.pos.z, seatZ, s)
    const walking = this.state === 'walk' || this.state === 'leave' || this.state === 'visitOut' || this.state === 'visitBack'
    const amp = walking ? 1 : this.state === 'visitStand' ? this.stepAmp : 0
    const bob = Math.abs(Math.cos(this.phase)) * 0.035 * amp
    const seatedDrop = SEAT_TOP - this.gy - HIP_HEIGHT + SEAT_CLEARANCE
    this.rig.position.y = this.gy + lerp(0, seatedDrop, s) + bob + (s > 0.99 ? Math.sin(c * 1.7) * 0.004 : 0)
    this.rig.rotation.y = this.yaw
    this.rig.visible = this.skinReady && this.state !== 'gone'

    const sw = Math.sin(this.phase) * 0.75 * amp
    const o = this.pose

    // Legs: the thigh rotates at the hip, the shin at the knee. Seated, the thighs rest on the log and the
    // shins hang over its edge and swing from the knee - rotating the whole leg instead drives it into the log.
    const thighSeat = BASE_LEG_X + Math.sin(c * 0.7) * 0.03
    const thighR = lerp(sw, thighSeat, s) + o.legRX
    const thighL = lerp(-sw, thighSeat, s) + o.legLX
    sk.rightLeg.rotation.x = thighR
    sk.leftLeg.rotation.x = thighL
    sk.rightLeg.rotation.z = lerp(0, -0.04, s) + o.legRZ
    sk.leftLeg.rotation.z = lerp(0, 0.04, s) + o.legLZ

    sk.rightArm.rotation.x = lerp(-sw * 0.9, BASE_ARM_X + Math.sin(c * 1.1) * 0.03, s) + o.armRX
    sk.leftArm.rotation.x = lerp(sw * 0.9, BASE_ARM_X + Math.sin(c * 1.1 + 1) * 0.03, s) + o.armLX
    sk.rightArm.rotation.z = lerp(0.04, 0.12, s) + o.armRZ
    sk.leftArm.rotation.z = lerp(-0.04, -0.12, s) + o.armLZ

    const b = this.bend
    if (b) {
      const kneeSeat = (a: number) => BASE_KNEE + Math.sin(c * 1.9 + a) * 0.28 + Math.sin(c * 0.7 + a * 2) * 0.1
      // forward swing of each leg lifts the foot (right leg is half a cycle ahead of the left)
      const walkKneeR = Math.max(0, -Math.cos(this.phase)) * 0.9 * amp
      const walkKneeL = Math.max(0, Math.cos(this.phase)) * 0.9 * amp
      const kneeR = lerp(walkKneeR, kneeSeat(0), s) + o.kneeR
      const kneeL = lerp(walkKneeL, kneeSeat(Math.PI), s) + o.kneeL
      // Seated, never let a shin swing back towards the log (thigh + knee <= 0.05) or bend backwards.
      const maxR = lerp(2.4, 0.05 - thighR, s)
      const maxL = lerp(2.4, 0.05 - thighL, s)
      b.rightLeg.rotation.x = Math.min(Math.max(kneeR, 0), Math.max(0, maxR))
      b.leftLeg.rotation.x = Math.min(Math.max(kneeL, 0), Math.max(0, maxL))

      const elbowWalkR = Math.min(0, -sw * 0.9) * 0.55
      const elbowWalkL = Math.min(0, sw * 0.9) * 0.55
      b.rightArm.rotation.x = Math.min(0, lerp(elbowWalkR, BASE_ELBOW + Math.sin(c * 1.1) * 0.04, s) + o.elbowR)
      b.leftArm.rotation.x = Math.min(0, lerp(elbowWalkL, BASE_ELBOW + Math.sin(c * 1.1 + 1) * 0.04, s) + o.elbowL)
    }

    this.upper.rotation.x = lerp(0, 0.06 + Math.sin(c * 1.2) * 0.012, s)
    const look = Math.sin(c * 0.45) * 0.35 * Math.max(0, Math.sin(c * 0.21 + 1))
    sk.head.rotation.y = lerp(0, look, s)
    sk.head.rotation.x = lerp(0, 0.08 + Math.sin(c * 0.9) * 0.04, s)

    // standing at the fire: a little life between the gestures (breathing, weight shift, slow head movement)
    if (this.state === 'visitStand') {
      sk.head.rotation.y += Math.sin(c * 0.5) * 0.12
      sk.head.rotation.x += Math.sin(c * 0.8) * 0.03
      sk.rightArm.rotation.x += Math.sin(c * 1.3) * 0.03
      sk.leftArm.rotation.x += Math.sin(c * 1.3 + 1.2) * 0.03
      this.upper.rotation.x += Math.sin(c * 1.2) * 0.01
    }

    // Gesture offsets for head and torso (limbs were handled above)
    sk.head.rotation.x += o.headX; sk.head.rotation.y += o.headY; sk.head.rotation.z = o.headZ
    this.upper.rotation.x += o.bodyX; this.upper.rotation.z = o.bodyZ

    // A cape would clip straight through the log while seated.
    this.player.backEquipment = this.capeLoaded && s < 0.15 ? 'cape' : null
  }
}
