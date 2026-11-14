<template>
  <template v-if="gateChecked">
  <!-- Toasts are always mounted regardless of route - Teleport to body -->
  <div id="toast-dock" />
  <FriendRequestToast />
  <PartyInviteToast />
  <GamepadHints />

  <RouterView v-if="isConsole || isLobby" />
  <div v-else class="app-shell">
    <!-- Splash screen -->
    <Transition name="splash">
      <div v-if="splashVisible" class="splash-screen">
        <img :src="logoUrl" class="splash-logo" alt="" />
      </div>
    </Transition>

    <div v-if="!isConsole && !isLobby" class="main-col" :inert="appLocked">
      <TopBar />
      <main class="main-content" ref="mainRef" @scroll="onScroll">
        <RouterView v-slot="{ Component }">
          <KeepAlive :max="8">
            <component :is="Component" />
          </KeepAlive>
        </RouterView>
      </main>
    </div>
    <!-- TEMP: quick servers row hidden -->
    <div v-if="false && !isConsole && !isLobby" class="corner-servers">
      <ServerRow />
    </div>
    <NotificationsDrawer />
    <CrashAnalyzerModal />
    <ConflictWarningModal />
    <SharedProfileModal />
    <InstanceWizard />
    <UpdateNotification />
    <WhatsNewModal />
    <LoginOverlay :show="appLocked" />
    <BetaWarningToast />
    <ToastHost />
    <ContextMenu />
    <CommandPalette />
  </div>
  </template>
</template>

<script setup lang="ts">
import { onMounted, watch, ref, computed, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import TopBar    from './components/layout/TopBar.vue'
import ServerRow from './components/home/ServerRow.vue'
import logoUrl  from './assets/bc-logo-new.png'

import NotificationsDrawer from './components/layout/NotificationsDrawer.vue'
import UpdateNotification from './components/common/UpdateNotification.vue'
import WhatsNewModal from './components/common/WhatsNewModal.vue'
import CrashAnalyzerModal from './components/common/CrashAnalyzerModal.vue'
import ConflictWarningModal from './components/common/ConflictWarningModal.vue'
import SharedProfileModal from './components/common/SharedProfileModal.vue'
import InstanceWizard from './components/common/InstanceWizard.vue'
import FriendRequestToast from './components/common/FriendRequestToast.vue'
import PartyInviteToast from './components/common/PartyInviteToast.vue'
import GamepadHints from './components/common/GamepadHints.vue'
import { startGamepad } from './composables/useGamepad'
import LoginOverlay from './components/common/LoginOverlay.vue'
import BetaWarningToast from './components/common/BetaWarningToast.vue'
import ToastHost from './components/common/ToastHost.vue'
import ContextMenu from './components/common/ContextMenu.vue'
import CommandPalette from './components/common/CommandPalette.vue'
import { useScrollState } from './composables/useScrollState'
import { useReducedMotion } from './composables/useReducedMotion'
import { useCommandPalette } from './composables/useCommandPalette'
import { useAccountStore } from './store/accountStore'
import { useLauncherStore } from './store/launcherStore'
import { useSettingsStore } from './store/settingsStore'
import { useFriendsStore } from './store/friendsStore'
import { useLobbyStore } from './store/lobbyStore'
import { useNotificationsStore } from './store/notificationsStore'
import { useLockerStore } from './store/lockerStore'
import { useQuestsStore } from './store/questsStore'
import { playLaunch, playMouseClick, warmAudio } from './composables/useSounds'

const route     = useRoute()
const isConsole = computed(() => route.path === '/console')
const isLobby   = computed(() => route.path === '/lobby')

const { locale } = useI18n()

const splashVisible   = ref(true)
const loadProgress    = ref(0)
const loadStage       = ref('Starting up…')
const splashStartedAt = Date.now()
const MIN_SPLASH_MS   = 2200

// SVG ring progress - circumference of r=54 is 2πr ≈ 339.29; dashoffset shrinks from full
// (nothing drawn) to 0 (fully drawn) as loadProgress climbs to 100.
const RING_CIRCUMFERENCE = 2 * Math.PI * 54
const ringOffset = computed(() => RING_CIRCUMFERENCE * (1 - loadProgress.value / 100))

const { updateScroll } = useScrollState()
const mainRef = ref<HTMLElement | null>(null)

function onScroll() {
  if (mainRef.value) updateScroll(mainRef.value)
}

useReducedMotion()

const accountStore  = useAccountStore()
// Mirrors LoginOverlay's own `show` condition (which TopBar's own "Sign in" pill also uses:
// `selectedAccount`, not just `hasAccounts` - an account can exist in storage without being the
// active one, e.g. after a logout, and that must still count as signed out). `inert` keeps the
// app behind it fully unreachable (mouse, tab focus, everything) while signed out, on top of the
// overlay's raised z-index - z-index alone only stops clicks, not keyboard-driven navigation.
const appLocked = computed(() => !splashVisible.value && !accountStore.selectedAccount)
const launcherStore = useLauncherStore()
const settingsStore = useSettingsStore()
const friendsStore  = useFriendsStore()
const lobbyStore    = useLobbyStore()
const notifStore    = useNotificationsStore()
const lockerStore   = useLockerStore()

// ── Discord Rich Presence - reflects what's actually on screen ──────────────
// Never runs while a game is actually running: launch_service.rs already owns
// the presence then ("Playing Minecraft <version>"), and this must not stomp it.
// `state` mirrors the actual TopBar nav label for that route (see TopBar.vue's
// navItems) so the two RPC lines read as "<action> - <tab>" instead of every
// tab collapsing into the same generic "In the launcher" second line.
const ROUTE_PRESENCE: Record<string, { details: string; state: string }> = {
  '/':          { details: 'On the Hub', state: 'Hub' },
  '/cosmetics': { details: 'Browsing the Locker', state: 'Locker' },
  '/mods':      { details: 'Exploring mods', state: 'Explore' },
  '/store':     { details: 'Browsing the Store', state: 'Store' },
  '/quests':    { details: 'Viewing quests', state: 'Quests' },
  '/capes':     { details: 'Browsing capes', state: 'Capes' },
  '/versions':  { details: 'Browsing versions', state: 'Versions' },
  '/settings':  { details: 'Configuring settings', state: 'Settings' },
  '/profiles':  { details: 'Browsing profiles', state: 'Profiles' },
  '/friends':   { details: 'Browsing friends', state: 'Friends' },
}

const discordPresence = computed(() => {
  if (route.path === '/profiles' && launcherStore.editingProfileName) {
    return { details: `Editing profile ${launcherStore.editingProfileName}`, state: 'Profiles' }
  }
  if (route.path === '/friends' && friendsStore.activeChatUsername) {
    return { details: `Chatting with ${friendsStore.activeChatUsername}`, state: 'Friends' }
  }
  return ROUTE_PRESENCE[route.path] ?? { details: 'Browsing the launcher', state: 'Hub' }
})

watch(discordPresence, (p) => {
  if (isConsole.value || isLobby.value) return
  if (launcherStore.status === 'running') return
  window.api.discord.setPresence(p.details, p.state)
}, { immediate: true })

function applyAccent(color: string) {
  document.documentElement.style.setProperty('--accent', color)
}


// The app no longer gates entry behind an access code - it unlocks straight
// into the normal boot flow. Console window is exempt (it's a separate
// mini-window, not the main app).
const gateChecked = ref(false)

async function initApp() {
  loadStage.value = 'Loading settings…'
  await settingsStore.load()
  loadProgress.value = 25

  if (settingsStore.settings.appearance.disableSplashScreen) splashVisible.value = false

  loadStage.value = 'Signing in…'
  await accountStore.loadAccounts()
  loadProgress.value = 50

  // Seed locker store from BejaClient cosmetics + account data (non-blocking)
  const acct = accountStore.selectedAccount
  if (acct) {
    const skinUrl = acct.skinUrl
    const model   = acct.skinModel ?? 'default'
    let capeUrl   = localStorage.getItem('beja_local_cape_url') ?? acct.capeUrl ?? null
    window.api.cosmetics.get(acct.uuid)
      .then((cosmetics: { cape_url?: string | null } | null) => {
        if (cosmetics?.cape_url) capeUrl = cosmetics.cape_url
        if (skinUrl) lockerStore.selectSkin({ skinUrl, capeUrl, model })
      })
      .catch(() => {
        if (skinUrl) lockerStore.selectSkin({ skinUrl, capeUrl, model })
      })
  }

  loadStage.value = 'Loading profiles…'
  await launcherStore.loadProfiles()
  loadProgress.value = 75

  loadStage.value = 'Ready'
  loadProgress.value = 100
  await nextTick()

  // Wait until the route component is actually painted in the DOM before closing splash
  await new Promise<void>(resolve => {
    let ticks = 0
    function check() {
      ticks++
      if (document.querySelector('.home-page') || ticks > 120) resolve()
      else requestAnimationFrame(check)
    }
    requestAnimationFrame(check)
  })

  if (splashVisible.value) {
    const remaining = Math.max(0, MIN_SPLASH_MS - (Date.now() - splashStartedAt))
    setTimeout(() => { splashVisible.value = false }, remaining + 150)
  }

  // Populates `friends`/`requests` and opens the live friends socket - nothing
  // else in the app called this, so incomingRequests was always empty here
  // (no pre-existing pending requests ever got surfaced as a notification on
  // launch) and the socket never connected (no live friend:request/accepted
  // pushes, no online/offline presence - only ever refreshed by visiting the
  // Friends page itself, which calls refresh() on its own mount).
  await friendsStore.connect()
  for (const req of friendsStore.incomingRequests) {
    notifStore.addFriendRequest(req.uuid, req.username)
  }
  applyAccent(settingsStore.settings.appearance.accentColor)
  locale.value = settingsStore.settings.appearance.language
  launcherStore.setupLaunchListeners()
  useQuestsStore().setupTracking()
  window.api.friends.onOnline(d  => friendsStore.handleOnline(d))
  window.api.friends.onOffline(d => friendsStore.handleOffline(d))
  window.api.friends.onRequest(d => {
    friendsStore.handleRequest(d)
    notifStore.addFriendRequest(d.uuid, d.username)
  })
  window.api.friends.onAccepted(() => friendsStore.handleAccepted())
  window.api.friends.onRemoved(()  => friendsStore.handleRemoved())

  // Lobby / party socket events
  window.api.lobby.onPartyState(d   => lobbyStore.handlePartyState(d as any))
  window.api.lobby.onMemberJoined(d => lobbyStore.handleMemberJoined(d as any))
  window.api.lobby.onMemberLeft(d   => lobbyStore.handleMemberLeft(d as any))
  window.api.lobby.onReadyUpdate(d  => lobbyStore.handleReadyUpdate(d as any))
  window.api.lobby.onSkinUpdate(d   => lobbyStore.handleSkinUpdate(d as any))
  window.api.lobby.onSpeaking(d     => lobbyStore.handleSpeaking(d as any))
  window.api.lobby.onDisbanded(()   => lobbyStore.handleDisbanded())
  window.api.lobby.onError(d        => lobbyStore.handlePartyError(d as any))
  window.api.lobby.onInviteReceived(d => lobbyStore.handleInviteReceived(d))
  window.api.lobby.onLaunched(async d => {
    const data = d as { server: string; port: number; profileId: string }
    try {
      await window.api.lobby.startWithServer(data.profileId, data.server, data.port)
    } catch { /* non-fatal */ }
  })

  warmAudio()
  startGamepad()
  document.addEventListener('mousedown', (e) => { if (e.button === 0) playMouseClick() })

  const { togglePalette } = useCommandPalette()
  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault()
      // Not signed in yet - the whole app behind the login overlay is meant to be
      // unreachable, and the palette can jump straight to pages/actions that bypass it.
      if (appLocked.value) return
      togglePalette()
    }
  })
}

// This is a shipped desktop app, not a website - the native WebView2 menu
// (Reload/Inspect/View Source/etc.) should never appear. Suppressed globally
// here; any element that wants its own menu calls openContextMenu(event, …)
// which does its own preventDefault() first, so this doesn't interfere with
// e.g. the chat message menu or the mods-list menu.
document.addEventListener('contextmenu', (e) => e.preventDefault())

// Same reasoning - native drag-out-of-the-window for images/icons/links
// reads as a leaky browser habit, not a native app. CSS (see _reset.scss)
// covers most cases; this is the JS-level belt-and-suspenders backstop.
document.addEventListener('dragstart', (e) => e.preventDefault())

onMounted(async () => {
  gateChecked.value = true
  if (isConsole.value) return
  await initApp()
})

watch(() => settingsStore.settings.appearance.accentColor, applyAccent)
watch(() => settingsStore.settings.appearance.language, (lang) => { locale.value = lang })
watch(
  () => settingsStore.settings.appearance.disableHoverEffects,
  (v) => document.documentElement.classList.toggle('no-hover-fx', v),
  { immediate: true },
)
watch(
  () => settingsStore.settings.appearance.theme,
  (v) => document.documentElement.classList.toggle('theme-win95', v === 'win95'),
  { immediate: true },
)

watch(() => launcherStore.status, (val, prev) => {
  if (val === 'running' && prev !== 'running') playLaunch()
  // Game just stopped - re-sync presence to whatever page is on screen now,
  // since set_idle_presence() (called from launch_service.rs on stop) always
  // resets to the generic default and would otherwise clobber e.g. "Configuring
  // settings" if that's actually what's showing.
  if (prev === 'running' && val !== 'running' && !isConsole.value && !isLobby.value) {
    window.api.discord.setPresence(discordPresence.value.details, discordPresence.value.state)
  }
})
</script>

<style lang="scss">
*, *::before, *::after { box-sizing: border-box; }

body { margin: 0; background: $bg; }

// ── Splash screen ─────────────────────────────────────────────────────────────
$splash-ease: cubic-bezier(0.16, 1, 0.3, 1);

.splash-screen {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: #000;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  animation: splash-self-fade 2200ms $splash-ease forwards;
}

@keyframes splash-self-fade {
  0%, 50% { opacity: 1; }
  100%    { opacity: 0; }
}

.splash-logo {
  width: 220px;
  height: 220px;
  object-fit: contain;
  animation: splash-logo-wiggle 2200ms $splash-ease forwards;
}

@keyframes splash-logo-wiggle {
  0%   { transform: rotate(0deg); }
  20%  { transform: rotate(30deg); }
  45%  { transform: rotate(-30deg); }
  100% { transform: rotate(360deg); }
}

// Transition - clean fade only
.splash-enter-active { transition: opacity 0.5s ease; }
.splash-leave-active { transition: opacity 0.6s ease; }
.splash-enter-from, .splash-leave-to { opacity: 0; }

.bg-layer {
  position: absolute;
  inset: 0;
  background-size: cover;
  background-position: center;
  z-index: -1;
  pointer-events: none;
}

.app-shell {
  display: flex;
  flex-direction: row;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  color: $text-primary;
  background-color: $bg;
  isolation: isolate;
}

// Floating overlay, top-right, just under the TopBar (which now owns the
// brand/logo and is full-width, so there's no more empty corner to sit
// inside like the old sidebar-era corner-brand had). Not part of TopBar
// itself - that row is already full with nav/search/gems/account/window
// controls - so this stays a free-floating sibling over main-content.
.corner-servers {
  position: absolute;
  top: 78px;
  right: 12px;
  z-index: 150;
  -webkit-app-region: no-drag;
  animation: corner-servers-fade-in 800ms cubic-bezier(0.16, 1, 0.3, 1) 300ms both;
}

@keyframes corner-servers-fade-in {
  from { opacity: 0; transform: translateY(-10px); }
  to   { opacity: 1; transform: translateY(0); }
}

.main-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  position: relative;
  z-index: 1;
  min-height: 0;
  overflow: hidden;
  background: transparent;
  isolation: isolate;
}

.main-content {
  flex: 1;
  position: relative;
  overflow-y: auto;
  overflow-x: hidden;
  min-height: 0;
  background: transparent;
  scrollbar-width: none;

  &::-webkit-scrollbar { display: none; }
}

// friend requests and party invites share one stack, so they never overlap
#toast-dock {
  position: fixed;
  bottom: 24px;
  right: 80px; // clears the right panel
  z-index: 9000;
  display: flex;
  flex-direction: column-reverse;
  gap: 8px;
  pointer-events: none;
}
</style>
