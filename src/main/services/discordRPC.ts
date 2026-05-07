import { Client } from 'discord-rpc'

// Client ID extracted from the Discord application (not the bot token)
const CLIENT_ID = '1491721511168639016'

let rpc: Client | null = null
let ready = false
let startTimestamp: number = Date.now()

export async function initDiscordRPC(): Promise<void> {
  try {
    rpc = new Client({ transport: 'ipc' })

    rpc.on('ready', () => {
      ready = true
      startTimestamp = Date.now()
      setIdlePresence()
    })

    await rpc.login({ clientId: CLIENT_ID })
  } catch {
    // Discord not running or RPC unavailable — fail silently
    rpc = null
  }
}

export function setIdlePresence(): void {
  if (!rpc || !ready) return
  rpc.setActivity({
    details: 'Browsing the launcher',
    state: 'Idle',
    largeImageKey: 'logo',
    largeImageText: 'BejaClient',
    startTimestamp,
    instance: false,
  }).catch(() => {})
}

export function setPlayingPresence(version?: string): void {
  if (!rpc || !ready) return
  rpc.setActivity({
    details: version ? `Playing Minecraft ${version}` : 'Playing Minecraft',
    state: 'In Game',
    largeImageKey: 'logo',
    largeImageText: 'BejaClient',
    startTimestamp: Date.now(),
    instance: true,
  }).catch(() => {})
}

export function destroyDiscordRPC(): void {
  if (rpc) {
    rpc.destroy().catch(() => {})
    rpc = null
    ready = false
  }
}
