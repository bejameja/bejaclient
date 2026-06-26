import { existsSync, readFileSync, writeFileSync } from 'fs'
import { join } from 'path'

// Key-value pairs to force regardless of what options.txt currently says
const PERF_OVERRIDES: Record<string, string> = {
  maxFps:                  '260',     // 260 = unlimited in MC 1.17+
  enableVsync:             'false',
  renderDistance:          '8',
  simulationDistance:      '6',
  particles:               '2',       // minimal
  cloudRenderMode:         'off',
  entityShadows:           'false',
  ao:                      'false',
  biomeBlendRadius:        '0',
}

export function patchOptionsFile(gameDir: string): void {
  const optPath = join(gameDir, 'options.txt')

  let lines: string[] = []
  if (existsSync(optPath)) {
    lines = readFileSync(optPath, 'utf-8').split('\n')
  }

  // Build a map of existing entries
  const map = new Map<string, string>()
  for (const line of lines) {
    const colon = line.indexOf(':')
    if (colon === -1) continue
    map.set(line.slice(0, colon).trim(), line.slice(colon + 1).trim())
  }

  // Apply overrides
  for (const [key, value] of Object.entries(PERF_OVERRIDES)) {
    map.set(key, value)
  }

  // Reconstruct file
  const out = [...map.entries()].map(([k, v]) => `${k}:${v}`).join('\n') + '\n'
  writeFileSync(optPath, out, 'utf-8')
}
