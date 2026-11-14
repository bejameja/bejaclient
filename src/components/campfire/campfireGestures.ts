// Seated idle gestures for the campfire. Each gesture adds offsets on top of the base seated pose
// (resting arms, swinging legs, breathing), faded in and out by an envelope so they never pop.
//
// Rotation conventions (skinview3d rig, character faces +Z):
//   head.x  negative = look up, positive = look down      body/upper.x  positive = lean forward
//   arm.x   negative = raise forward/up                   arm.z  positive on the right arm / negative on the left = towards the body
//   leg.x   negative = raise forward                      leg.z  positive on the left leg / negative on the right = outwards
//   knee    positive = shin swings back (a seated shin hangs down at ~1.25)   elbow  negative = forearm swings forward

export const BASE_ARM_X = -0.55   // resting seated arm
export const BASE_LEG_X = -1.5    // resting seated thigh (almost horizontal)
export const BASE_KNEE = 1.25     // resting seated knee bend, shin hangs down
export const BASE_ELBOW = -0.35   // resting seated elbow bend

export interface Pose {
  headX: number; headY: number; headZ: number
  bodyX: number; bodyZ: number
  armRX: number; armRZ: number
  armLX: number; armLZ: number
  legRX: number; legRZ: number
  legLX: number; legLZ: number
  kneeR: number; kneeL: number
  elbowR: number; elbowL: number
}

export const zeroPose = (): Pose => ({
  headX: 0, headY: 0, headZ: 0, bodyX: 0, bodyZ: 0,
  armRX: 0, armRZ: 0, armLX: 0, armLZ: 0,
  legRX: 0, legRZ: 0, legLX: 0, legLZ: 0,
  kneeR: 0, kneeL: 0, elbowR: 0, elbowL: 0,
})

export interface GestureContext {
  /** Head yaw (relative to the seat) that looks straight at the fire. */
  yawToFire: number
  /** Head yaw towards a random seated neighbour, or null when alone. */
  yawToPeer: number | null
}

export interface Gesture {
  name: string
  weight: number
  /** [min, max] seconds */
  dur: [number, number]
  /** t = seconds into the gesture, d = total length, e = smooth 0→1→0 envelope. */
  apply: (o: Pose, t: number, d: number, e: number, c: GestureContext) => void
}

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v))
const both = (o: Pose, target: number, e: number) => {
  o.armRX += (target - BASE_ARM_X) * e
  o.armLX += (target - BASE_ARM_X) * e
}

export const GESTURES: Gesture[] = [
  { name: 'lookFire', weight: 2, dur: [2.5, 4.5], apply: (o, t, d, e, c) => {
    o.headY += clamp(c.yawToFire, -1, 1) * e
    o.headX += 0.05 * e
  } },
  { name: 'lookPeer', weight: 3, dur: [2.2, 4], apply: (o, t, d, e, c) => {
    const y = c.yawToPeer ?? Math.sin((t / d) * Math.PI * 2) * 0.8
    o.headY += clamp(y, -1.1, 1.1) * e
    o.headZ += 0.06 * Math.sign(y || 1) * e
  } },
  { name: 'lookAround', weight: 2, dur: [3.5, 5.5], apply: (o, t, d, e) => {
    o.headY += Math.sin((t / d) * Math.PI * 2) * 0.85 * e
    o.headX += 0.04 * e
  } },
  { name: 'lookUp', weight: 2, dur: [2.5, 4], apply: (o, t, d, e) => {
    o.headX += -0.5 * e
    o.bodyX += -0.04 * e
  } },
  { name: 'lookDown', weight: 1, dur: [2, 3.5], apply: (o, t, d, e) => {
    o.headX += 0.45 * e
    o.bodyX += 0.08 * e
  } },
  { name: 'warmHands', weight: 3, dur: [3, 5], apply: (o, t, d, e) => {
    both(o, -1.35, e)
    o.elbowR += -0.45 * e; o.elbowL += -0.45 * e
    o.armRZ += -0.05 * e; o.armLZ += 0.05 * e
    o.armRX += Math.sin(t * 9) * 0.03 * e
    o.armLX += Math.sin(t * 9 + 1.7) * 0.03 * e
    o.bodyX += 0.1 * e
    o.headX += 0.05 * e
  } },
  { name: 'stretch', weight: 2, dur: [2.6, 3.4], apply: (o, t, d, e) => {
    both(o, -3.0, e)
    o.armRZ += -0.35 * e; o.armLZ += 0.35 * e
    o.bodyX += -0.12 * e
    o.headX += -0.25 * e
  } },
  { name: 'scratchHead', weight: 2, dur: [2.2, 3.2], apply: (o, t, d, e) => {
    o.armRX += (-2.6 - BASE_ARM_X) * e
    o.elbowR += (-1.2 + Math.sin(t * 13) * 0.2) * e
    o.armRZ += 0.45 * e
    o.headZ += 0.14 * e
  } },
  { name: 'leanForward', weight: 2, dur: [3.5, 6], apply: (o, t, d, e) => {
    o.bodyX += 0.38 * e
    o.headX += -0.25 * e
    both(o, -0.9, e)
    o.armRZ += 0.18 * e; o.armLZ += -0.18 * e
  } },
  { name: 'leanBack', weight: 2, dur: [3, 5], apply: (o, t, d, e) => {
    o.bodyX += -0.18 * e
    o.headX += -0.1 * e
    both(o, 0.5, e)
  } },
  { name: 'shiver', weight: 1, dur: [2.5, 4], apply: (o, t, d, e) => {
    o.bodyZ += Math.sin(t * 45) * 0.025 * e
    o.bodyX += Math.sin(t * 37) * 0.015 * e
    both(o, -0.9, e * 0.6)
    o.armRZ += 0.3 * e; o.armLZ += -0.3 * e
    o.headX += 0.18 * e
  } },
  { name: 'legKick', weight: 3, dur: [2.4, 3.6], apply: (o, t, d, e) => {
    // the shins swing from the knee; the thighs stay put on the log
    o.kneeR += Math.sin(t * 6.5) * 0.4 * e
    o.kneeL += Math.sin(t * 6.5 + Math.PI) * 0.4 * e
    o.bodyZ += Math.sin(t * 6.5) * 0.015 * e
  } },
  { name: 'legBounce', weight: 2, dur: [2.5, 4], apply: (o, t, d, e) => {
    const s = Math.sin(t * 8) * 0.3 * e
    o.kneeR += s; o.kneeL += s
    o.headX += Math.sin(t * 8) * 0.02 * e
  } },
  { name: 'clap', weight: 2, dur: [2.4, 3.2], apply: (o, t, d, e) => {
    both(o, -1.55, e)
    o.elbowR += -0.7 * e; o.elbowL += -0.7 * e
    const k = (0.2 + (0.5 + 0.5 * Math.sin(t * 13)) * 0.3) * e
    o.armRZ += k; o.armLZ += -k
    o.headX += 0.05 * e
  } },
  { name: 'wave', weight: 2, dur: [2.6, 3.6], apply: (o, t, d, e, c) => {
    o.armRX += (-2.9 - BASE_ARM_X) * e
    o.elbowR += (-0.4 + Math.sin(t * 11) * 0.5) * e
    o.armRZ += -0.3 * e
    o.headY += clamp(c.yawToPeer ?? 0.4, -0.8, 0.8) * 0.6 * e
    o.headZ += -0.08 * e
  } },
  { name: 'pokeFire', weight: 2, dur: [3, 5], apply: (o, t, d, e, c) => {
    o.armRX += (-1.15 - BASE_ARM_X) * e + Math.sin(t * 4.5) * 0.22 * e
    o.armRZ += 0.05 * e
    o.bodyX += 0.12 * e
    o.headX += 0.25 * e
    o.headY += clamp(c.yawToFire, -0.7, 0.7) * 0.8 * e
  } },
  { name: 'yawn', weight: 1, dur: [2.8, 3.6], apply: (o, t, d, e) => {
    o.headX += -0.55 * e
    o.armRX += (-2.2 - BASE_ARM_X) * e
    o.elbowR += -1.1 * e
    o.armRZ += 0.35 * e
    o.bodyX += -0.1 * e
  } },
  { name: 'nod', weight: 1, dur: [2, 3], apply: (o, t, d, e) => {
    o.headX += Math.sin(t * 6) * 0.22 * e
  } },
  { name: 'shake', weight: 1, dur: [2, 3], apply: (o, t, d, e) => {
    o.headY += Math.sin(t * 7) * 0.4 * e
  } },
  { name: 'rock', weight: 2, dur: [4, 7], apply: (o, t, d, e) => {
    const s = Math.sin(t * 1.8)
    o.bodyZ += s * 0.09 * e
    o.headZ += -s * 0.06 * e
    o.kneeR += s * 0.12 * e; o.kneeL += -s * 0.12 * e
  } },
  { name: 'drumKnees', weight: 2, dur: [3, 5], apply: (o, t, d, e) => {
    o.armRX += Math.max(0, Math.sin(t * 9)) * 0.28 * e
    o.armLX += Math.max(0, Math.sin(t * 9 + 2.5)) * 0.28 * e
    o.headX += Math.sin(t * 9) * 0.05 * e
    o.headZ += Math.sin(t * 4.5) * 0.05 * e
  } },
  { name: 'tiltHead', weight: 2, dur: [2.5, 4], apply: (o, t, d, e) => {
    o.headZ += 0.3 * e
    o.bodyZ += 0.04 * e
    o.headX += 0.03 * e
  } },
  { name: 'sip', weight: 1, dur: [3, 4.2], apply: (o, t, d, e) => {
    o.armRX += (-2.25 - BASE_ARM_X) * e
    o.elbowR += -1.3 * e
    o.armRZ += 0.3 * e
    o.headX += -0.28 * e
  } },
  { name: 'stretchLegs', weight: 2, dur: [3, 5], apply: (o, t, d, e) => {
    // legs straight out in front: knees fully extended
    o.kneeR += -BASE_KNEE * e; o.kneeL += -BASE_KNEE * e
    o.legRZ += -0.12 * e; o.legLZ += 0.12 * e
    both(o, 0.6, e)
    o.bodyX += -0.2 * e
    o.headX += -0.2 * e
  } },
]

// Standing gestures for the visit at the fire. The seated ones above are additive offsets on the seated pose (legs
// horizontal, arms resting forward), so they do not fit a standing base: here the base is the neutral standing pose
// (everything 0), so every value is absolute. The character faces the fire.
export const STAND_GESTURES: Gesture[] = [
  { name: 'warmHands', weight: 3, dur: [4, 7], apply: (o, t, d, e) => {
    o.armRX += -1.3 * e; o.armLX += -1.3 * e
    o.elbowR += -0.5 * e; o.elbowL += -0.5 * e
    o.armRX += Math.sin(t * 9) * 0.03 * e
    o.armLX += Math.sin(t * 9 + 1.7) * 0.03 * e
    o.bodyX += 0.08 * e
    o.headX += 0.12 * e
  } },
  { name: 'rubHands', weight: 2, dur: [3, 5], apply: (o, t, d, e) => {
    o.armRX += -1.15 * e; o.armLX += -1.15 * e
    o.elbowR += -0.95 * e; o.elbowL += -0.95 * e
    const k = (0.2 + 0.12 * Math.sin(t * 10)) * e
    o.armRZ += k; o.armLZ += -k
    o.bodyX += 0.06 * e
    o.headX += 0.1 * e
  } },
  { name: 'poke', weight: 2, dur: [3, 5], apply: (o, t, d, e) => {
    o.armRX += (-1.0 + Math.sin(t * 4.5) * 0.2) * e
    o.elbowR += -0.35 * e
    o.bodyX += 0.14 * e
    o.headX += 0.3 * e
  } },
  { name: 'stretch', weight: 2, dur: [2.6, 3.4], apply: (o, t, d, e) => {
    o.armRX += -3.0 * e; o.armLX += -3.0 * e
    o.armRZ += -0.35 * e; o.armLZ += 0.35 * e
    o.bodyX += -0.12 * e
    o.headX += -0.25 * e
  } },
  { name: 'lookPeer', weight: 3, dur: [2.2, 4], apply: (o, t, d, e, c) => {
    const y = c.yawToPeer ?? Math.sin((t / d) * Math.PI * 2) * 0.8
    o.headY += clamp(y, -1.1, 1.1) * e
    o.headZ += 0.05 * Math.sign(y || 1) * e
  } },
  { name: 'lookAround', weight: 2, dur: [3.5, 5.5], apply: (o, t, d, e) => {
    o.headY += Math.sin((t / d) * Math.PI * 2) * 0.85 * e
    o.headX += 0.04 * e
  } },
  { name: 'lookUp', weight: 2, dur: [2.5, 4], apply: (o, t, d, e) => {
    o.headX += -0.5 * e
    o.bodyX += -0.04 * e
  } },
  { name: 'scratchHead', weight: 1, dur: [2.2, 3.2], apply: (o, t, d, e) => {
    o.armRX += -2.6 * e
    o.elbowR += (-1.2 + Math.sin(t * 13) * 0.2) * e
    o.armRZ += 0.45 * e
    o.headZ += 0.14 * e
  } },
  { name: 'shiver', weight: 1, dur: [2.5, 4], apply: (o, t, d, e) => {
    o.bodyZ += Math.sin(t * 45) * 0.025 * e
    o.bodyX += Math.sin(t * 37) * 0.015 * e + 0.05 * e
    o.armRX += -0.5 * e; o.armLX += -0.5 * e
    o.elbowR += -0.8 * e; o.elbowL += -0.8 * e
    o.armRZ += 0.3 * e; o.armLZ += -0.3 * e
    o.headX += 0.15 * e
  } },
]

function pickFrom(list: Gesture[], previous: Gesture | null): Gesture {
  const total = list.reduce((s, g) => s + g.weight, 0)
  for (;;) {
    let r = Math.random() * total
    for (const g of list) {
      r -= g.weight
      if (r <= 0) { if (g !== previous) return g; break }
    }
  }
}

/** Weighted random pick that never repeats the previous gesture (seated gestures). */
export function pickGesture(previous: Gesture | null): Gesture { return pickFrom(GESTURES, previous) }

/** Same for the standing gestures of a visit. */
export function pickStandGesture(previous: Gesture | null): Gesture { return pickFrom(STAND_GESTURES, previous) }

/** Smooth in/out: 0 → 1 over the first 20 %, hold, 1 → 0 over the last 20 %. */
export function envelope(p: number): number {
  const x = Math.max(0, Math.min(1, Math.min(p / 0.2, (1 - p) / 0.2)))
  return x * x * (3 - 2 * x)
}
