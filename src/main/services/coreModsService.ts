import { existsSync, mkdirSync, readdirSync } from 'fs'
import { join } from 'path'
import { getProjectVersions, downloadFile } from './modrinthService'

const CORE_MODS = [
  { id: 'AANobbMI', slug: 'sodium',      name: 'Sodium'      },
  { id: 'gvQqBUqZ', slug: 'lithium',     name: 'Lithium'     },
  { id: 'uXXizFIs', slug: 'ferritecore', name: 'FerriteCore' },
]

// Avoid re-checking on every launch within the same session
const checkedCache = new Set<string>()

function getExistingModPaths(dir: string): string[] {
  if (!existsSync(dir)) return []
  return readdirSync(dir)
    .filter(f => CORE_MODS.some(m => f.startsWith(m.slug + '-') && f.endsWith('.jar')))
    .map(f => join(dir, f))
}

export async function ensureCoreMods(
  binDir: string,
  mcVersion: string,
  onLog: (line: string) => void,
): Promise<string[]> {
  const cacheKey = `${binDir}|${mcVersion}`
  if (checkedCache.has(cacheKey)) return getExistingModPaths(binDir)

  if (!existsSync(binDir)) mkdirSync(binDir, { recursive: true })
  const existing = new Set(readdirSync(binDir))

  for (const mod of CORE_MODS) {
    const alreadyHave = [...existing].some(f => f.startsWith(mod.slug + '-') && f.endsWith('.jar'))
    if (alreadyHave) continue

    onLog(`[BejaClient] Downloading ${mod.name} for MC ${mcVersion}…`)
    try {
      const versions = await getProjectVersions(mod.id, mcVersion, 'fabric')
      if (!versions.length) {
        onLog(`[BejaClient] ${mod.name}: no Fabric build for ${mcVersion}, skipping`)
        continue
      }
      const best = versions[0]
      const file = best.files.find((f: { primary: boolean }) => f.primary) ?? best.files[0]
      if (!file) continue
      const dest = join(binDir, file.filename)
      await downloadFile(file.url, dest)
      onLog(`[BejaClient] ✓ ${mod.name} ${best.version_number}`)
      existing.add(file.filename)
    } catch (err) {
      onLog(`[BejaClient] ${mod.name} download failed: ${String(err)} — continuing`)
      // Don't cache — retry next launch
      continue
    }
  }

  checkedCache.add(cacheKey)
  return getExistingModPaths(binDir)
}
