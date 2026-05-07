import { existsSync, mkdirSync, readdirSync, renameSync, copyFileSync, statSync, unlinkSync } from 'fs'
import { join, basename, extname } from 'path'
import { getSettings } from './settingsService'
import { getProfile } from './profileService'
import { shell } from 'electron'

export interface ModInfo {
  id: string
  name: string
  fileName: string
  enabled: boolean
  filePath: string
  fileSize: number
  modifiedAt: string
}

function getModsDir(profileId: string): string {
  const profile = getProfile(profileId)
  const settings = getSettings()
  const gameDir = profile?.gameDir || settings.game.defaultGameDir
  return join(gameDir, 'mods')
}

function ensureModsDir(modsDir: string): void {
  if (!existsSync(modsDir)) mkdirSync(modsDir, { recursive: true })
}

export function listMods(profileId: string): ModInfo[] {
  const modsDir = getModsDir(profileId)
  ensureModsDir(modsDir)

  return readdirSync(modsDir, { withFileTypes: true })
    .filter(f => f.isFile())
    .filter(f => f.name.endsWith('.jar') || f.name.endsWith('.jar.disabled'))
    .map(f => {
      const filePath = join(modsDir, f.name)
      const enabled = f.name.endsWith('.jar')
      const displayName = f.name
        .replace(/\.jar\.disabled$/, '')
        .replace(/\.jar$/, '')
        .replace(/[-_]/g, ' ')
        .replace(/\b\w/g, c => c.toUpperCase())
      const stat = statSync(filePath)

      return {
        id: f.name,
        name: displayName,
        fileName: f.name,
        enabled,
        filePath,
        fileSize: stat.size,
        modifiedAt: stat.mtime.toISOString(),
      }
    })
    .sort((a, b) => a.name.localeCompare(b.name))
}

export function toggleMod(profileId: string, modId: string): ModInfo[] {
  const modsDir = getModsDir(profileId)
  const filePath = join(modsDir, modId)

  if (!existsSync(filePath)) throw new Error(`Mod file not found: ${modId}`)

  if (modId.endsWith('.jar.disabled')) {
    const newPath = filePath.replace(/\.disabled$/, '')
    renameSync(filePath, newPath)
  } else if (modId.endsWith('.jar')) {
    renameSync(filePath, `${filePath}.disabled`)
  }

  return listMods(profileId)
}

export function installMod(profileId: string, sourcePath: string): ModInfo[] {
  const modsDir = getModsDir(profileId)
  ensureModsDir(modsDir)

  const fileName = basename(sourcePath)
  if (!fileName.endsWith('.jar')) throw new Error('Only .jar mod files are supported')

  const destPath = join(modsDir, fileName)
  copyFileSync(sourcePath, destPath)

  return listMods(profileId)
}

export function deleteMod(profileId: string, modId: string): ModInfo[] {
  const modsDir = getModsDir(profileId)
  const filePath = join(modsDir, modId)
  if (existsSync(filePath)) unlinkSync(filePath)
  return listMods(profileId)
}

export function openModsFolder(profileId: string): void {
  const modsDir = getModsDir(profileId)
  ensureModsDir(modsDir)
  shell.openPath(modsDir)
}
