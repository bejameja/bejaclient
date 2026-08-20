<template>
  <div class="home-page" :class="{ 'no-intro': !playIntro }">

    <!-- Main scrollable column -->
    <div class="main-col">

      <div class="hero-wrap">
        <EditableRegion
          id="home.skinPreviewBg"
          label="Skin-Vorschau Hintergrund"
          :features="['radius', 'outline', 'bgVideo']"
        >
        <div class="video-card">
          <video v-if="displayVideo" ref="videoRef" class="scene-video" :src="displayVideo" autoplay loop muted playsinline @error="onVideoError" />
          <div v-else class="download-overlay">
            <div class="download-spinner" />
            <span class="download-label">Downloading assets…</span>
          </div>

          <!-- Left flanking member -->
          <div class="flank-slot flank-slot--left">
            <LobbySkinSlot :member="lobbySlots[0]" size="2xl" :initial-rotation-y="0.524" @invite="openInvite" />
          </div>

          <!-- Center: local player — preserves original HeroSkinViewer positioning/animation -->
          <div class="skin-wrap">
            <div v-if="lobbyStore.isLeader" class="slot-crown">
              <svg width="22" height="17" viewBox="0 0 22 17" fill="none">
                <path d="M1 15L4.5 5.5L11 10L17.5 2L21 9.5V15H1Z" fill="#FFD700" stroke="#E8A800" stroke-width="1"/>
              </svg>
            </div>
            <HeroSkinViewer
              ref="heroViewerRef"
              :skin-url="activeSkinUrl"
              :cape-url="activeCapeUrl"
              :model="activeSkinModel"
              animation="custom-idle"
              :zoom="0.75"
              :initial-rotation-y="0.524"
              :auto-rotate-speed="0"
            />
            <div class="skin-footer">
              <div v-if="account" class="skin-namebar">
                <span class="skin-you-tag">You</span>
                <span class="skin-username">{{ account.username }}</span>
                <span v-if="ownedEmotes.length" class="skin-emote-hint">Press B for emotes</span>
              </div>
              <Transition name="emote-pop">
                <div v-if="ownedEmotes.length && emotePickerOpen" ref="emotePickerEl" class="emote-bar">
                  <button
                    v-for="item in ownedEmotes"
                    :key="item.id"
                    class="emote-btn"
                    :title="item.name"
                    @click="playEmote(item)"
                  >
                    <svg viewBox="0 0 24 24" class="emote-btn-icon">
                      <template v-if="item.id === 'shop_emote_floss'">
                        <circle cx="12.6" cy="4.6" r="2" fill="#ffb454" />
                        <g stroke="#ffb454" stroke-width="1.8" stroke-linecap="round" fill="none">
                          <path d="M12.4 6.8 11.7 13.2" />
                          <path d="M12.1 8.4 17.4 6.2" />
                          <path d="M12.1 8.4 17.9 9.4" />
                          <path d="M11.7 13.2 9.4 19" />
                          <path d="M11.7 13.2 13.6 19.2" />
                        </g>
                      </template>
                      <template v-else-if="item.id === 'shop_emote_griddy'">
                        <circle cx="10" cy="4.6" r="2" fill="#b46eff" />
                        <g stroke="#b46eff" stroke-width="1.8" stroke-linecap="round" fill="none">
                          <path d="M10.3 6.8 11.3 13" />
                          <path d="M10.7 8.6 16.5 7.4" />
                          <path d="M10.9 10.2 16.5 9.8" />
                          <path d="M11.3 13 10 19.2" />
                          <path d="M11.3 13 6.4 16.2" />
                        </g>
                      </template>
                      <template v-else-if="item.id === 'shop_emote_rat_dance'">
                        <ellipse cx="11" cy="14.8" rx="6.5" ry="4.3" fill="#b9bcc7" />
                        <path d="M16.2 12.6 21 14.8l-4.8 2z" fill="#b9bcc7" />
                        <circle cx="15" cy="10.2" r="1.9" fill="#b9bcc7" />
                        <circle cx="15" cy="10.2" r=".9" fill="#f2a6c0" />
                        <circle cx="17.3" cy="14.2" r=".7" fill="#26262e" />
                        <path d="M4.7 15.6c-1.9 0-2.8 1.7-1.7 3.1" stroke="#b9bcc7" stroke-width="1.3" fill="none" stroke-linecap="round" />
                      </template>
                    </svg>
                    <span class="emote-btn-name">{{ item.name }}</span>
                  </button>
                </div>
              </Transition>
            </div>
          </div>

          <!-- Right flanking member -->
          <div class="flank-slot flank-slot--right">
            <LobbySkinSlot :member="lobbySlots[1]" size="2xl" :initial-rotation-y="-0.524" @invite="openInvite" />
          </div>

          <!-- Voice controls (shown when party has ≥2 members or voice is active) -->
          <div class="voice-controls">
            <button
              class="voice-btn"
              :class="{ active: !voice.isMuted.value, muted: voice.isMuted.value }"
              :title="voice.isMuted.value ? 'Unmute' : 'Mute'"
              @click="voice.toggleMic()"
            >
              <svg v-if="!voice.isMuted.value" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><line x1="12" y1="19" x2="12" y2="23"/><line x1="8" y1="23" x2="16" y2="23"/>
              </svg>
              <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="1" y1="1" x2="23" y2="23"/><path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"/><path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23"/><line x1="12" y1="19" x2="12" y2="23"/><line x1="8" y1="23" x2="16" y2="23"/>
              </svg>
            </button>
            <button
              class="voice-btn"
              :class="{ active: !voice.isDeafened.value, muted: voice.isDeafened.value }"
              :title="voice.isDeafened.value ? 'Undeafen' : 'Deafen'"
              @click="voice.toggleDeafen()"
            >
              <svg v-if="!voice.isDeafened.value" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 18v-6a9 9 0 0 1 18 0v6"/><path d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3zM3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z"/>
              </svg>
              <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="1" y1="1" x2="23" y2="23"/><path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"/><path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23"/>
              </svg>
            </button>
            <div v-if="lobbyStore.party" class="party-id">
              {{ lobbyStore.party.id }}
              <button
                class="party-id-refresh"
                title="Generate a new code"
                :disabled="lobbyStore.isCreating"
                @click="lobbyStore.regenerateParty()"
              >
                <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/>
                  <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
                </svg>
              </button>
            </div>
            <button
              v-else
              class="voice-btn create-lobby-btn"
              title="Create a lobby and get a code"
              :disabled="lobbyStore.isCreating"
              @click="lobbyStore.createParty()"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
              Create Lobby
            </button>
            <InfoTooltip
              v-if="!lobbyStore.party"
              text="Creates a shareable code so friends can join your session and launch together."
              class="create-lobby-info"
            />
            <button class="voice-btn join-party-btn" title="Join a party by code" @click="openJoin">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/><polyline points="10 17 15 12 10 7"/><line x1="15" y1="12" x2="3" y2="12"/>
              </svg>
            </button>
          </div>
        </div>
        </EditableRegion>

        <!-- Launch / Ready row -->
        <div class="hero-launch">
          <div class="launch-ropes" aria-hidden="true">
            <span class="rope rope--left"></span>
            <span class="rope rope--right"></span>
          </div>
          <div class="launch-drop">
            <LaunchButton v-if="lobbyStore.isLeader || !lobbyStore.party" />
            <button v-else class="ready-btn" :class="{ 'ready-btn--ready': lobbyStore.isReady }" @click="lobbyStore.toggleReady()">
              <svg v-if="lobbyStore.isReady" width="14" height="14" viewBox="0 0 14 14" fill="none">
                <path d="M2 7l3.5 3.5 6.5-6.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              {{ lobbyStore.isReady ? 'Ready' : 'Not Ready' }}
            </button>
          </div>
        </div>
      </div>

      <EditableRegion id="home.emptyEmbed" label="Leeres Embed" flex-fill :features="['radius', 'outline', 'color', 'bgColor', 'fontFamily']">
      <div class="empty-embed">
        <div class="maintenance-notice">
          <svg class="maintenance-icon" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="9" />
            <line x1="12" y1="8" x2="12" y2="13" />
            <line x1="12" y1="16" x2="12" y2="16.01" />
          </svg>
          <div class="maintenance-text">
            <h3 class="maintenance-title">{{ $t('home.maintenanceTitle') }}</h3>
            <p class="maintenance-body">{{ $t('home.maintenanceBody') }}</p>
          </div>
        </div>
      </div>
      </EditableRegion>

    </div>

    <!-- Friends panel -->
    <div class="friends-panel">
      <EditableRegion id="home.friendsCard" label="Freunde-Karte" flex-fill :features="['radius', 'outline', 'color', 'fontFamily']">
      <div class="friends-card">
        <div class="friends-card-header">
          <div class="friends-heading-row">
            <h2 class="friends-heading">{{ $t('home.friends') }}</h2>
          </div>
          <span class="friends-online-count">
            {{ $t('home.onlineCount', { count: friendsStore.onlineCount }) }}
          </span>
        </div>
        <div class="friends-card-list">
          <template v-if="friends.length">
            <div
              v-for="friend in friends"
              :key="friend.uuid"
              class="friend-row"
              @click="openFriendChat(friend)"
            >
              <div class="friend-avatar-wrap">
                <img
                  class="friend-avatar"
                  :src="`https://mc-heads.net/head/${friend.uuid}/64`"
                  :alt="friend.username"
                  @error="(e: Event) => ((e.target as HTMLImageElement).src = 'https://mc-heads.net/head/MHF_Steve/64')"
                />
                <span class="friend-redirect-badge"><Icon name="external-link" :size="9" /></span>
                <span class="friend-status-dot" :class="{ online: friend.online }" />
              </div>
              <div class="friend-info">
                <div class="friend-name-row">
                  <span class="friend-name">{{ friend.username }}</span>
                  <span v-if="friend.countryCode" class="friend-flag">{{ countryFlag(friend.countryCode) }}</span>
                </div>
                <div v-if="friend.playing" class="friend-status-line">
                  {{ $t('home.playing') }} <span class="friend-status-game">{{ friend.playing.game }} {{ friend.playing.version }}</span>
                </div>
              </div>
            </div>
          </template>
          <p v-else class="friends-empty">{{ $t('home.noFriends') }}</p>
        </div>
      </div>
      </EditableRegion>
    </div>

    <!-- Invite overlay -->
    <InviteOverlay :visible="inviteOpen" :initial-tab="inviteInitTab" @close="inviteOpen = false" />

  </div>
</template>

<script setup lang="ts">
import { computed, ref, onActivated, onDeactivated, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useFriendsStore, type Friend } from '../store/friendsStore'
import { useAccountStore }  from '../store/accountStore'
import { useLockerStore }   from '../store/lockerStore'
import { useLobbyStore }    from '../store/lobbyStore'
import { useShopStore }     from '../store/shopStore'
import type { ShopItem }    from '../types'
import { useLobbyVoice }    from '../composables/useLobbyVoice'
import LobbySkinSlot  from '../components/lobby/LobbySkinSlot.vue'
import InviteOverlay  from '../components/lobby/InviteOverlay.vue'
import HeroSkinViewer from '../components/skin/HeroSkinViewer.vue'
import LaunchButton   from '../components/home/LaunchButton.vue'
import InfoTooltip    from '../components/common/InfoTooltip.vue'
import Icon           from '../components/common/Icon.vue'
import EditableRegion from '../components/common/EditableRegion.vue'
import { useElementStyle } from '../composables/useElementStyle'
import { consumeHubIntro } from './homePageIntro'
// ── Video ─────────────────────────────────────────────────────────────────────

// Module-level (see homePageIntro.ts): true only the first time this app
// session reaches the hub, false on every later tab switch back to it —
// even if KeepAlive's LRU cache evicts and remounts the component.
const playIntro = consumeHubIntro()

const sceneVideo = ref('')
const videoRef   = ref<HTMLVideoElement | null>(null)

// Editor Mode — lets the user swap the hero/skin-preview background for a
// custom video (see SettingsPage → Appearance → Editor Mode).
const { override: heroBgOverride } = useElementStyle('home.skinPreviewBg')
const displayVideo = computed(() => heroBgOverride.value.bgVideo || sceneVideo.value)

// ── Stores / composables ──────────────────────────────────────────────────────

const friendsStore  = useFriendsStore()
const accountStore  = useAccountStore()
const lobbyStore    = useLobbyStore()
const shopStore     = useShopStore()
const voice         = useLobbyVoice()

const heroViewerRef = ref<InstanceType<typeof HeroSkinViewer> | null>(null)

const lockerStore = useLockerStore()
const account     = computed(() => accountStore.selectedAccount)

const activeSkinUrl   = computed(() => lockerStore.skinUrl  ?? account.value?.skinUrl  ?? null)
const activeCapeUrl   = computed(() => lockerStore.capeUrl  ?? account.value?.capeUrl  ?? null)
const activeSkinModel = computed(() => lockerStore.model    ?? account.value?.skinModel ?? 'default')

const friends    = computed(() => friendsStore.friends)
const lobbySlots = computed(() => lobbyStore.slots)

const router = useRouter()

function countryFlag(countryCode: string): string {
  return String.fromCodePoint(...[...countryCode.toUpperCase()].map(c => 127397 + c.charCodeAt(0)))
}

function openFriendChat(friend?: Friend) {
  const target = friend ?? friends.value.find(f => f.online) ?? friends.value[0]
  if (!target) return
  router.push({ name: 'friends', query: { chatWith: target.uuid } })
}

// ── Emotes ────────────────────────────────────────────────────────────────────

const ownedEmotes     = computed<ShopItem[]>(() =>
  shopStore.items.filter(i => i.type === 'emote' && shopStore.owned.includes(i.id)),
)
const emotePickerOpen = ref(false)
const emotePickerEl   = ref<HTMLElement | null>(null)

function playEmote(item: ShopItem) {
  heroViewerRef.value?.triggerEmote()
  window.api.lobby.emit('party:emote', { emote: item.id })
  emotePickerOpen.value = false
}

function isTypingTarget(el: EventTarget | null): boolean {
  if (!(el instanceof HTMLElement)) return false
  return el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable
}

function onEmoteKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && emotePickerOpen.value) { emotePickerOpen.value = false; return }
  if (e.key.toLowerCase() !== 'b' || e.ctrlKey || e.metaKey || e.altKey) return
  if (isTypingTarget(e.target) || inviteOpen.value) return
  if (!ownedEmotes.value.length) return
  emotePickerOpen.value = !emotePickerOpen.value
}

function onEmoteOutsideClick(e: MouseEvent) {
  if (!emotePickerOpen.value) return
  if (emotePickerEl.value && !emotePickerEl.value.contains(e.target as Node)) {
    emotePickerOpen.value = false
  }
}

onActivated(() => {
  window.addEventListener('keydown', onEmoteKeydown)
  document.addEventListener('mousedown', onEmoteOutsideClick)
})

onDeactivated(() => {
  window.removeEventListener('keydown', onEmoteKeydown)
  document.removeEventListener('mousedown', onEmoteOutsideClick)
  emotePickerOpen.value = false
})

onMounted(() => {
  if (!shopStore.items.length) shopStore.load()
})

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
function openJoin() { inviteInitTab.value = 'join'; inviteOpen.value = true }

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
  // Load background video
  try {
    sceneVideo.value = await (window as any).api.video.getScene()
  } catch {}

  // Init voice capture
  await voice.init()
  initVoiceIpc()
})

onActivated(() => { videoRef.value?.play() })
// Kept alive by App.vue's <KeepAlive> — pause the decode/GPU work while
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

// ── Hub entrance — plays once on mount (KeepAlive skips it on tab revisits) ───
@keyframes hub-rise {
  from { opacity: 0; transform: translateY(64px) scale(0.93); filter: blur(6px); }
  to   { opacity: 1; transform: translateY(0) scale(1); filter: blur(0); }
}

@keyframes hub-fade {
  from { opacity: 0; }
  to   { opacity: 1; }
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

@keyframes launch-drop-y {
  0%   { opacity: 0; transform: translateY(-170px); }
  8%   { opacity: 1; }
  100% { transform: translateY(0); }
}

@keyframes launch-rope-extend {
  0%   { height: 2px; opacity: 0; }
  8%   { opacity: 1; }
  100% { height: 170px; }
}

// Tab revisit (not a real app launch) — skip the one-time entrance
// animations but keep the continuous idle loops (skinFloat, crown-float).
.home-page.no-intro {
  .launch-drop, .rope, .video-card, .voice-controls,
  .friends-card-header, .empty-embed, .friends-card, .friend-row {
    animation: none;
  }
  .skin-wrap {
    animation: skinFloat 3s ease-in-out infinite alternate;
  }
  .flank-slot {
    animation-name: skinFloat;
    animation-duration: 3s;
    animation-timing-function: ease-in-out;
    animation-iteration-count: infinite;
    animation-direction: alternate;
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

.hero-launch {
  position: absolute;
  bottom: 20px;
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
  transform-origin: top center;
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
  width: 2px;
  height: 2px;
  opacity: 0;
  background: linear-gradient(to bottom, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.4));
  transform-origin: top center;
  animation: launch-rope-extend 1100ms cubic-bezier(0.16, 1, 0.3, 1) 980ms both;

  &--left { left: -108px; }
  &--right { left: 106px; }
}

// ── Video card ────────────────────────────────────────────────────────────────
@property --border-angle {
  syntax: '<angle>';
  initial-value: 0deg;
  inherits: false;
}

.video-card {
  width: 100%;
  height: 56vh;
  border-radius: var(--edr-radius, 4px);
  overflow: hidden;
  clip-path: inset(0 round var(--edr-radius, 4px));
  outline: var(--edr-outline, none);
  outline-offset: 3px;
  flex-shrink: 0;
  position: relative;
  border: 2px solid transparent;
  background-image:
    linear-gradient(#111, #111),
    conic-gradient(
      from var(--border-angle),
      rgba(255, 255, 255, 0.04) 0%,
      rgba(255, 255, 255, 0.55) 18%,
      rgba(255, 255, 255, 0.04) 36%,
      rgba(255, 255, 255, 0.04) 100%
    );
  background-origin: border-box;
  background-clip: padding-box, border-box;
  transition: --border-angle 2000ms cubic-bezier(0.2, 0, 0, 1);
  animation: hub-rise 1200ms cubic-bezier(0.16, 1, 0.3, 1) both;

  &:hover {
    --border-angle: 135deg;
  }
}

.scene-video {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  transform: scale(1.15);
}

// ── Center player (HeroSkinViewer, original positioning) ─────────────────────
.skin-wrap {
  position: absolute;
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  // width scales with vh (like height below) so the aspect ratio stays constant
  // across window sizes — a fixed px width against a vh-based height stretched
  // the viewer taller/narrower on bigger screens, clipping arms/shoulders.
  width: 36.8vh;
  height: calc(100% + 14vh);
  pointer-events: none;
  animation: skinFloat 3s ease-in-out infinite alternate, hub-fade-blur 1100ms ease 420ms both;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.slot-crown {
  margin-top: 10px;
  margin-bottom: -4px;
  filter: drop-shadow(0 0 8px rgba(255, 215, 0, 0.65));
  animation: crown-float 3s ease-in-out infinite;
  z-index: 2;
}

@keyframes crown-float {
  0%, 100% { transform: translateY(0); }
  50%       { transform: translateY(-3px); }
}

@keyframes skinFloat {
  from { transform: translateX(-50%) translateY(0px); }
  to   { transform: translateX(-50%) translateY(-14px); }
}

.skin-footer {
  position: absolute;
  bottom: calc(14vh + 10px);
  display: flex;
  flex-direction: column-reverse;
  align-items: center;
  gap: 8px;
  z-index: 15;
}

.skin-namebar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 4px;
  background: rgba(255,255,255,0.06);
  border: 1px solid rgba(255,255,255,0.08);
  pointer-events: none;
}

.skin-emote-hint {
  font-size: 10px;
  color: $text-muted;
  letter-spacing: 0.2px;
  padding-left: 4px;
  border-left: 1px solid rgba(255, 255, 255, 0.12);
}

.emote-bar {
  @extend %glass-panel;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px;
  border-radius: 4px;
  pointer-events: auto;
}

.emote-pop-enter-active { transition: opacity 140ms ease, transform 140ms cubic-bezier(0.34, 1.56, 0.64, 1); }
.emote-pop-leave-active { transition: opacity 100ms ease, transform 100ms ease; }
.emote-pop-enter-from,
.emote-pop-leave-to {
  opacity: 0;
  transform: translateY(6px) scale(0.94);
}

.emote-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  cursor: pointer;
  transition: background 120ms ease, transform 120ms ease;

  &:hover { background: rgba(255, 255, 255, 0.14); transform: scale(1.05); }
  &:active { transform: scale(0.96); }
}

.emote-btn-icon {
  width: 22px;
  height: 22px;
}

.emote-btn-name {
  font-size: 10px;
  color: $text-secondary;
  white-space: nowrap;
}

.skin-username {
  font-size: 13px;
  font-weight: 600;
  color: $text-primary;
}

.skin-you-tag {
  font-size: 10px;
  color: $accent;
  font-weight: 700;
  background: rgba(85, 178, 255, 0.18);
  border-radius: 4px;
  padding: 1px 5px;
  letter-spacing: 0.5px;
}

// ── Flanking lobby slots ──────────────────────────────────────────────────────
.flank-slot {
  position: absolute;
  top: 33%;
  animation-name: skinFloat, hub-fade;
  animation-duration: 3s, 900ms;
  animation-timing-function: ease-in-out, ease;
  animation-iteration-count: infinite, 1;
  animation-direction: alternate, normal;
  animation-fill-mode: none, both;

  &--left  { left: 20%; transform: translateX(-50%); animation-delay: -1.1s, 640ms; }
  &--right { left: 80%; transform: translateX(-50%); animation-delay: -2.2s, 780ms; }
}

// ── Voice controls ────────────────────────────────────────────────────────────
.voice-controls {
  position: absolute;
  bottom: 14px;
  right: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  z-index: 10;
  animation: hub-fade 800ms ease 1100ms both;
}

.voice-btn {
  width: 34px;
  height: 34px;
  border-radius: 4px;
  border: none;
  background: #0d0d0d;
  color: rgba(255, 255, 255, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 160ms $ease-out, color 160ms $ease-out;

  &.active { color: rgba(255, 255, 255, 0.9); }
  &.muted  { color: #ff453a; background: rgba(255, 69, 58, 0.12); }
  &:hover  { background: #1a1a1a; }
}

.party-id {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: 'IBM Plex Mono', monospace;
  font-size: 10px;
  letter-spacing: 0.16em;
  color: rgba(255, 255, 255, 0.5);
  background: #0d0d0d;
  padding: 4px 6px 4px 8px;
  border-radius: 4px;
  border: none;
  user-select: all;
}

.create-lobby-btn {
  position: relative;
  width: auto;
  height: 34px;
  border-radius: 4px;
  padding: 0 14px;
  gap: 6px;
  font-size: 11px;
  font-weight: 400;
  letter-spacing: 0.02em;
  white-space: nowrap;
  overflow: hidden;
  background: rgba(32, 32, 36, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.12);
  backdrop-filter: blur(14px) saturate(160%);
  -webkit-backdrop-filter: blur(14px) saturate(160%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.14),
    0 12px 32px rgba(0, 0, 0, 0.55);

  &:hover { background: rgba(48, 48, 52, 0.55); }
  &:disabled { opacity: 0.5; cursor: default; }
}

.create-lobby-info {
  margin-left: -2px;
}

.party-id-refresh {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 3px;
  border: none;
  background: none;
  color: rgba(255, 255, 255, 0.35);
  cursor: pointer;
  border-radius: 4px;
  user-select: none;
  transition: color 150ms, background 150ms;

  &:hover:not(:disabled) { color: rgba(255, 255, 255, 0.9); background: rgba(255, 255, 255, 0.08); }
  &:disabled { opacity: 0.4; cursor: default; }
}

// ── Ready button (non-leader) ─────────────────────────────────────────────────
.ready-btn {
  height: 46px;
  padding: 0 28px;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.6);
  font-family: $font-family;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.04em;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: background 150ms, border-color 150ms, color 150ms;
  backdrop-filter: blur(8px);

  &:hover { background: rgba(255, 255, 255, 0.1); color: #fff; }

  &--ready {
    background: rgba(52, 199, 89, 0.18);
    border-color: rgba(52, 199, 89, 0.5);
    color: #34c759;

    &:hover { background: rgba(52, 199, 89, 0.26); }
  }
}

// ── Friends panel ─────────────────────────────────────────────────────────────
.friends-panel {
  width: 318px;
  flex-shrink: 0;
  padding: 0 12px 12px 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.friends-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 14px 14px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
  animation: hub-fade 700ms ease 850ms both;
}

.friends-heading-row {
  display: flex;
  flex-direction: row;
  align-items: baseline;
  gap: 7px;
  min-width: 0;
}

.friends-sep {
  color: rgba(255, 255, 255, 0.35);
  font-size: 12px;
}

.friends-heading {
  display: inline-block;
  font-size: 17px;
  font-weight: 500;
  color: #fff;
  background: #090C20;
  border-radius: 4px;
  padding: 1px 6px;
  margin: 0 0 0 -6px;
  font-family: $font-family;
}

.friends-online-count {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.03em;
  color: #fff;
}

.online-dot {
  width: 6px;
  height: 6px;
  background: #fff;
  transform: rotate(45deg);
  flex-shrink: 0;
  transition: background 200ms ease, box-shadow 200ms ease;

  &.lit {
    background: #30d158;
    box-shadow: 0 0 5px rgba(48, 209, 88, 0.65);
  }
}

.satellite-link {
  display: flex;
  align-items: center;
  gap: 4px;
  background: none;
  border: none;
  padding: 0;
  font-size: 12px;
  font-weight: 600;
  color: $text-muted;
  cursor: pointer;
  white-space: nowrap;
  transition: color 150ms ease;

  &:hover { color: $text-primary; }
}

$friends-embed-bg: #0f0f11;

// ── Empty placeholder embed — same visual language as .friends-card ─────────
.empty-embed {
  flex: 1;
  min-height: 0;
  border-radius: var(--edr-radius, 4px);
  outline: var(--edr-outline, none);
  outline-offset: 3px;
  box-sizing: border-box;
  background: var(--edr-bg, $friends-embed-bg);
  border: 1px solid #262627;
  animation: hub-fade 900ms ease 1100ms both;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}

.maintenance-notice {
  display: flex;
  align-items: center;
  gap: 14px;
  max-width: 480px;
}

.maintenance-icon {
  flex-shrink: 0;
  color: $warning;
}

.maintenance-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.maintenance-title {
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  font-family: $font-family;
}

.maintenance-body {
  font-size: 12px;
  line-height: 1.5;
  color: rgba(255, 255, 255, 0.5);
  font-family: $font-family;
}

.friends-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  border-radius: var(--edr-radius, 4px);
  outline: var(--edr-outline, none);
  outline-offset: 3px;
  color: var(--edr-color, inherit);
  font-family: var(--edr-font, inherit);
  overflow: hidden;
  box-sizing: border-box;
  background: $friends-embed-bg;
  border: 1px solid #262627;
  animation: hub-slide-in 1300ms cubic-bezier(0.16, 1, 0.3, 1) 700ms both;
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

  &:hover {
    background: rgba(255, 255, 255, 0.05);

    .friend-avatar { transform: scale(1.06); }
  }
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
  border-radius: 4px;
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
