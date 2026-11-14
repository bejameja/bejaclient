<template>
  <div class="home-page" :class="{ 'no-intro': !playIntro }">

    <!-- Main scrollable column -->
    <div class="main-col">

      <div class="hero-wrap">
        <div class="video-card">

          <!-- Campfire hub scene: everyone in the party sits around one fire in a shared 3D scene -->
          <CampfireScene
            ref="campfireRef"
            :members="campfireMembers"
            :local-uuid="account?.uuid ?? null"
            @invite="openInvite"
            @self-contextmenu="onSelfContextMenu"
          />

          <!-- Dev builds only: trigger scene events by hand (Ctrl + the number); Ctrl+H shows / hides this list -->
          <div v-if="isDev && showSceneHelp" class="dev-tools">
            <button v-for="(label, i) in SCENE_ACTION_LABELS" :key="label" class="dev-btn" @click="runSceneAction(i + 1)">
              <span class="dev-key">Ctrl+{{ i + 1 }}</span>{{ label }}
            </button>
          </div>

          <!-- Launch: white text in the bottom-left corner of the scene -->
          <div class="hero-launch-text">
            <LaunchButton text-only :ready-mode="!!lobbyStore.party && !lobbyStore.isLeader" />
          </div>

          <!-- Old launch button, dropped in on rope (hidden, kept for later) -->
          <div v-show="showLaunch" class="hero-launch">
            <div class="launch-drop" @animationend="onLaunchDropSettled">
              <div class="launch-ropes">
                <div class="rope rope--left"></div>
                <div class="rope rope--right"></div>
              </div>
              <LaunchButton :ready-mode="!!lobbyStore.party && !lobbyStore.isLeader" />
            </div>
          </div>

          <!-- Voice / party toolbar -->
          <div class="voice-controls">
            <template v-if="lobbyStore.party">
              <button
                class="voice-btn"
                :class="{ muted: voice.isMuted.value, active: voice.isSpeaking.value }"
                title="Toggle mic"
                @click="voice.toggleMic()"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M12 2a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"/>
                  <path d="M19 10v1a7 7 0 0 1-14 0v-1"/>
                  <line x1="12" y1="18" x2="12" y2="22"/>
                  <line v-if="voice.isMuted.value" x1="4" y1="4" x2="20" y2="20"/>
                </svg>
              </button>
              <button
                class="voice-btn"
                :class="{ muted: voice.isDeafened.value }"
                title="Toggle deafen"
                @click="voice.toggleDeafen()"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M3 14v-2a9 9 0 0 1 18 0v2"/>
                  <rect x="3" y="14" width="5" height="7" rx="1.5"/>
                  <rect x="16" y="14" width="5" height="7" rx="1.5"/>
                  <line v-if="voice.isDeafened.value" x1="4" y1="4" x2="20" y2="20"/>
                </svg>
              </button>
            </template>

            <template v-if="lobbyStore.party">
              <div class="party-id" @click="copyPartyId" title="Click to copy">
                {{ lobbyStore.party.id }}
                <button class="party-id-refresh" title="New code" @click.stop="lobbyStore.regenerateParty()">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 12a9 9 0 1 1-3-6.7"/>
                    <polyline points="21 3 21 9 15 9"/>
                  </svg>
                </button>
              </div>
              <button class="voice-btn delete-party-btn" title="Leave party" @click="lobbyStore.leaveParty()">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M9 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h4"/>
                  <polyline points="16 17 21 12 16 7"/>
                  <line x1="21" y1="12" x2="9" y2="12"/>
                </svg>
              </button>
            </template>
            <template v-else>
              <button class="voice-btn create-lobby-btn" title="Create party" @click="lobbyStore.createParty()">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <line x1="12" y1="5" x2="12" y2="19"/>
                  <line x1="5" y1="12" x2="19" y2="12"/>
                </svg>
              </button>
              <input
                v-if="joinInputActive"
                ref="joinInputEl"
                v-model="joinCodeInline"
                class="join-lobby-input"
                :class="{ 'join-lobby-input--busy': joiningInline }"
                placeholder="CODE"
                maxlength="6"
                :disabled="joiningInline"
                @blur="onJoinInputBlur"
                @keydown.esc="cancelJoinInline"
              />
              <button v-else class="voice-btn join-lobby-btn" @click="openJoin">
                <svg class="create-lobby-info" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
                  <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
                </svg>
                Join Party
              </button>
            </template>
          </div>
        </div>
      </div>

      <!-- Quick-join server row -->
      <div v-if="showQuickServers" class="quick-servers">
        <button
          v-for="s in quickServers"
          :key="s.id"
          class="quick-server-slot"
          :class="{ joining: joiningServerId === s.id }"
          @click="joinQuickServer(s)"
        >
          <img v-if="s.favicon" :src="s.favicon" :alt="s.name" />
          <span v-else class="quick-server-fallback">{{ s.name.charAt(0) }}</span>
        </button>
      </div>

      <!-- Quick-play server cards (hosts: see SERVER_CARDS) -->
      <div class="bottom-panel bottom-panel--servers">
        <button
          v-for="c in SERVER_CARDS"
          :key="c.id"
          class="server-card"
          :class="{ joining: joiningServerId === c.id }"
          :title="`Join ${c.name}`"
          @click="joinServerCard(c)"
        >
          <span class="server-card-bg" :style="{ backgroundImage: `url(${c.bg})` }"></span>
          <img class="server-card-logo" :src="c.logo" :alt="c.name" draggable="false" />
          <span class="server-card-label">{{ c.label }}</span>
        </button>
      </div>

    </div>

    <!-- Invite overlay -->
    <InviteOverlay :visible="inviteOpen" :initial-tab="inviteInitTab" @close="inviteOpen = false" />

    <!-- Own profile, opened via right-click on the local player -->
    <PlayerProfileModal v-model="selfProfileOpen" :player="selfProfilePlayer" />

  </div>
</template>

<script setup lang="ts">
import { computed, ref, onActivated, onDeactivated, onBeforeUnmount, onMounted, watch, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { useFriendsStore, type Friend } from '../store/friendsStore'
import { useAccountStore }  from '../store/accountStore'
import { useQuestsStore }   from '../store/questsStore'
import { levelFromXp }      from '../utils/level'
import { useLockerStore }   from '../store/lockerStore'
import { useLobbyStore }    from '../store/lobbyStore'
import { useLauncherStore } from '../store/launcherStore'
import type { PlayerProfile } from '../types'
import { useLobbyVoice }    from '../composables/useLobbyVoice'
import { showToast }        from '../composables/useToasts'
import { openContextMenu, type ContextMenuItem } from '../composables/useContextMenu'
import InviteOverlay  from '../components/lobby/InviteOverlay.vue'
import CampfireScene  from '../components/campfire/CampfireScene.vue'
import { SCENE_ACTION_LABELS, type SceneMember } from '../components/campfire/campfireConfig'
import PlayerProfileModal from '../components/friends/PlayerProfileModal.vue'
import LaunchButton   from '../components/home/LaunchButton.vue'
import logoHoplite    from '../assets/servers/hoplite.png'
import logoHypixel    from '../assets/servers/hypixel.png'
import logoWynncraft  from '../assets/servers/wynncraft.png'
import logoGomme      from '../assets/servers/gommehd.png'
import bgHoplite      from '../assets/servers/bg/hoplite.jpg'
import bgHypixel      from '../assets/servers/bg/hypixel.jpg'
import bgWynncraft    from '../assets/servers/bg/wynncraft.jpg'
import bgGomme        from '../assets/servers/bg/gommehd.jpg'
import { consumeHubIntro } from './homePageIntro'
// ── Video ─────────────────────────────────────────────────────────────────────

// Module-level (see homePageIntro.ts): true only the first time this app
// session reaches the hub, false on every later tab switch back to it -
// even if KeepAlive's LRU cache evicts and remounts the component. Set from
// onActivated (not here in setup) - <KeepAlive> only calls setup() once for
// the component's whole lifetime, but calls onActivated every time the tab
// is revisited (including the very first mount), so reading it here would
// freeze `true` for as long as the instance stays cached.
const playIntro = ref(false)

// ── News / status panel ──────────────────────────────────────────────────────
const onlineCount = ref<number | null>(null)
const ICON_RELAY = `<svg viewBox="0 0 20 20" fill="none"><circle cx="6" cy="14" r="2.4" stroke="currentColor" stroke-width="1.4"/><circle cx="14" cy="6" r="2.4" stroke="currentColor" stroke-width="1.4"/><path d="M7.7 12.3 12.3 7.7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>`
const ICON_MASK  = `<svg viewBox="0 0 20 20" fill="none"><circle cx="10" cy="10" r="7" stroke="currentColor" stroke-width="1.4"/><path d="M7 9.5c.4-1 1.2-1.5 3-1.5s2.6.5 3 1.5M7.5 12.5c.7.8 1.6 1.2 2.5 1.2s1.8-.4 2.5-1.2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>`
const ICON_BOLT  = `<svg viewBox="0 0 20 20" fill="none"><path d="M11 2 4 11h5l-1 7 7-9h-5l1-7Z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/></svg>`
const ICON_BELL  = `<svg viewBox="0 0 20 20" fill="none"><path d="M10 3.5c-2.3 0-3.6 1.7-3.6 4v2.3L5 12.5h10L13.6 9.8V7.5c0-2.3-1.3-4-3.6-4Z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/><path d="M8.3 14.8c.3.8.9 1.2 1.7 1.2s1.4-.4 1.7-1.2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>`
const ICON_USERS = `<svg viewBox="0 0 20 20" fill="none"><circle cx="7" cy="7" r="2.4" stroke="currentColor" stroke-width="1.4"/><path d="M2.5 16c.4-2.6 2.1-4 4.5-4s4.1 1.4 4.5 4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/><circle cx="14" cy="7.5" r="2" stroke="currentColor" stroke-width="1.4"/><path d="M12.8 12.3c1.9.3 3.1 1.6 3.4 3.7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>`
// Kept honest - real shipped features, not filler copy.
const newsItems = [
  { icon: ICON_RELAY, title: 'World Relay',       desc: 'Host your singleplayer world for friends over the internet - no port forwarding, your IP never shown.' },
  { icon: ICON_MASK,  title: 'Cosmetics',          desc: 'Capes you own show up right here on your character.' },
  { icon: ICON_BOLT,  title: 'Boosted Performance',desc: 'Sodium, Lithium & FerriteCore are bundled into every launch automatically.' },
]

const bottomTab = ref<'news' | 'friends'>('news')

// ── Quick-join server row ────────────────────────────────────────────────────
// Same real host list ServerRow.vue uses (kept in sync with serverPingService.ts's
// FEATURED_SERVERS) - just a subset sized for this row instead of that component's
// 20-slot flip-card corner widget.
interface QuickServer { id: string; name: string; host: string; port: number; favicon: string | null }
const QUICK_SERVERS: QuickServer[] = [
  { id: 'hypixel',      name: 'Hypixel',      host: 'mc.hypixel.net',        port: 25565, favicon: null },
  { id: '2b2t',         name: '2b2t',         host: 'connect.2b2t.org',      port: 25565, favicon: null },
  { id: 'wynncraft',    name: 'Wynncraft',    host: 'play.wynncraft.com',    port: 25565, favicon: null },
  { id: 'cubecraft',    name: 'CubeCraft',    host: 'play.cubecraft.net',    port: 25565, favicon: null },
  { id: 'gommehd',      name: 'GommeHD',      host: 'gommehd.net',           port: 25565, favicon: null },
  { id: 'griefergames', name: 'GrieferGames', host: 'griefergames.net',      port: 25565, favicon: null },
  { id: 'pikanetwork',  name: 'PikaNetwork',  host: 'play.pika-network.net', port: 25565, favicon: null },
  { id: 'donutsmp',     name: 'DonutSMP',     host: 'donutsmp.net',          port: 25565, favicon: null },
]
// Quick-play cards under the scene: logo = the server's own icon, bg = key art (src/assets/servers). All four hosts were
// checked online (MOTD matched). Order = card order.
const SERVER_CARDS: Array<{ id: string; name: string; label: string; host: string; port: number; logo: string; bg: string }> = [
  { id: 'hoplite',   name: 'Hoplite',   label: 'HOPLITE SERVER',  host: 'play.hoplite.gg',    port: 25565, logo: logoHoplite,   bg: bgHoplite },
  { id: 'hypixel',   name: 'Hypixel',   label: 'HYPIXEL',         host: 'mc.hypixel.net',     port: 25565, logo: logoHypixel,   bg: bgHypixel },
  { id: 'wynncraft', name: 'Wynncraft', label: 'WYNNCRAFT',       host: 'play.wynncraft.com', port: 25565, logo: logoWynncraft, bg: bgWynncraft },
  { id: 'gommehd',   name: 'GommeHD',   label: 'GOMME HD SERVER', host: 'gommehd.net',        port: 25565, logo: logoGomme,     bg: bgGomme },
]

function joinServerCard(c: (typeof SERVER_CARDS)[number]): void {
  void joinQuickServer({ id: c.id, name: c.name, host: c.host, port: c.port, favicon: null })
}

const quickServers = ref<QuickServer[]>(QUICK_SERVERS.map(s => ({ ...s })))
const joiningServerId = ref<string | null>(null)

async function joinQuickServer(s: QuickServer) {
  if (joiningServerId.value) return
  const profile = launcherStore.activeProfile
  if (!profile) {
    showToast({ title: 'No profile selected', body: 'Pick a profile before joining a server.', variant: 'error', duration: 3500 })
    return
  }
  joiningServerId.value = s.id
  try {
    await window.api.lobby.startWithServer(profile.id, s.host, s.port)
  } catch (e) {
    showToast({ title: 'Failed to join', body: e instanceof Error ? e.message : String(e), variant: 'error', duration: 4000 })
  } finally {
    joiningServerId.value = null
  }
}

const sceneVideo = ref('')
const sceneImage = ref('')
const videoRef   = ref<HTMLVideoElement | null>(null)

// ── Stores / composables ──────────────────────────────────────────────────────

const friendsStore  = useFriendsStore()
const accountStore  = useAccountStore()
const questsStore   = useQuestsStore()
const lobbyStore    = useLobbyStore()

async function copyPartyId() {
  const id = lobbyStore.party?.id
  if (!id) return
  try {
    await navigator.clipboard.writeText(id)
    showToast({ title: 'Party code copied', body: id, variant: 'info', duration: 1800 })
  } catch (e) {
    showToast({ title: 'Could not copy', body: e instanceof Error ? e.message : String(e), variant: 'error', duration: 3000 })
  }
}
const launcherStore = useLauncherStore()
const voice         = useLobbyVoice()


// ── Campfire hub scene ───────────────────────────────────────────────────────
// Everyone in the party sits around a campfire in one shared 3D scene (the hub's centrepiece).
const campfireRef = ref<InstanceType<typeof CampfireScene> | null>(null)
const isDev = import.meta.env.DEV
const showSceneHelp = ref(false)   // Ctrl+H toggles the list of test actions
const showLaunch = ref(false)    // launch button removed from the hub for now (dev: Ctrl+L shows it again)
const showQuickServers = ref(false)   // quick-join server row hidden for now (set true to bring it back)

// Test actions (dev builds): the buttons in the corner of the hub card, or Ctrl + 1..9 while the hub is open.
function runSceneAction(n: number): void {
  const body = campfireRef.value?.runAction(n) ?? 'The scene is not ready yet.'
  showToast({ title: `${n} \u00b7 ${SCENE_ACTION_LABELS[n - 1]}`, body, variant: 'info', duration: 2200 })
}

function onSceneActionKey(e: KeyboardEvent): void {
  if (!isDev || !e.ctrlKey || e.shiftKey || e.altKey || e.metaKey) return
  if (e.key.toLowerCase() === 'h') { e.preventDefault(); showSceneHelp.value = !showSceneHelp.value; return }
  if (e.key.toLowerCase() === 'l') { e.preventDefault(); showLaunch.value = !showLaunch.value; return }
  if (!/^[1-9]$/.test(e.key)) return
  const n = Number(e.key)
  if (n > SCENE_ACTION_LABELS.length) return
  e.preventDefault()
  runSceneAction(n)
}

onActivated(() => window.addEventListener('keydown', onSceneActionKey))
onDeactivated(() => window.removeEventListener('keydown', onSceneActionKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onSceneActionKey))   // also on unmount/HMR, or stale listeners stack up

const lockerStore = useLockerStore()
const account     = computed(() => accountStore.selectedAccount)

const activeSkinUrl   = computed(() => lockerStore.skinUrl  ?? account.value?.skinUrl  ?? null)
const activeCapeUrl   = computed(() => lockerStore.capeUrl  ?? account.value?.capeUrl  ?? null)
const activeSkinModel = computed(() => lockerStore.model    ?? account.value?.skinModel ?? 'default')

// ── Own profile (right-click on the local player) ───────────────────────────
const selfProfileOpen = ref(false)

const selfProfilePlayer = computed<PlayerProfile | null>(() => {
  if (!account.value) return null
  return {
    uuid: account.value.uuid,
    username: account.value.username,
    skinUrl: activeSkinUrl.value,
    capeUrl: activeCapeUrl.value,
    skinModel: activeSkinModel.value,
  }
})

function onSelfContextMenu(event: MouseEvent): void {
  if (!account.value) return
  const username = account.value.username
  const items: ContextMenuItem[] = [
    { label: 'View Profile', icon: 'eye', onClick: () => { selfProfileOpen.value = true } },
    { label: 'Copy Username', icon: 'copy', onClick: () => navigator.clipboard.writeText(username) },
  ]
  openContextMenu(event, items)
}

const campfireMembers = computed<SceneMember[]>(() => {
  const acc = account.value
  if (!acc) return []
  const self: SceneMember = {
    uuid: acc.uuid, username: acc.username, skinUrl: activeSkinUrl.value, capeUrl: activeCapeUrl.value,
    model: activeSkinModel.value, isLeader: lobbyStore.isLeader,
    level: levelFromXp(questsStore.totalXp).level, levelProgress: levelFromXp(questsStore.totalXp).progress,
  }
  const others = (lobbyStore.party?.members ?? [])
    .filter(m => m.uuid !== acc.uuid)
    .map<SceneMember>(m => ({
      uuid: m.uuid, username: m.username, skinUrl: m.skinUrl, capeUrl: m.capeUrl,
      model: m.skinModel, isLeader: m.uuid === lobbyStore.party?.leaderId,
    }))
  return [self, ...others]
})

const friends    = computed(() => friendsStore.friends)

const router = useRouter()

function countryFlag(countryCode: string): string {
  return String.fromCodePoint(...[...countryCode.toUpperCase()].map(c => 127397 + c.charCodeAt(0)))
}

function openFriendChat(friend?: Friend) {
  const target = friend ?? friends.value.find(f => f.online) ?? friends.value[0]
  if (!target) return
  router.push({ name: 'friends', query: { chatWith: target.uuid } })
}

// An element with an active CSS `animation` keeps creating its own stacking
// context for as long as `animation-name` isn't `none` - even once the
// one-time launch-drop-y entrance has finished and is just sitting at its
// `both`-filled end state. That nested stacking context is what was blocking
// LaunchButton's backdrop-filter (a descendant) from ever compositing: it
// works fine elsewhere in the app (e.g. the account dropdown, teleported
// straight to <body> with no such ancestor), just not behind this animation.
// Dropping the animation entirely once it's done removes the context.
function onLaunchDropSettled(e: AnimationEvent) {
  if (e.animationName === 'launch-drop-y') (e.currentTarget as HTMLElement).style.animation = 'none'
}

// ── Invite overlay ────────────────────────────────────────────────────────────

const inviteOpen    = ref(false)
const inviteInitTab = ref<'invite' | 'join'>('invite')

async function openInvite() {
  // Clicking an empty slot to invite someone creates the lobby on demand,
  // rather than one existing silently for every user from page load.
  if (!lobbyStore.party) await lobbyStore.createParty()
  inviteInitTab.value = 'invite'
  inviteOpen.value = true
}

// ── Inline join-by-code (the "Join Party" button morphs into this) ────────────

const joinInputActive = ref(false)
const joinCodeInline   = ref('')
const joiningInline    = ref(false)
const joinInputEl      = ref<HTMLInputElement | null>(null)

function openJoin(): void {
  joinCodeInline.value  = ''
  joinInputActive.value = true
  nextTick(() => joinInputEl.value?.focus())
}

function cancelJoinInline(): void {
  joinInputActive.value = false
  joinCodeInline.value  = ''
}

// blur cancels only when empty - mid-code, a stray focus loss (e.g. alt-tab)
// shouldn't silently discard what they typed; keydown.esc is the explicit cancel.
function onJoinInputBlur(): void {
  if (!joinCodeInline.value) cancelJoinInline()
}

async function submitJoinInline(): Promise<void> {
  if (joinCodeInline.value.length < 6 || joiningInline.value) return
  joiningInline.value = true
  const result = await lobbyStore.joinParty(joinCodeInline.value)
  joiningInline.value = false
  if (result.ok) {
    cancelJoinInline()
  } else {
    showToast({ title: 'Failed to join party', body: result.error, variant: 'error' })
    joinCodeInline.value = ''
    joinInputEl.value?.focus()
  }
}

// Auto-submit the moment a full 6-character code is typed - no separate
// confirm button needed for a fixed-length code.
watch(joinCodeInline, v => { if (v.length === 6) submitJoinInline() })

// ── Voice: wire IPC events → composable ──────────────────────────────────────

function initVoiceIpc(): void {
  window.api.lobby.onVoiceOffer(d  => voice.handleOffer(d  as any))
  window.api.lobby.onVoiceAnswer(d => voice.handleAnswer(d as any))
  window.api.lobby.onVoiceIce(d   => voice.handleIce(d    as any))
}

// When a new member joins, initiate a call to them (leader side)
watch(() => lobbyStore.party?.members.length, (next, prev) => {
  if (!next || !prev || next <= prev) return
  const newMember = lobbyStore.party?.members[next - 1]
  if (newMember && newMember.uuid !== accountStore.selectedAccount?.uuid) {
    voice.initiateCall(newMember.uuid)
  }
})

// ── Lifecycle ─────────────────────────────────────────────────────────────────

onMounted(async () => {
  questsStore.loadLeaderboard().catch(() => {})   // fills totalXp -> level badge in the campfire nametag
  // Hero background - served straight from the Pterodactyl backend's /uploads (same host
  // client_update_service.rs pulls the beja jars from, not the api.bejaclient.xyz Cloudflare
  // proxy), not bundled with the app. Swap classic-style: replace the file on the server (Panel
  // → Files → uploads/) and flip which of these two lines is active.
  sceneImage.value = 'http://206.217.141.184:3093/uploads/scene.png'
  // sceneVideo.value = 'http://206.217.141.184:3093/uploads/scene.mp4'

  onlineCount.value = await window.api.stats.online()
  window.api.stats.onOnlineCount((count: number) => { onlineCount.value = count })

  QUICK_SERVERS.forEach(async (s, i) => {
    try {
      const status = await window.api.servers.ping(s.host, s.port)
      if (status?.favicon) quickServers.value[i] = { ...quickServers.value[i], favicon: status.favicon }
    } catch { /* stays with the letter fallback */ }
  })

  // Init voice capture
  await voice.init()
  initVoiceIpc()
})

onActivated(() => {
  playIntro.value = consumeHubIntro()
  videoRef.value?.play()
})
// Kept alive by App.vue's <KeepAlive> - pause the decode/GPU work while
// the user is on another tab instead of burning it in the background.
onDeactivated(() => { videoRef.value?.pause() })

function onVideoError(e: Event) {
  const v = e.target as HTMLVideoElement
  console.error('[video] error code:', v.error?.code, 'src:', v.currentSrc)
}
</script>

<style lang="scss" scoped>
@import '../styles/motion';

.home-page {
  height: 100%;
  display: flex;
  flex-direction: row;
  overflow: hidden;
}

// ── Hub entrance - plays once on mount (KeepAlive skips it on tab revisits) ───
//
// No `filter` here (unlike hub-fade-blur/hub-slide-in below) - .video-card is
// what LaunchButton's backdrop-filter renders against (the launch button sits
// absolutely-positioned on top of it), and an active `filter` on an element
// blocks a sibling's `backdrop-filter` from compositing at all in Chromium/
// WebView2 until that filter animation settles. That's why the launch button
// had no blur for the first ~1.2s after mount.
@keyframes hub-rise {
  from { opacity: 0; transform: translateY(64px) scale(0.93); }
  to   { opacity: 1; transform: translateY(0) scale(1); }
}

@keyframes hub-fade {
  from { opacity: 0; }
  to   { opacity: 1; }
}

// Same visibility-based swap as launch-drop-y - .voice-controls now has
// backdrop-filter children (voice-btn/party-id), and an opacity-animating
// ancestor blocks those from compositing until the animation ends, same
// bug as LaunchButton had.
@keyframes voice-controls-in {
  from { visibility: hidden; }
  to   { visibility: visible; }
}

@keyframes hub-fade-blur {
  from { opacity: 0; filter: blur(10px); }
  to   { opacity: 1; filter: blur(0); }
}

@keyframes hub-slide-in {
  from { opacity: 0; transform: translateX(64px); filter: blur(6px); }
  to   { opacity: 1; transform: translateX(0); filter: blur(0); }
}

@keyframes hub-row-slide-in {
  from { opacity: 0; transform: translateX(36px); }
  to   { opacity: 1; transform: translateX(0); }
}

// margin-top + visibility, not transform + opacity: animating transform or
// opacity on this ancestor keeps creating its own stacking context for the
// whole animation (delay included), which blocks LaunchButton's
// backdrop-filter (a descendant) from compositing at all until the
// animation ends. margin-top/visibility give the same "hidden, then drops
// into place" effect without ever creating one.
@keyframes launch-drop-y {
  0%   { visibility: hidden; margin-top: -170px; }
  8%   { visibility: visible; }
  100% { margin-top: 0; }
}

@keyframes launch-rope-extend {
  0%   { height: 2px; opacity: 0; }
  8%   { opacity: 1; }
  100% { height: 170px; }
}

// Tab revisit (not a real app launch) - skip the one-time entrance
// animations.
.home-page.no-intro {
  .launch-drop, .rope, .video-card, .voice-controls,
  .quick-servers, .bottom-panel, .friend-row {
    animation: none;
  }
}

// ── Main column ───────────────────────────────────────────────────────────────
.main-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0 14px 14px;
  gap: 14px;
}

// ── Hero section ──────────────────────────────────────────────────────────────
.hero-wrap {
  position: relative;
  flex-shrink: 0;
}

.hero-launch-text {
  position: absolute;
  left: 22px;
  bottom: 46px;
  z-index: 6;
}

.hero-launch {
  position: absolute;
  bottom: 50px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10;
  display: flex;
  align-items: center;
}

.launch-drop {
  display: flex;
  align-items: center;
  gap: 12px;
  animation: launch-drop-y 1100ms cubic-bezier(0.16, 1, 0.3, 1) 980ms both;
}

.launch-ropes {
  position: absolute;
  top: -170px;
  left: 50%;
  width: 0;
  height: 0;
  pointer-events: none;
}

.rope {
  position: absolute;
  top: 0;
  width: 4px;
  height: 2px;
  opacity: 0;
  // hemp rope: tan, with a short dark twist every few pixels
  background: repeating-linear-gradient(to bottom, #b08a55 0 4px, #6e5230 4px 6px);
  transform-origin: top center;
  animation: launch-rope-extend 1100ms cubic-bezier(0.16, 1, 0.3, 1) 980ms both;

  &--left { left: -109px; }
  &--right { left: 105px; }
}

// ── Video card ────────────────────────────────────────────────────────────────
.video-card {
  width: 100%;
  height: 74vh;
  // the scene sits on top of the news panel: rounded on top, square at the bottom where the two meet
  border-radius: var(--edr-radius, #{$radius-lg}) var(--edr-radius, #{$radius-lg}) 0 0;
  overflow: hidden;
  clip-path: inset(0 round var(--edr-radius, #{$radius-lg}) var(--edr-radius, #{$radius-lg}) 0 0);
  flex-shrink: 0;
  position: relative;
  background: #13141A;
  animation: hub-rise 1200ms cubic-bezier(0.16, 1, 0.3, 1) both;
}

.scene-video {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  transform: scale(1.15);
  background: #111;
}

// Dev-only test buttons in the top-left corner of the hub card.
.dev-tools {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 12;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
}

.dev-btn {
  padding: 5px 10px;
  border-radius: 0;   // the global button reset rounds corners; the hub scene overlays stay square
  background: rgba(10, 12, 18, 0.7);
  border: 1px dashed rgba(255, 255, 255, 0.3);
  color: rgba(255, 255, 255, 0.75);
  font-size: 11px;
  letter-spacing: 0.04em;
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease, border-color 150ms ease;

  &:hover {
    background: rgba(255, 255, 255, 0.14);
    border-color: rgba(255, 255, 255, 0.7);
    color: #fff;
  }
}

.dev-key {
  display: inline-block;
  min-width: 42px;
  margin-right: 8px;
  color: rgba(255, 255, 255, 0.45);
  font-family: 'IBM Plex Mono', monospace;
}

// ── Voice controls ────────────────────────────────────────────────────────────

.voice-controls {
  position: absolute;
  bottom: 14px;
  right: 14px;
  display: flex;
  align-items: center;
  gap: 6px;
  z-index: 10;
  animation: voice-controls-in 1ms linear 1100ms both;

  // Invisible hit-area, 12px larger on every side than the visible row -
  // a separate layer (via inset) so it can't affect the row's own bottom/
  // right position above. Without it, a click landing just past a button's
  // own edge but still inside this row's visual footprint falls through to
  // whatever sits underneath - namely the empty right-flank invite button
  // (2xl size, 210x320px), which reaches down far enough to overlap this
  // corner when there's no party.
  &::before {
    content: '';
    position: absolute;
    inset: -12px;
  }

  // thin divider between the voice controls and the party controls

}

.voice-btn {
  position: relative;
  width: 38px;
  height: 38px;
  border-radius: 12px;
  border: none;
  background: #13141A;
  box-shadow: none;
  color: rgba(255, 255, 255, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 120ms, color 120ms, transform 100ms;

  svg { width: 18px; height: 18px; }

  &:hover { background: #232328; color: #fff; }
  &:active { transform: scale(0.95); }

  // speaking: lit pill
  &.active { color: #fff; background: #2a2a30; }
  // muted / deafened: red icon, red hairline
  &.muted { color: #ff6b61; background: rgba(255, 69, 58, 0.14); }
}

.party-id {
  display: flex;
  align-items: center;
  height: 38px;
  gap: 6px;
  font-family: 'IBM Plex Mono', monospace;
  font-size: 12px;
  letter-spacing: 0.16em;
  color: rgba(255, 255, 255, 0.8);
  background: #13141A;
  padding: 0 4px 0 12px;
  border-radius: 12px;
  box-shadow: none;
  cursor: pointer;
  user-select: all;
  transition: background 120ms;

  &:hover { background: #232328; }
}

.delete-party-btn {
  &:hover { color: #ff6b61; background: rgba(255, 69, 58, 0.14); }
}

// Icon-only - inherits its square 34x34 shape straight from .voice-btn.
.create-lobby-btn {
  &:disabled { opacity: 0.5; cursor: default; filter: brightness(1); }
}

// Fixed width (not auto) - .join-lobby-input below matches it exactly so
// swapping button↔input on click doesn't resize/jump in the toolbar.
.join-lobby-btn {
  position: relative;
  width: auto;
  min-width: 108px;
  height: 38px;
  justify-content: center;
  border-radius: 12px;
  padding: 0 14px;
  gap: 8px;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  outline: none;
}

// Replaces .join-lobby-btn in place while joinInputActive - same footprint
// (34px tall, glass style) so the toolbar row doesn't jump.
.join-lobby-input {
  height: 38px;
  width: 108px;
  border-radius: 12px;
  padding: 0 10px;
  background: #13141A;
  border: none;
  box-shadow: none;
  font-family: 'IBM Plex Mono', monospace;
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.25em;
  text-align: center;
  text-transform: uppercase;
  color: #fff;
  outline: none;
  transition: box-shadow 120ms, opacity 200ms;

  &::placeholder { color: rgba(255, 255, 255, 0.2); letter-spacing: 0.2em; }
  &:focus { box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.4); }
  &--busy { opacity: 0.5; }
}

.create-lobby-info {
  margin-left: -2px;
}

.party-id-refresh {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 5px;
  border: none;
  background: none;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  border-radius: 8px;
  user-select: none;
  transition: color 120ms, background 120ms;

  &:hover:not(:disabled) { color: #fff; background: rgba(255, 255, 255, 0.1); }
  &:disabled { opacity: 0.4; cursor: default; }
}

$friends-embed-bg: #13141A;

// ── Quick-join server row ────────────────────────────────────────────────────
.quick-servers {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-evenly;   // spread across the whole line
  gap: 22px;
  padding: 2px 0;
  animation: hub-fade 900ms ease 1000ms both;
}

.quick-server-slot {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  padding: 0;
  border: none;
  border-radius: $radius-sm;
  background: transparent;
  cursor: pointer;
  overflow: hidden;
  transition: transform 160ms cubic-bezier(0.34, 1.56, 0.64, 1), opacity 160ms ease;

  &:hover { transform: scale(1.12); }
  &:active { transform: scale(0.94); }
  &.joining { opacity: 0.4; pointer-events: none; }

  img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    image-rendering: pixelated;
  }
}

.quick-server-fallback {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: $radius-sm;
  background: $surface-elevated;
  font-size: 12px;
  font-weight: 700;
  color: $text-secondary;
}

// ── Bottom panel (News / Friends tabs) - replaces both the old empty-embed
// placeholder and the separate right-side friends sidebar column. ──────────
.bottom-panel.bottom-panel--servers {   // two classes: must beat the later .bottom-panel rule (flex column)
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 20px;
  flex: 0 0 auto;
  height: 16vh;   // was: fill the rest of the page (~30vh)
  box-sizing: content-box;
  padding: 14px 0 0;
  border-radius: 0;
  background: transparent;
  overflow: hidden;
}

.server-card {
  position: relative;
  display: grid;
  place-items: center;
  min-height: 0;
  padding: 0;
  border: 0;
  border-radius: 0;   // the global button reset rounds corners; these cards stay square
  background: #0C0D12;
  cursor: pointer;
  overflow: hidden;
  isolation: isolate;

  .server-card-bg {
    position: absolute;
    inset: 0;
    z-index: -2;
    background-size: cover;
    background-position: center;
    transform: scale(1.02);
    filter: saturate(1) brightness(0.8);
    transition: transform 500ms cubic-bezier(0.16, 1, 0.3, 1), filter 300ms ease;
  }
  // dark fade towards the bottom so the label reads
  &::after {
    content: '';
    position: absolute;
    inset: 0;
    z-index: -1;
    background: linear-gradient(to bottom, rgba(8, 9, 13, 0.05) 30%, rgba(8, 9, 13, 0.78) 100%);
    pointer-events: none;
  }

  .server-card-logo {
    height: 58%;          // sized by height: the Hypixel emblem is tall, the other icons are square
    max-height: 130px;
    width: auto;
    max-width: 70%;
    object-fit: contain;
    filter: saturate(1) drop-shadow(0 6px 14px rgba(0, 0, 0, 0.55));
    transition: transform 240ms cubic-bezier(0.34, 1.56, 0.64, 1), filter 300ms ease;
  }

  .server-card-label {
    position: absolute;
    left: 14px;
    bottom: 10px;
    font-family: 'Plus Jakarta Sans', sans-serif;
    font-size: 14px;
    font-weight: 800;
    letter-spacing: 0.02em;
    color: #fff;
    text-shadow: 0 1px 6px rgba(0, 0, 0, 0.7);
  }

  &:hover {
    .server-card-bg { transform: scale(1.07); filter: saturate(1) brightness(0.95); }
    .server-card-logo { transform: scale(1.06); filter: saturate(1) drop-shadow(0 6px 14px rgba(0, 0, 0, 0.55)); }
  }
  &:active .server-card-logo { transform: scale(0.97); }
  &.joining { opacity: 0.45; pointer-events: none; }
}

.bottom-panel {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: $sp-3;
  padding: $sp-4 $sp-5;
  box-sizing: border-box;
  border-radius: 0 0 $radius $radius;   // square at the top where it meets the scene
  background: $friends-embed-bg;
  animation: hub-fade 900ms ease 1100ms both;
  overflow-y: auto;
}

.bottom-tabs {
  display: flex;
  align-items: center;
  gap: $sp-2;
  flex-shrink: 0;
}

.bottom-tab {
  width: 30px;
  height: 30px;
  padding: 0;
  border: none;
  border-radius: $radius-sm;
  background: transparent;
  color: $text-secondary;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  transition: background $transition, color $transition;

  :deep(svg) { width: 16px; height: 16px; }

  &:hover { color: $text-primary; }

  &.active {
    background: $surface-elevated;
    color: $text-primary;
  }
}

.bottom-tab-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 5px;
  border-radius: 8px;
  background: $accent;
  color: #fff;
}

.news-online-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-left: auto;
  font-size: 11px;
  font-weight: 600;
  color: $text-secondary;
}

.news-online-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: $success;
  box-shadow: 0 0 6px $success;
}

.news-list {
  display: flex;
  flex-direction: column;
  gap: $sp-2;
}

.news-item {
  display: flex;
  align-items: flex-start;
  gap: $sp-3;
  padding: $sp-3;
  border-radius: $radius-sm;
  background: $surface;
  transition: background $transition;

  &:hover { background: $surface-elevated; }
}

.news-item-icon {
  flex-shrink: 0;
  width: 30px;
  height: 30px;
  border-radius: $radius-sm;
  background: $surface-elevated;
  display: flex;
  align-items: center;
  justify-content: center;
  color: $accent;

  :deep(svg) { width: 16px; height: 16px; }
}

.news-item-title {
  font-size: 12.5px;
  font-weight: 700;
  color: $text-primary;
  margin-bottom: 2px;
}

.news-item-desc {
  font-size: 11px;
  line-height: 1.5;
  color: $text-secondary;
}

.friends-card-list {
  flex: 1;
  padding: 10px 14px 14px;
  overflow-y: auto;
  scrollbar-width: none;
  &::-webkit-scrollbar { display: none; }
}

.friends-empty {
  font-size: 12px;
  color: $text-muted;
  font-style: italic;
  margin: 0;
}

.friend-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  margin: 0 -8px;
  border-radius: $radius-sm;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  cursor: pointer;
  transition: background 160ms $ease-out;
  opacity: 0;
  animation: hub-row-slide-in 640ms cubic-bezier(0.16, 1, 0.3, 1) 1000ms both;
  &:last-child { border-bottom: none; }

  // Same hover treatment as the top bar's nav items (TopBar.vue's .nav-label):
  // no background fill, just the name dimming/brightening.
  .friend-name { color: rgba(255, 255, 255, 0.22); transition: color 200ms ease; }

  &:hover .friend-avatar { transform: scale(1.06); }
  &:hover:not(.online) .friend-name { color: rgba(255, 255, 255, 0.45); }
  &.online .friend-name { color: #fff; }
}

@for $i from 1 through 16 {
  .friend-row:nth-child(#{$i}) { animation-delay: #{1000ms + ($i - 1) * 70ms}; }
}

.friend-avatar-wrap {
  position: relative;
  flex-shrink: 0;
  width: 36px;
  height: 36px;
}

.friend-avatar {
  width: 36px;
  height: 36px;
  border-radius: $radius-sm;
  image-rendering: pixelated;
  display: block;
  transition: transform 160ms $ease-out;
}

.friend-redirect-badge {
  position: absolute;
  top: -4px;
  left: -4px;
  width: 15px;
  height: 15px;
  border-radius: 50%;
  background: #1c1c1f;
  border: 2px solid $friends-embed-bg;
  display: flex;
  align-items: center;
  justify-content: center;
  color: $text-muted;
}

.friend-status-dot {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: $text-muted;
  border: 2px solid $friends-embed-bg;
  &.online { background: #30d158; box-shadow: 0 0 6px rgba(48, 209, 88, 0.7); }
}

.friend-info {
  min-width: 0;
  flex: 1;
}

.friend-name-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.friend-name {
  font-size: 13px;
  font-weight: 700;
  color: $text-primary;
}

.friend-flag {
  font-size: 12px;
  line-height: 1;
}

.friend-status-line {
  font-size: 11px;
  color: $text-muted;
  margin-top: 2px;
}

.friend-status-game {
  color: #30d158;
  font-weight: 600;
}

// ── Download overlay ──────────────────────────────────────────────────────────
.download-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.download-spinner {
  width: 28px;
  height: 28px;
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-top-color: $primary;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.download-label {
  font-size: 12px;
  color: $text-muted;
  letter-spacing: 0.04em;
}
</style>
