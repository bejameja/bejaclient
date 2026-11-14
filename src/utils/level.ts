// Player level from lifetime XP (player_pass.xp). XP needed to reach level n is 150 * (n-1)^2,
// so early levels come quickly (L2 = 150 XP, one claimed quest) and later ones slow down.
const BASE = 150

export interface LevelInfo { level: number; progress: number; xpIntoLevel: number; xpForNext: number }

export function levelFromXp(xp: number): LevelInfo {
  const x = Math.max(0, Math.floor(xp || 0))
  const level = Math.floor(Math.sqrt(x / BASE)) + 1
  const start = BASE * (level - 1) ** 2
  const next = BASE * level ** 2
  return { level, progress: (x - start) / (next - start), xpIntoLevel: x - start, xpForNext: next - start }
}
