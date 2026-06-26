import { readdirSync, existsSync, mkdirSync, renameSync } from 'fs'
import { join, extname } from 'path'

interface BlacklistEntry {
  pattern: RegExp
  modId: string
  reason: string
  suggestion: string
}

const BLACKLIST: BlacklistEntry[] = [
  {
    pattern: /^optifine[-_]/i,
    modId: 'OptiFine',
    reason: 'Incompatible with Sodium renderer.',
    suggestion: 'Remove OptiFine. BejaClient bundles Sodium for performance.',
  },
  {
    pattern: /^optifabric[-_]/i,
    modId: 'OptiFabric',
    reason: 'OptiFabric loads OptiFine which conflicts with Sodium.',
    suggestion: 'Remove OptiFabric and OptiFine.',
  },
  {
    pattern: /^rubidium[-_]/i,
    modId: 'Rubidium',
    reason: 'Rubidium is a Sodium port — duplicates Sodium bundled by BejaClient.',
    suggestion: 'Remove Rubidium. BejaClient bundles Sodium directly.',
  },
  {
    pattern: /^embeddium[-_]/i,
    modId: 'Embeddium',
    reason: 'Embeddium conflicts with Sodium renderer.',
    suggestion: 'Remove Embeddium.',
  },
  {
    pattern: /^vulkanmod[-_]/i,
    modId: 'VulkanMod',
    reason: 'VulkanMod replaces the renderer — incompatible with Sodium.',
    suggestion: 'Remove VulkanMod.',
  },
  {
    pattern: /^canvas[-_]/i,
    modId: 'Canvas',
    reason: 'Canvas renderer conflicts with Sodium.',
    suggestion: 'Remove Canvas.',
  },
  {
    pattern: /^ferritecore[-_]|^ferrite-core[-_]/i,
    modId: 'FerriteCore (manual)',
    reason: 'BejaClient bundles FerriteCore — duplicate causes class conflicts.',
    suggestion: 'Remove this FerriteCore JAR; BejaClient loads it automatically.',
  },
  {
    pattern: /^sodium-fabric[-_]|^sodium[-_]\d/i,
    modId: 'Sodium (manual)',
    reason: 'BejaClient bundles Sodium — duplicate causes class conflicts.',
    suggestion: 'Remove this Sodium JAR; BejaClient loads it automatically.',
  },
  {
    pattern: /^lithium[-_]/i,
    modId: 'Lithium (manual)',
    reason: 'BejaClient bundles Lithium — duplicate causes class conflicts.',
    suggestion: 'Remove this Lithium JAR; BejaClient loads it automatically.',
  },
]

export interface CompatibilityResult {
  moved: Array<{ file: string; modId: string; reason: string; suggestion: string }>
  clean: boolean
}

export function enforceModCompatibility(
  modsDir: string,
  onLog: (line: string) => void,
): CompatibilityResult {
  if (!existsSync(modsDir)) return { moved: [], clean: true }

  const disabledDir = join(modsDir, '.disabled')
  const moved: CompatibilityResult['moved'] = []

  const jars = readdirSync(modsDir).filter(
    f => extname(f).toLowerCase() === '.jar' && !f.startsWith('beja-'),
  )

  for (const jar of jars) {
    const entry = BLACKLIST.find(e => e.pattern.test(jar))
    if (!entry) continue

    if (!existsSync(disabledDir)) mkdirSync(disabledDir, { recursive: true })

    const src  = join(modsDir, jar)
    const dest = join(disabledDir, jar)

    try {
      renameSync(src, dest)
      onLog(`[ModChecker] Disabled ${jar} → .disabled/  |  Reason: ${entry.reason}`)
      moved.push({ file: jar, modId: entry.modId, reason: entry.reason, suggestion: entry.suggestion })
    } catch (err) {
      onLog(`[ModChecker] WARN: Could not move ${jar}: ${String(err)}`)
    }
  }

  if (moved.length === 0) onLog('[ModChecker] No conflicting mods found.')
  return { moved, clean: moved.length === 0 }
}
