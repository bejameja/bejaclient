export interface Account {
  id: string
  username: string
  uuid: string
  accessToken: string
  refreshToken: string
  tokenExpiry: number
  skinUrl: string | null
  capeUrl: string | null
  skinModel: 'default' | 'slim'
  selected: boolean
  bejaToken?: string
}

export type LoaderType = 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge'

export interface LaunchProfile {
  id: string
  name: string
  description?: string
  version: string
  loader: LoaderType
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
  playtimeMs: number
  imageUrl?: string | null
  backgroundUrl?: string | null
  isolateProfile?: boolean
}

export interface RemoteVersion {
  id: string
  type: 'release' | 'snapshot' | 'old_beta' | 'old_alpha'
  url: string
  time: string
  releaseTime: string
}

export interface FabricLoaderVersion {
  loader: { version: string; stable: boolean }
  intermediary: { version: string }
  launcherMeta?: { version: number }
}

export interface VersionManifest {
  latest: { release: string; snapshot: string }
  versions: RemoteVersion[]
}

export interface ModInfo {
  id: string
  name: string
  fileName: string
  enabled: boolean
  filePath: string
  fileSize: number
  modifiedAt: string
  iconDataUrl?: string
}

export interface ModUpdateInfo {
  projectId: string
  versionId: string
  versionNumber: string
}

export interface GameSettings {
  defaultGameDir: string
  defaultJavaPath: string
  minRam: number
  maxRam: number
  jvmArgs: string
  resolution: { width: number; height: number }
  fullscreen: boolean
  extraGameArgs: string
}

export interface LauncherSettings {
  closeOnLaunch: boolean
  keepLauncherOpen: boolean
  autoUpdate: boolean
  concurrentDownloads: number
  soundEnabled: boolean
  soundVolume: number
  soundStyle: 'soft' | 'clicky'
  curseforgeApiKey: string
  advancedMode: boolean
  debugLogging: boolean
}

export interface AppearanceSettings {
  language: string
  accentColor: string
  reduceMotion: 'system' | 'on' | 'off'
  disableHoverEffects: boolean
  disableSplashScreen: boolean
  theme: 'default' | 'win95'
}

export interface ElementOverride {
  radius?: 'sharp' | 'default' | 'rounded'
  outline?: boolean
  outlineColor?: string
  outlineWidth?: number
  color?: string
  bgColor?: string
  fontFamily?: string
  fontSize?: number
  fontWeight?: 'normal' | 'medium' | 'bold'
  letterSpacing?: number
  opacity?: number
  blur?: number
  shadow?: 'none' | 'soft' | 'glow' | 'strong'
  hidden?: boolean
  bgVideo?: string
  bgImage?: string
  offsetX?: number
  offsetY?: number
  scale?: number
}

export interface CustomizationSettings {
  experimentalEnabled: boolean
  editorMode: boolean
  wiggleEnabled: boolean
  elements: Record<string, ElementOverride>
}

export interface AppSettings {
  game: GameSettings
  launcher: LauncherSettings
  appearance: AppearanceSettings
  customization: CustomizationSettings
  activeProfileId: string | null
}

export interface PlayerProfile {
  uuid: string
  username: string
  skinUrl: string | null
  capeUrl: string | null
  skinModel: 'default' | 'slim'
}

export interface BejaPlayerProfile {
  uuid: string
  username: string
  joinedAt: string | null
  xp: number
  bejaCapeUrl: string | null
  bejaCapeType: string | null
}

export type LaunchStatus = 'idle' | 'starting' | 'running' | 'stopping' | 'error'

export interface ChatMessage {
  id: number
  fromUuid: string
  fromUsername: string
  toUuid: string
  content: string
  sentAt: string
}

export interface PartyMember {
  uuid: string
  username: string
  skinUrl: string | null
  capeUrl: string | null
  skinModel: 'default' | 'slim'
  isLeader: boolean
  isReady: boolean
  isSpeaking: boolean
}

export interface Party {
  id: string
  members: PartyMember[]
  leaderId: string
}

export interface ServerStatus {
  id: string
  name: string
  host: string
  port: number
  featured: boolean
  online: boolean
  favicon: string | null
  version: string | null
  playersOnline: number
  playersMax: number
  motd: string | null
  ping: number | null
}

export type ModrinthProjectType = 'mod' | 'modpack' | 'resourcepack' | 'shader' | 'datapack'
export type ExploreSource = 'modrinth' | 'curseforge' | 'both'

export interface ExploreHit {
  id: string
  title: string
  description: string
  iconUrl: string | null
  downloads: number
  categories: string[]
  source: 'modrinth' | 'curseforge'
  projectType: string
  slug: string
  author: string | null
}

export interface DetectedProfile {
  id: string
  source: 'curseforge' | 'lunar' | 'modrinth_app'
  name: string
  version: string
  loader: string
  loaderVersion: string
  gameDir: string
  modCount: number
  iconPath: string | null
}

export interface ModDetails {
  id: string
  title: string
  author: string | null
  iconUrl: string | null
  downloads: number
  categories: string[]
  description: string
  descriptionFormat: 'markdown' | 'html'
  gallery: string[]
  sourceUrl: string
  license: string | null
  createdAt: string | null
  updatedAt: string | null
  source: 'modrinth' | 'curseforge'
}

export interface ModrinthHit {
  project_id: string
  slug: string
  title: string
  description: string
  categories: string[]
  project_type: ModrinthProjectType
  downloads: number
  icon_url: string | null
  latest_version: string
  versions: string[]       // returned by /search endpoint
  game_versions?: string[] // returned by /project endpoint (not in search results)
  loaders: string[]
}

export interface ModrinthVersion {
  id: string
  name: string
  version_number: string
  game_versions: string[]
  loaders: string[]
  files: { url: string; filename: string; primary: boolean; size: number }[]
}

export interface VersionProgress {
  task: string
  progress: number
  total: number
}

declare global {
  interface Window {
    api: {
      window: {
        minimize(): void
        maximize(): void
        close(): void
        isMaximized(): Promise<boolean>
        onMaximized(cb: (maximized: boolean) => void): void
      }
      auth: {
        login(): Promise<Account>
        logout(id: string): Promise<boolean>
        listAccounts(): Promise<Account[]>
        selectAccount(id: string): Promise<Account[]>
        refreshAccount(id: string): Promise<Account | null>
        importFromLauncher(): Promise<Account[]>
        onDeviceCode(cb: (data: { verificationUri: string; userCode: string; expiresIn: number }) => void): void
        onBrowserOpened(cb: () => void): void
        onAccountsUpdated(cb: (accounts: Account[]) => void): void
      }
      versions: {
        listRemote(): Promise<VersionManifest>
        listInstalled(): Promise<string[]>
        install(versionId: string, loaderType: string, loaderVersion?: string): Promise<string[]>
        delete(versionId: string): Promise<string[]>
        onProgress(cb: (data: VersionProgress) => void): void
        listFabricVersions(mcVersion: string): Promise<FabricLoaderVersion[]>
        listForgeVersions(mcVersion: string): Promise<string[]>
      }
      launch: {
        start(profileId: string): Promise<void>
        kill(): Promise<boolean>
        openConsole(): void
        onLog(cb: (line: string) => void): void
        onStatus(cb: (status: string) => void): void
      }
      console: {
        ready(): void
        onLog(cb: (line: string) => void): void
        onStatus(cb: (s: string) => void): void
        onClear(cb: () => void): void
      }
      profiles: {
        list(): Promise<LaunchProfile[]>
        create(profile: Omit<LaunchProfile, 'id' | 'createdAt' | 'lastPlayed' | 'playtimeMs'>): Promise<LaunchProfile>
        update(id: string, profile: Partial<LaunchProfile>): Promise<LaunchProfile | null>
        delete(id: string): Promise<boolean>
        getActive(): Promise<LaunchProfile | null>
        setActive(id: string): Promise<LaunchProfile | null>
        exportPack(id: string): Promise<{ ok: boolean; mods?: number; error?: string } | false>
        importPack(): Promise<{ profile: LaunchProfile; mods: string[] } | { error: string } | null>
        share(id: string): Promise<{ id: string; protocolUrl: string; webUrl: string } | { error: string }>
        peekShared(shareId: string): Promise<{
          ownerUsername: string
          profileName: string
          version?: string
          loader?: string
          modCount: number
        } | { error: string }>
        importShared(shareId: string): Promise<
          { profile: LaunchProfile; mods: string[]; ownerUsername: string } | { error: string }
        >
        onSharedLink(cb: (shareId: string) => void): void
        detectExternal(): Promise<DetectedProfile[]>
        importExternal(id: string): Promise<LaunchProfile>
      }
      mods: {
        list(profileId: string): Promise<ModInfo[]>
        install(profileId: string, filePath?: string): Promise<ModInfo[]>
        toggle(profileId: string, modId: string): Promise<ModInfo[]>
        delete(profileId: string, modId: string): Promise<ModInfo[]>
        openFolder(profileId: string): Promise<void>
        checkConflicts(profileId: string): Promise<string[]>
        autoFix(profileId: string): Promise<{ fixed: string[] }>
        checkUpdate(profileId: string, modId: string): Promise<ModUpdateInfo | null>
      }
      settings: {
        get(): Promise<AppSettings>
        set(settings: AppSettings): Promise<boolean>
        getGameDir(): Promise<string>
        setGameDir(dir: string): Promise<boolean>
        chooseJava(): Promise<string | null>
        chooseDir(): Promise<string | null>
      }
      modrinth: {
        search(query: string, type: ModrinthProjectType, gameVersion?: string, loader?: string, offset?: number, categories?: string[]): Promise<{ hits: ModrinthHit[]; total_hits: number }>
        categories(): Promise<{ name: string; project_type: string; header: string }[]>
        exploreSearch(query: string, type: string, source: string, gameVersion?: string, loader?: string, offset?: number, categories?: string[], sort?: string): Promise<{ hits: ExploreHit[]; total: number }>
        installCurseforge(modId: string, projectType: string, profileId: string): Promise<boolean>
        details(id: string, source: string): Promise<ModDetails>
        versions(projectId: string, gameVersion?: string, loader?: string): Promise<ModrinthVersion[]>
        installMod(projectId: string, profileId: string): Promise<boolean>
        installModpack(projectId: string, versionId: string | null): Promise<{ profileId: string; name: string }>
        installResourcePack(projectId: string, profileId: string): Promise<boolean>
        installShader(projectId: string, profileId: string): Promise<boolean>
        installDatapack(projectId: string, profileId: string): Promise<boolean>
        onProgress(cb: (msg: string) => void): void
        swapMod(profileId: string, oldModId: string, projectId: string, versionId: string): Promise<ModInfo[]>
      }
      system: {
        getVersion(): Promise<string>
        getJavaVersions(): Promise<string[]>
        platform: string
        openExternal(url: string): Promise<void>
      }
      players: {
        lookup(username: string): Promise<PlayerProfile | null>
        saveSkin(skinUrl: string, username: string): Promise<string>
        fetchImage(url: string): Promise<string>
        mcProfile(accessToken: string): Promise<{ id: string; name: string; capes: { id: string; state: string; url: string; alias: string }[] } | null>
        bejaProfile(uuid: string): Promise<BejaPlayerProfile | null>
        mcCreated(uuid: string): Promise<string | null>
        capes(uuid: string): Promise<{ service: string; capeUrl: string }[]>
        search(query: string): Promise<{ uuid: string; username: string; source: 'beja' | 'mojang' }[]>
        setSkin(accessToken: string, url: string, variant: 'classic' | 'slim'): Promise<unknown>
        setSkinFile(accessToken: string, base64Png: string, variant: 'classic' | 'slim'): Promise<unknown>
        setCape(accessToken: string, capeId: string): Promise<unknown>
        clearCape(accessToken: string): Promise<unknown>
      }
      friends: {
        connect(): Promise<boolean>
        disconnect(): Promise<void>
        list(): Promise<{
          uuid: string
          username: string
          status: string
          direction: string
          online: boolean
          countryCode?: string | null
          playing?: { game: string; version: string } | null
        }[]>
        sendRequest(username: string): Promise<{ ok?: boolean; error?: string }>
        acceptRequest(uuid: string): Promise<{ ok?: boolean; error?: string }>
        removeOrDecline(uuid: string): Promise<{ ok?: boolean; error?: string }>
        onOnline(cb: (d: { uuid: string; username: string; playing?: { game: string; version: string } | null }) => void): void
        onOffline(cb: (d: { uuid: string }) => void): void
        onRequest(cb: (d: { uuid: string; username: string }) => void): void
        onAccepted(cb: (d: { uuid: string; username: string }) => void): void
        onRemoved(cb: (d: { uuid: string }) => void): void
        onSocketStatus(cb: (status: 'connected' | 'disconnected' | 'error') => void): void
      }
      stats: {
        online(): Promise<number>
        onOnlineCount(cb: (count: number) => void): void
      }
      lobby: {
        emit(event: string, data: unknown): Promise<void>
        startWithServer(profileId: string, server: string, port: number): Promise<void>
        onPartyState(cb: (d: Party) => void): void
        onMemberJoined(cb: (d: PartyMember) => void): void
        onMemberLeft(cb: (d: { uuid: string }) => void): void
        onReadyUpdate(cb: (d: { uuid: string; isReady: boolean }) => void): void
        onSkinUpdate(cb: (d: { uuid: string; skinUrl: string | null; capeUrl: string | null; skinModel: 'default' | 'slim' }) => void): void
        onLaunched(cb: (d: { server: string; port: number; profileId: string }) => void): void
        onDisbanded(cb: () => void): void
        onError(cb: (d: { message?: string }) => void): void
        onSpeaking(cb: (d: { uuid: string; isSpeaking: boolean }) => void): void
        onVoiceOffer(cb: (d: { from: string; sdp: string }) => void): void
        onVoiceAnswer(cb: (d: { from: string; sdp: string }) => void): void
        onVoiceIce(cb: (d: { from: string; candidate: RTCIceCandidateInit }) => void): void
        onInviteReceived(cb: (d: { partyId: string; fromUuid: string; fromUsername: string }) => void): void
        onEmote(cb: (d: { uuid: string; emote: string }) => void): void
      }
      installs: {
        get(): Promise<{ mods: Record<string, string[]>; servers: Record<string, string[]> }>
      }
      servers: {
        list(): Promise<ServerStatus[]>
        ping(host: string, port: number): Promise<{ favicon: string | null; version: string | null; playersOnline: number; playersMax: number; motd: string | null; ping: number } | null>
        add(host: string, port: number, name: string): Promise<string>
        remove(id: string): Promise<boolean>
        addToProfile(host: string, port: number, name: string, favicon: string | null, profileId: string): Promise<boolean>
        onPingResult(cb: (data: { id: string; online: boolean; favicon: string | null; version: string | null; playersOnline: number; playersMax: number; motd: string | null; ping: number }) => void): void
      }
      chat: {
        send(toUuid: string, content: string): Promise<void>
        history(targetUuid: string): Promise<ChatMessage[]>
        sendTyping(toUuid: string): Promise<void>
        onMessage(cb: (msg: ChatMessage) => void): void
        onTyping(cb: (d: { fromUuid: string }) => void): void
      }
      giphy: {
        search(query: string): Promise<{ id: string; thumb: string; url: string; title: string }[]>
      }
      discord: {
        setPresence(details: string, state: string): Promise<void>
      }
      updater: {
        check(): Promise<void>
        download(): Promise<void>
        install(): void
        onChecking(cb: () => void): void
        onAvailable(cb: (info: { version: string; releaseNotes?: string }) => void): void
        onNotAvailable(cb: () => void): void
        onProgress(cb: (p: { percent: number; transferred: number; total: number; bytesPerSecond: number }) => void): void
        onDownloaded(cb: (info: { version: string }) => void): void
        onError(cb: (msg: string) => void): void
      }
      cosmetics: {
        get(uuid: string): Promise<{ cape_url?: string | null; cape_type?: string; equipped?: string[] } | null>
        update(data: { cape_url?: string | null; cape_type?: string; equipped?: string[] }): Promise<unknown>
      }
      capes: {
        list(offset?: number): Promise<{ capes: CommunityCape[]; total: number }>
        upload(base64Image: string, filename: string, name: string): Promise<{ ok: boolean; id?: number; url?: string; name?: string; error?: string }>
        report(id: number): Promise<{ ok?: boolean; error?: string }>
      }
      pass: {
        get(): Promise<unknown>
        progress(): Promise<unknown>
        daily(): Promise<unknown>
      }
      quests: {
        list(): Promise<{ week: string; quests: Quest[] }>
        progress(questId: string, amount: number): Promise<{ questId: string; progress: number; claimed: boolean } | null>
        claim(questId: string): Promise<{ awarded: boolean; xp_gained?: number; xp?: number; coins_gained?: number; balance?: number }>
        leaderboard(): Promise<{ entries: LeaderboardEntry[]; myRank: number | null }>
      }
      wallet: {
        getBalance(): Promise<{ balance: number }>
      }
      shop: {
        list(): Promise<{ items: ShopItem[]; owned: string[] }>
        purchase(itemId: string): Promise<{ purchased: boolean; reason?: string; item?: ShopItem; balance?: number }>
      }
      video: {
        getScene(): Promise<string>
        pickCustomBg(kind: 'video' | 'image'): Promise<string | null>
      }
    }
  }
}

export interface Quest {
  id: string
  name: string
  goal: number
  xp: number
  coins: number
  progress: number
  claimed: boolean
}

export interface CommunityCape {
  id: number
  name: string
  uploader_username: string
  filename: string
  created_at: string
  url: string
}

export interface ShopItem {
  id: string
  name: string
  price: number
  rarity: string
  type: string
  capeUrl?: string | null
}

export interface LeaderboardEntry {
  rank: number
  uuid: string
  username: string
  xp: number
}
