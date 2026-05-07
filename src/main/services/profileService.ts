import { app } from 'electron'
import { join } from 'path'
import { readFileSync, writeFileSync, existsSync } from 'fs'
import { randomUUID } from 'crypto'

export interface LaunchProfile {
  id: string
  name: string
  version: string
  loader: 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge'
  loaderVersion: string
  gameDir: string
  minRam: number
  maxRam: number
  javaPath: string
  jvmArgs: string
  resolution: { width: number; height: number }
  useBejaClient: boolean
  createdAt: string
  lastPlayed: string | null
}

function getProfilesPath() {
  return join(app.getPath('userData'), 'profiles.json')
}

export function listProfiles(): LaunchProfile[] {
  const path = getProfilesPath()
  if (!existsSync(path)) return []
  try {
    return JSON.parse(readFileSync(path, 'utf-8'))
  } catch {
    return []
  }
}

export function saveProfiles(profiles: LaunchProfile[]): void {
  writeFileSync(getProfilesPath(), JSON.stringify(profiles, null, 2), 'utf-8')
}

export function createProfile(data: Omit<LaunchProfile, 'id' | 'createdAt' | 'lastPlayed'>): LaunchProfile {
  const profile: LaunchProfile = {
    ...data,
    id: randomUUID(),
    createdAt: new Date().toISOString(),
    lastPlayed: null,
  }
  const profiles = listProfiles()
  profiles.push(profile)
  saveProfiles(profiles)
  return profile
}

export function updateProfile(id: string, data: Partial<LaunchProfile>): LaunchProfile | null {
  const profiles = listProfiles()
  const idx = profiles.findIndex(p => p.id === id)
  if (idx === -1) return null
  profiles[idx] = { ...profiles[idx], ...data }
  saveProfiles(profiles)
  return profiles[idx]
}

export function deleteProfile(id: string): boolean {
  const profiles = listProfiles()
  const filtered = profiles.filter(p => p.id !== id)
  if (filtered.length === profiles.length) return false
  saveProfiles(filtered)
  return true
}

export function getProfile(id: string): LaunchProfile | null {
  return listProfiles().find(p => p.id === id) ?? null
}
