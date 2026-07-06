import { Client } from 'discord-rpc'
import { getSettings } from './settingsService'

// Default Client ID extracted from the Discord application (not the bot token)
const DEFAULT_CLIENT_ID = '1491721511168639016'

let rpc: Client | null = null
let ready = false
let startTimestamp: number = Date.now()

let currentPresenceType: 'idle' | 'playing' = 'idle'
let currentPlayingVersion: string | undefined = undefined
let currentPlayingProfileName: string | undefined = undefined
let activeClientId: string = ''

export async function initDiscordRPC(): Promise<void> {
  const settings = getSettings().discord
  const enabled = settings ? settings.enabled : true
  if (!enabled) return

  const clientId = (settings && settings.clientId.trim()) ? settings.clientId.trim() : DEFAULT_CLIENT_ID
  activeClientId = clientId

  try {
    rpc = new Client({ transport: 'ipc' })

    rpc.on('ready', () => {
      ready = true
      startTimestamp = Date.now()
      refreshPresence()
    })

    await rpc.login({ clientId })
  } catch {
    // Discord not running or RPC unavailable — fail silently
    rpc = null
    ready = false
  }
}

export async function updateDiscordRPCFromSettings(): Promise<void> {
  const settings = getSettings().discord

  if (!settings || !settings.enabled) {
    if (rpc) {
      destroyDiscordRPC()
    }
    return
  }

  const desiredClientId = settings.clientId.trim() || DEFAULT_CLIENT_ID

  // if rpc is running with a different client id then reconnect
  if (rpc && activeClientId !== desiredClientId) {
    destroyDiscordRPC()
  }

  if (!rpc) {
    activeClientId = desiredClientId
    await initDiscordRPC()
  } else if (ready) {
    refreshPresence()
  }
}

function refreshPresence(): void {
  if (currentPresenceType === 'playing') {
    setPlayingPresence(currentPlayingVersion, currentPlayingProfileName)
  } else {
    setIdlePresence()
  }
}

export function setIdlePresence(): void {
  currentPresenceType = 'idle'
  currentPlayingVersion = undefined
  currentPlayingProfileName = undefined

  if (!rpc || !ready) return

  const settings = getSettings().discord || {
    idleDetails: 'Browsing the launcher',
    idleState: 'Idle',
    idleLargeImageKey: 'logo',
    idleLargeImageText: 'BejaClient',
    idleSmallImageKey: '',
    idleSmallImageText: '',
  }

  const activity: any = {
    details: settings.idleDetails ? settings.idleDetails.trim() || undefined : undefined,
    state: settings.idleState ? settings.idleState.trim() || undefined : undefined,
    startTimestamp,
    instance: false,
  }

  if (settings.idleLargeImageKey && settings.idleLargeImageKey.trim()) {
    activity.largeImageKey = settings.idleLargeImageKey.trim()
    if (settings.idleLargeImageText && settings.idleLargeImageText.trim()) {
      activity.largeImageText = settings.idleLargeImageText.trim()
    }
  }

  if (settings.idleSmallImageKey && settings.idleSmallImageKey.trim()) {
    activity.smallImageKey = settings.idleSmallImageKey.trim()
    if (settings.idleSmallImageText && settings.idleSmallImageText.trim()) {
      activity.smallImageText = settings.idleSmallImageText.trim()
    }
  }

  rpc.setActivity(activity).catch(() => { })
}

export function setPlayingPresence(version?: string, profileName?: string): void {
  currentPresenceType = 'playing'
  currentPlayingVersion = version
  currentPlayingProfileName = profileName

  if (!rpc || !ready) return

  const settings = getSettings().discord || {
    playingDetails: 'Playing Minecraft {version}',
    playingState: 'In Game',
    playingLargeImageKey: 'logo',
    playingLargeImageText: 'BejaClient',
    playingSmallImageKey: '',
    playingSmallImageText: '',
  }

  const formatPlaceholder = (str: string) => {
    return str
      .replace(/{version}/g, version || 'Minecraft')
      .replace(/{profile}/g, profileName || '')
  }

  const activity: any = {
    details: settings.playingDetails ? formatPlaceholder(settings.playingDetails).trim() || undefined : undefined,
    state: settings.playingState ? formatPlaceholder(settings.playingState).trim() || undefined : undefined,
    startTimestamp: Date.now(),
    instance: true,
  }

  if (settings.playingLargeImageKey && settings.playingLargeImageKey.trim()) {
    activity.largeImageKey = settings.playingLargeImageKey.trim()
    if (settings.playingLargeImageText && settings.playingLargeImageText.trim()) {
      activity.largeImageText = formatPlaceholder(settings.playingLargeImageText).trim()
    }
  }

  if (settings.playingSmallImageKey && settings.playingSmallImageKey.trim()) {
    activity.smallImageKey = settings.playingSmallImageKey.trim()
    if (settings.playingSmallImageText && settings.playingSmallImageText.trim()) {
      activity.smallImageText = formatPlaceholder(settings.playingSmallImageText).trim()
    }
  }

  rpc.setActivity(activity).catch(() => { })
}

export function destroyDiscordRPC(): void {
  if (rpc) {
    rpc.destroy().catch(() => { })
    rpc = null
    ready = false
  }
}
