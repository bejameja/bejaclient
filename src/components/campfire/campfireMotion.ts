import { Vector3 } from 'three'

/**
 * Chaikin corner cutting: turns a sharp-cornered route into a gentle curve. Start and end stay where they are, so the
 * last stretch still ends straight at the target.
 */
export function smoothRoute(pts: Vector3[], iterations = 2): Vector3[] {
  if (pts.length < 3) return pts
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

/**
 * Turning with inertia: the turning speed eases towards what the angle difference asks for instead of jumping
 * to it, so a heading change starts slowly, swings, and settles. Returns the new turning speed (rad/s).
 */
export function easeTurnRate(rate: number, diff: number, dt: number, gain: number, maxRate: number, response: number): number {
  const want = Math.max(-maxRate, Math.min(maxRate, diff * gain))
  return rate + (want - rate) * Math.min(1, dt * response)
}
