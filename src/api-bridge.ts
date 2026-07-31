// Tauri replacement for the old Electron `contextBridge.exposeInMainWorld('api', ...)` preload.
//
// Goal: keep the exact same `window.api.*` shape the Vue app already calls everywhere, so
// stores/components/pages need zero edits when migrating off Electron. Every method here is a
// thin wrapper: `invoke('rust_command_name', args)` for request/response calls, and
// `listen('event:name', cb)` for the push events the main process used to send.
//
// Rust-side command names are the snake_case equivalent of the old IPC channel
// (e.g. 'auth:login' -> `auth_login`). Event names are kept as-is (they're just string
// identifiers on both sides, not Rust symbols) so `emit("launch:log", line)` on the Rust side
// lines up with `listen('launch:log', ...)` here.

import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { platform as osPlatform } from '@tauri-apps/plugin-os'

const appWindow = getCurrentWindow()

// process.platform used to come from Electron's preload (Node global). @tauri-apps/plugin-os's
// platform() is synchronous, so translate it to the same 'win32'/'darwin'/... values Node used.
const osPlat = osPlatform()
const cachedPlatform: string = osPlat === 'windows' ? 'win32' : osPlat === 'macos' ? 'darwin' : osPlat

export const api = {
  window: {
    minimize: () => appWindow.minimize(),
    maximize: () => appWindow.toggleMaximize(),
    close: () => appWindow.close(),
    isMaximized: () => appWindow.isMaximized(),
    onMaximized: (cb: (maximized: boolean) => void) => {
      appWindow.onResized(async () => cb(await appWindow.isMaximized()))
    },
  },

  auth: {
    login: () => invoke('auth_login'),
    logout: (id: string) => invoke('auth_logout', { id }),
    listAccounts: () => invoke('auth_list_accounts'),
    selectAccount: (id: string) => invoke('auth_select_account', { id }),
    refreshAccount: (id: string) => invoke('auth_refresh', { id }),
    importFromLauncher: () => invoke('auth_import_launcher'),
    onDeviceCode: (cb: (data: { verificationUri: string; userCode: string; expiresIn: number }) => void) => {
      listen<{ verificationUri: string; userCode: string; expiresIn: number }>('auth:device-code', (e) => cb(e.payload))
    },
    onBrowserOpened: (cb: () => void) => {
      listen('auth:browser-opened', () => cb())
    },
    onAccountsUpdated: (cb: (accounts: unknown[]) => void) => {
      listen<unknown[]>('auth:accounts-updated', (e) => cb(e.payload))
    },
  },

  versions: {
    listRemote: () => invoke('versions_list_remote'),
    listInstalled: () => invoke('versions_list_installed'),
    install: (versionId: string, loaderType: string, loaderVersion?: string) =>
      invoke('versions_install', { versionId, loaderType, loaderVersion }),
    delete: (versionId: string) => invoke('versions_delete', { versionId }),
    onProgress: (cb: (data: { task: string; progress: number; total: number }) => void) => {
      listen<{ task: string; progress: number; total: number }>('versions:progress', (e) => cb(e.payload))
    },
    listFabricVersions: (mcVersion: string) => invoke('versions_list_fabric', { mcVersion }),
    listForgeVersions: (mcVersion: string) => invoke('versions_list_forge', { mcVersion }),
  },

  launch: {
    start: (profileId: string) => invoke('launch_start', { profileId }),
    kill: () => invoke('launch_kill'),
    openConsole: () => invoke('launch_open_console'),
    saveLogs: (lines: string[]) => invoke('launch_save_logs', { lines }),
    onLog: (cb: (line: string) => void) => {
      listen<string>('launch:log', (e) => cb(e.payload))
    },
    onStatus: (cb: (status: string) => void) => {
      listen<string>('launch:status', (e) => cb(e.payload))
    },
  },

  profiles: {
    list: () => invoke('profiles_list'),
    create: (profile: unknown) => invoke('profiles_create', { profile }),
    update: (id: string, profile: unknown) => invoke('profiles_update', { id, profile }),
    delete: (id: string) => invoke('profiles_delete', { id }),
    getActive: () => invoke('profiles_get_active'),
    setActive: (id: string) => invoke('profiles_set_active', { id }),
    exportPack: (id: string) => invoke('profiles_export', { id }),
    importPack: () => invoke('profiles_import'),
    share: (id: string) => invoke('profiles_share', { id }),
    importShared: (shareId: string) => invoke('profiles_import_shared', { shareId }),
    peekShared: (shareId: string) => invoke('profiles_peek_shared', { shareId }),
    onSharedLink: (cb: (shareId: string) => void) => {
      listen<string>('profiles:shared-link', (e) => cb(e.payload))
    },
  },

  mods: {
    list: (profileId: string) => invoke('mods_list', { profileId }),
    checkConflicts: (profileId: string) => invoke('mods_check_conflicts', { profileId }),
    install: (profileId: string, filePath: string) => invoke('mods_install', { profileId, filePath }),
    toggle: (profileId: string, modId: string) => invoke('mods_toggle', { profileId, modId }),
    delete: (profileId: string, modId: string) => invoke('mods_delete', { profileId, modId }),
    openFolder: (profileId: string) => invoke('mods_open_folder', { profileId }),
    autoFix: (profileId: string) => invoke('mods_auto_fix', { profileId }),
  },

  settings: {
    get: () => invoke('settings_get'),
    set: (settings: unknown) => invoke('settings_set', { settings }),
    getGameDir: () => invoke('settings_game_dir'),
    setGameDir: (dir: string) => invoke('settings_set_game_dir', { dir }),
    chooseJava: () => invoke('settings_choose_java'),
    chooseDir: () => invoke('settings_choose_dir'),
  },

  modrinth: {
    search: (query: string, type: string, gameVersion?: string, loader?: string, offset?: number, categories?: string[]) =>
      invoke('modrinth_search', { query, type, gameVersion, loader, offset, categories }),
    categories: () => invoke('modrinth_categories'),
    exploreSearch: (
      query: string,
      type: string,
      source: string,
      gameVersion?: string,
      loader?: string,
      offset?: number,
      categories?: string[],
    ) => invoke('explore_search', { query, type, source, gameVersion, loader, offset, categories }),
    installCurseforge: (modId: string, projectType: string, profileId: string) =>
      invoke('curseforge_install', { modId, projectType, profileId }),
    versions: (projectId: string, gameVersion?: string, loader?: string) =>
      invoke('modrinth_versions', { projectId, gameVersion, loader }),
    installMod: (projectId: string, profileId: string) => invoke('modrinth_install_mod', { projectId, profileId }),
    installModpack: (projectId: string, versionId: string | null) =>
      invoke('modrinth_install_modpack', { projectId, versionId }),
    installResourcePack: (projectId: string, profileId: string) =>
      invoke('modrinth_install_resourcepack', { projectId, profileId }),
    installShader: (projectId: string, profileId: string) => invoke('modrinth_install_shader', { projectId, profileId }),
    installDatapack: (projectId: string, profileId: string) =>
      invoke('modrinth_install_datapack', { projectId, profileId }),
    onProgress: (cb: (msg: string) => void) => {
      listen<string>('modrinth:progress', (e) => cb(e.payload))
    },
    swapMod: (profileId: string, oldModId: string, projectId: string, versionId: string) =>
      invoke('modrinth_swap_mod', { profileId, oldModId, projectId, versionId }),
  },

  system: {
    getVersion: () => invoke('system_get_version'),
    getJavaVersions: () => invoke('system_java_versions'),
    platform: cachedPlatform,
    openExternal: (url: string) => invoke('system_open_external', { url }),
  },

  console: {
    ready: () => invoke('console_ready'),
    onLog: (cb: (line: string) => void) => listen<string>('console:log', (e) => cb(e.payload)),
    onStatus: (cb: (s: string) => void) => listen<string>('console:status', (e) => cb(e.payload)),
    onClear: (cb: () => void) => listen('console:clear', () => cb()),
  },

  players: {
    lookup: (username: string) => invoke('players_lookup', { username }),
    saveSkin: (skinUrl: string, username: string) => invoke('players_save_skin', { skinUrl, username }),
    fetchImage: (url: string) => invoke('players_fetch_image', { url }),
    mcProfile: (accessToken: string) => invoke('players_mc_profile', { accessToken }),
    bejaProfile: (uuid: string) => invoke('players_beja_profile', { uuid }),
    mcCreated: (uuid: string) => invoke('players_mc_created', { uuid }),
    capes: (uuid: string) => invoke('players_capes', { uuid }),
    search: (query: string) => invoke('players_search', { query }),
    setSkin: (accessToken: string, url: string, variant: 'classic' | 'slim') =>
      invoke('players_set_skin', { accessToken, url, variant }),
    setSkinFile: (accessToken: string, base64Png: string, variant: 'classic' | 'slim') =>
      invoke('players_set_skin_file', { accessToken, base64Png, variant }),
    setCape: (accessToken: string, capeId: string) => invoke('players_set_cape', { accessToken, capeId }),
    clearCape: (accessToken: string) => invoke('players_clear_cape', { accessToken }),
  },

  friends: {
    connect: () => invoke('friends_connect'),
    disconnect: () => invoke('friends_disconnect'),
    list: () => invoke('friends_list'),
    sendRequest: (username: string) => invoke('friends_request', { username }),
    acceptRequest: (uuid: string) => invoke('friends_accept', { uuid }),
    removeOrDecline: (uuid: string) => invoke('friends_remove', { uuid }),
    onOnline: (cb: (d: { uuid: string; username: string; playing?: { game: string; version: string } | null }) => void) =>
      listen<{ uuid: string; username: string; playing?: { game: string; version: string } | null }>(
        'friend:online',
        (e) => cb(e.payload),
      ),
    onOffline: (cb: (d: { uuid: string }) => void) => listen<{ uuid: string }>('friend:offline', (e) => cb(e.payload)),
    onRequest: (cb: (d: { uuid: string; username: string }) => void) =>
      listen<{ uuid: string; username: string }>('friend:request', (e) => cb(e.payload)),
    onAccepted: (cb: (d: { uuid: string; username: string }) => void) =>
      listen<{ uuid: string; username: string }>('friend:accepted', (e) => cb(e.payload)),
    onRemoved: (cb: (d: { uuid: string }) => void) => listen<{ uuid: string }>('friend:removed', (e) => cb(e.payload)),
    onSocketStatus: (cb: (status: 'connected' | 'disconnected' | 'error') => void) =>
      listen<'connected' | 'disconnected' | 'error'>('socket:status', (e) => cb(e.payload)),
  },

  stats: {
    online: () => invoke('stats_online'),
    onOnlineCount: (cb: (count: number) => void) => listen<{ count: number }>('stats:online', (e) => cb(e.payload.count)),
  },

  lobby: {
    emit: (event: string, data: unknown) => invoke('lobby_emit', { event, data }),
    startWithServer: (profileId: string, server: string, port: number) =>
      invoke('launch_start_server', { profileId, server, port }),
    onPartyState: (cb: (d: unknown) => void) => listen('party:state', (e) => cb(e.payload)),
    onMemberJoined: (cb: (d: unknown) => void) => listen('party:member_joined', (e) => cb(e.payload)),
    onMemberLeft: (cb: (d: unknown) => void) => listen('party:member_left', (e) => cb(e.payload)),
    onReadyUpdate: (cb: (d: unknown) => void) => listen('party:ready_update', (e) => cb(e.payload)),
    onSkinUpdate: (cb: (d: unknown) => void) => listen('party:skin_update', (e) => cb(e.payload)),
    onLaunched: (cb: (d: unknown) => void) => listen('party:launched', (e) => cb(e.payload)),
    onDisbanded: (cb: () => void) => listen('party:disbanded', () => cb()),
    onError: (cb: (d: unknown) => void) => listen('party:error', (e) => cb(e.payload)),
    onSpeaking: (cb: (d: unknown) => void) => listen('voice:speaking', (e) => cb(e.payload)),
    onVoiceOffer: (cb: (d: unknown) => void) => listen('voice:offer', (e) => cb(e.payload)),
    onVoiceAnswer: (cb: (d: unknown) => void) => listen('voice:answer', (e) => cb(e.payload)),
    onVoiceIce: (cb: (d: unknown) => void) => listen('voice:ice', (e) => cb(e.payload)),
    onInviteReceived: (cb: (d: { partyId: string; fromUuid: string; fromUsername: string }) => void) =>
      listen<{ partyId: string; fromUuid: string; fromUsername: string }>('party:invite_received', (e) => cb(e.payload)),
    onEmote: (cb: (d: { uuid: string; emote: string }) => void) =>
      listen<{ uuid: string; emote: string }>('party:emote', (e) => cb(e.payload)),
  },

  cosmetics: {
    get: (uuid: string) => invoke('cosmetics_get', { uuid }),
    update: (data: { cape_url?: string | null; cape_type?: string; equipped?: string[] }) =>
      invoke('cosmetics_update', { data }),
    inventory: (uuid: string) => invoke('cosmetics_inventory', { uuid }),
  },

  crates: {
    list: () => invoke('crates_list'),
    open: (crateId: string) => invoke('crates_open', { crateId }),
    keys: () => invoke('crates_keys'),
  },

  crafting: {
    inventory: () => invoke('crafting_inventory'),
    combine: (rarity: string) => invoke('crafting_combine', { rarity }),
  },

  capes: {
    list: (offset?: number) => invoke('capes_list', { offset }),
    upload: (base64Image: string, filename: string, name: string) =>
      invoke('capes_upload', { base64Image, filename, name }),
    report: (id: number) => invoke('capes_report', { id }),
  },

  pass: {
    get: () => invoke('pass_get'),
    progress: () => invoke('pass_progress'),
    daily: () => invoke('pass_daily'),
  },

  quests: {
    list: () => invoke('quests_list'),
    progress: (questId: string, amount: number) => invoke('quests_progress', { questId, amount }),
    claim: (questId: string) => invoke('quests_claim', { questId }),
    leaderboard: () => invoke('quests_leaderboard'),
  },

  wallet: {
    getBalance: () => invoke('wallet_balance'),
  },
  shop: {
    list: () => invoke('shop_list'),
    purchase: (itemId: string) => invoke('shop_purchase', { itemId }),
  },

  installs: {
    get: () => invoke('installs_get'),
  },

  servers: {
    list: () => invoke('servers_list'),
    ping: (host: string, port: number) => invoke('servers_ping', { host, port }),
    add: (host: string, port: number, name: string) => invoke('servers_add', { host, port, name }),
    remove: (id: string) => invoke('servers_remove', { id }),
    addToProfile: (host: string, port: number, name: string, favicon: string | null, profileId: string) =>
      invoke('servers_add_to_profile', { host, port, name, favicon, profileId }),
    onPingResult: (cb: (data: unknown) => void) => listen('servers:ping-result', (e) => cb(e.payload)),
  },

  chat: {
    send: (toUuid: string, content: string) => invoke('chat_send', { toUuid, content }),
    history: (targetUuid: string) => invoke('chat_history', { targetUuid }),
    sendTyping: (toUuid: string) => invoke('chat_typing', { toUuid }),
    onMessage: (cb: (msg: unknown) => void) => listen('chat:message', (e) => cb(e.payload)),
    onTyping: (cb: (d: { fromUuid: string }) => void) => listen<{ fromUuid: string }>('chat:typing', (e) => cb(e.payload)),
  },

  video: {
    // Rust hands back an absolute filesystem path; convertFileSrc() turns it into the
    // asset://-backed URL the webview is actually allowed to load (see tauri.conf.json's
    // assetProtocol scope) — replaces the Electron build's little 127.0.0.1:49217 HTTP server.
    getScene: async () => convertFileSrc(await invoke<string>('video_get_scene')),
    pickCustomBg: async (kind: 'video' | 'image') => {
      const path = await invoke<string | null>('video_pick_custom_bg', { kind })
      return path ? convertFileSrc(path) : null
    },
  },

  updater: {
    check: () => invoke('updater_check'),
    download: () => invoke('updater_download'),
    install: () => invoke('updater_install'),
    onChecking: (cb: () => void) => listen('updater:checking', () => cb()),
    onAvailable: (cb: (info: { version: string; releaseNotes?: string }) => void) =>
      listen<{ version: string; releaseNotes?: string }>('updater:available', (e) => cb(e.payload)),
    onNotAvailable: (cb: () => void) => listen('updater:not-available', () => cb()),
    onProgress: (cb: (p: { percent: number; transferred: number; total: number; bytesPerSecond: number }) => void) =>
      listen<{ percent: number; transferred: number; total: number; bytesPerSecond: number }>('updater:progress', (e) =>
        cb(e.payload),
      ),
    onDownloaded: (cb: (info: { version: string }) => void) =>
      listen<{ version: string }>('updater:downloaded', (e) => cb(e.payload)),
    onError: (cb: (msg: string) => void) => listen<string>('updater:error', (e) => cb(e.payload)),
  },
}

// The canonical `Window.api` type is already declared in `src/types/index.ts` (carried over
// unchanged from the Electron preload's type surface) — don't redeclare it here, just assign.
;(window as unknown as { api: typeof api }).api = api
