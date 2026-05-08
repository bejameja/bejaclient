import { ChildProcess } from 'child_process'
import { copyFileSync, existsSync, mkdirSync, readdirSync, unlinkSync } from 'fs'
import { join } from 'path'
import { app } from 'electron'
import { getSettings } from './settingsService'
import { getProfile, type LaunchProfile } from './profileService'
import { getSelectedAccount, refreshAccount } from './authService'
import { checkAndUpdateClientJar, resolveAdapterJar, resolveBootstrapJar } from './clientUpdateService'

function removeBejaModJars(gameDir: string, onLog: (line: string) => void): void {
  const modsDir = join(gameDir, 'mods')
  if (!existsSync(modsDir)) return
  // Remove legacy bejaclient-*.jar dropped in mods/ (BejaClient runs as agent, not a mod)
  const stale = readdirSync(modsDir).filter(
    f => f.startsWith('bejaclient-') && f.endsWith('.jar')
  )
  for (const f of stale) {
    try {
      unlinkSync(join(modsDir, f))
      onLog(`[BejaClient] Removed stale mod JAR: ${f}`)
    } catch { /* ignore */ }
  }
}

function resolveVersionId(mcVersion: string, loader: string, gameDir: string): string {
  if (loader === 'vanilla') return mcVersion
  const versionsDir = join(gameDir, 'versions')
  if (!existsSync(versionsDir)) return mcVersion
  const dirs = readdirSync(versionsDir, { withFileTypes: true })
    .filter(d => d.isDirectory())
    .map(d => d.name)
  const loaderKey = loader === 'neoforge' ? 'neoforge' : loader
  const match = dirs.find(id =>
    id.includes(mcVersion) && id.toLowerCase().includes(loaderKey.toLowerCase())
  )
  return match ?? mcVersion
}

function buildJvmArgs(profile: LaunchProfile): string[] {
  const args: string[] = profile.jvmArgs ? profile.jvmArgs.split(' ').filter(Boolean) : []

  if (profile.useBejaClient) {
    const bootstrapJar = resolveBootstrapJar()
    if (bootstrapJar) {
      args.unshift(`-javaagent:${bootstrapJar}=${profile.version}`)
      console.log(`[BejaBootstrap] Injecting: ${bootstrapJar} (MC ${profile.version})`)
    } else {
      console.warn('[BejaBootstrap] Bootstrap JAR not found — client will launch without BejaClient.')
    }
  }

  return args
}

let activeProcess: ChildProcess | null = null

export function isRunning(): boolean {
  return activeProcess !== null && !activeProcess.killed
}

export async function launchGame(
  profileId: string,
  onLog: (line: string) => void,
  onStatus: (status: string) => void,
): Promise<void> {
  if (isRunning()) throw new Error('Game is already running')

  const profile = getProfile(profileId)
  if (!profile) throw new Error(`Profile ${profileId} not found`)

  let account = getSelectedAccount()
  if (!account) throw new Error('No account selected. Please log in first.')

  if (account.tokenExpiry < Date.now() + 60000) {
    onStatus('Refreshing authentication...')
    const refreshed = await refreshAccount(account.id)
    if (!refreshed) throw new Error('Failed to refresh account. Please log in again.')
    account = refreshed
  }

  const settings = getSettings()
  const javaPath = profile.javaPath || settings.game.defaultJavaPath || 'java'
  const gameDir = profile.gameDir || settings.game.defaultGameDir

  // Auto-update the BejaClient JAR before launch (non-fatal if offline)
  if (profile.useBejaClient) {
    await checkAndUpdateClientJar(onLog, onStatus)
  }

  const { launch } = await import('@xmcl/core')

  onStatus('starting')

  const versionId = resolveVersionId(profile.version, profile.loader, gameDir)
  onLog(`[Launcher] Java: ${javaPath}`)
  onLog(`[Launcher] Game dir: ${gameDir}`)
  onLog(`[Launcher] Version: ${versionId}`)

  // Always purge any leftover bejaclient mod JARs — BejaClient runs as a Java agent, not a mod
  removeBejaModJars(gameDir, onLog)

  // Stage adapter JAR into mods/ so Fabric Loader always picks it up on its classpath.
  // fabric.addMods is unreliable in Fabric Loader 0.18.6+; mods/ is always scanned.
  const modsDir = join(gameDir, 'mods')
  const adapterTempPath = join(modsDir, 'beja-adapter-loader.jar')
  if (existsSync(adapterTempPath)) {
    try { unlinkSync(adapterTempPath) } catch { /* ignore */ }
  }
  let adapterWasStaged = false
  if (profile.useBejaClient) {
    const adapterJar = resolveAdapterJar()
    if (adapterJar) {
      if (!existsSync(modsDir)) mkdirSync(modsDir, { recursive: true })
      copyFileSync(adapterJar, adapterTempPath)
      adapterWasStaged = true
      onLog(`[BejaClient] Staged adapter JAR → mods/beja-adapter-loader.jar`)
    } else {
      onLog('[BejaClient] Adapter JAR not found — BejaHooks calls may fail.')
    }
  }

  const proc = await launch({
    gamePath: gameDir,
    resourcePath: gameDir,
    javaPath,
    version: versionId,
    accessToken: account.accessToken,
    gameProfile: {
      id: account.uuid.replace(/-/g, ''),
      name: account.username,
    },
    minMemory: profile.minRam,
    maxMemory: profile.maxRam,
    extraJVMArgs: buildJvmArgs(profile),
    resolution: {
      width: profile.resolution.width,
      height: profile.resolution.height,
      fullscreen: false,
    },
  })

  activeProcess = proc
  onStatus('running')

  proc.stdout?.setEncoding('utf-8')
  proc.stderr?.setEncoding('utf-8')
  proc.stdout?.on('data', (data: string) => {
    data
      .split('\n')
      .filter(Boolean)
      .forEach(line => onLog(line))
  })
  proc.stderr?.on('data', (data: string) => {
    data
      .split('\n')
      .filter(Boolean)
      .forEach(line => onLog(`[ERR] ${line}`))
  })

  proc.on('exit', code => {
    if (adapterWasStaged && existsSync(adapterTempPath)) {
      try { unlinkSync(adapterTempPath) } catch { /* ignore */ }
    }
    activeProcess = null
    onStatus(`stopped:${code ?? 0}`)
  })
}

export function killGame(): void {
  if (activeProcess && !activeProcess.killed) {
    activeProcess.kill()
    activeProcess = null
  }
}
