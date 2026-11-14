import type { IconName } from './campfireIcons'

// Keeps the campfire scene the same on every party member's screen. There is no server logic for this: the messages
// travel over the party's existing `party:emote` relay (the server forwards the string to the rest of the party
// unchanged), prefixed so they can be told apart.
//
// Who decides what:
//  - every player's own character decides its gestures and visits to the fire and tells the others,
//  - the player on seat 0 (the party leader) decides when the dog and the villager come, and tells the others.
// Receivers just play what they are told; a character whose owner sends nothing (an older launcher) keeps running its
// own random idle animations.

export type SyncMsg =
  | { k: 'g'; n: string; d: number; p: string | null }        // gesture: name, duration (s), looked-at peer ('v' = villager, else the peer's seat number)
  | { k: 'v'; s: number }                                     // visit to the fire: time spent standing there (s)
  | { k: 'dog'; a: 'come' | 'away' }
  | { k: 'vil'; plan: [IconName, IconName, boolean][] }       // villager visit: the trades, as [give, get, accepted]

export const SYNC_PREFIX = 'cf1:'

export function encodeSync(msg: SyncMsg): string {
  return SYNC_PREFIX + JSON.stringify(msg)
}

export function decodeSync(raw: unknown): SyncMsg | null {
  if (typeof raw !== 'string' || !raw.startsWith(SYNC_PREFIX)) return null
  try {
    const m = JSON.parse(raw.slice(SYNC_PREFIX.length))
    if (!m || typeof m !== 'object') return null
    switch (m.k) {
      case 'g': return typeof m.n === 'string' && typeof m.d === 'number' ? { k: 'g', n: m.n, d: Math.min(12, Math.max(0.5, m.d)), p: typeof m.p === 'string' ? m.p : null } : null
      case 'v': return typeof m.s === 'number' ? { k: 'v', s: Math.min(60, Math.max(5, m.s)) } : null
      case 'dog': return m.a === 'come' || m.a === 'away' ? { k: 'dog', a: m.a } : null
      case 'vil': return Array.isArray(m.plan) && m.plan.length > 0 && m.plan.length <= 8
        ? { k: 'vil', plan: m.plan.map((x: any) => [String(x[0]), String(x[1]), !!x[2]]) as [IconName, IconName, boolean][] }
        : null
      default: return null
    }
  } catch {
    return null
  }
}
