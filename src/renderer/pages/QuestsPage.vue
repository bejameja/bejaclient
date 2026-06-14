<template>
  <div class="quests-page">

    <!-- ── Quest board ─────────────────────────────────────────────────────── -->
    <div class="board-col">
      <div class="board-head">
        <h2 class="board-heading">{{ $t('quests.heading') }}</h2>
        <div class="week-tag">
          <span class="week-label">{{ $t('quests.weeklyLabel', { week: store.week || '—' }) }}</span>
          <span class="week-resets">{{ $t('quests.resets') }}</span>
        </div>
      </div>
      <div class="board-divider" />

      <div v-if="store.loading" class="state-area"><span class="spinner" /></div>
      <div v-else-if="!store.quests.length" class="state-area">
        <span class="state-text">{{ $t('quests.empty') }}</span>
      </div>

      <TransitionGroup v-else tag="div" name="quest-pop" class="quest-list" appear>
        <div
          v-for="(q, i) in store.quests"
          :key="q.id"
          class="quest-card"
          :class="{ complete: q.progress >= q.goal && !q.claimed, claimed: q.claimed }"
          :style="{ '--stagger': i * 60 + 'ms' }"
        >
          <div class="quest-main">
            <div class="quest-top">
              <span class="quest-name">{{ questName(q) }}</span>
              <span class="quest-xp">+{{ q.xp }} <span class="xp-unit">XP</span></span>
            </div>
            <div class="quest-bar">
              <div class="quest-bar-fill" :style="{ width: pct(q) + '%' }" />
            </div>
            <span class="quest-count">{{ q.progress }} / {{ q.goal }}</span>
          </div>

          <button
            class="claim-btn"
            :class="{ ready: q.progress >= q.goal && !q.claimed }"
            :disabled="q.claimed || q.progress < q.goal || claiming === q.id"
            @click="onClaim(q)"
          >
            <svg v-if="q.claimed" width="11" height="11" viewBox="0 0 10 10">
              <polyline points="1.5,5 4,7.5 8.5,2" stroke="currentColor" stroke-width="1.6" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
            <template v-else>{{ q.claimed ? $t('quests.claimed') : (q.progress >= q.goal ? $t('quests.claim') : $t('quests.locked')) }}</template>
          </button>
        </div>
      </TransitionGroup>

      <!-- +XP toast -->
      <Transition name="xp-float">
        <div v-if="store.lastAward" class="xp-toast">{{ $t('quests.awarded', { xp: store.lastAward }) }}</div>
      </Transition>
    </div>

    <!-- ── Leaderboard ─────────────────────────────────────────────────────── -->
    <div class="lb-col">
      <h2 class="lb-heading">{{ $t('quests.leaderboard') }}</h2>
      <span class="lb-rank-label">
        {{ store.myRank ? $t('quests.yourRank', { rank: store.myRank }) : $t('quests.unranked') }}
      </span>

      <div class="lb-card">
        <div v-if="store.lbLoading" class="state-area"><span class="spinner" /></div>
        <div v-else-if="!store.leaderboard.length" class="state-area">
          <span class="state-text">{{ $t('quests.lbEmpty') }}</span>
        </div>

        <template v-else>
          <div
            v-for="entry in store.leaderboard"
            :key="entry.uuid"
            class="lb-row"
            :class="[`rank-${entry.rank}`, { me: entry.uuid === myUuid }]"
          >
            <span class="lb-rank">{{ entry.rank }}</span>
            <div class="lb-head-wrap">
              <img
                class="lb-head"
                :src="`https://mc-heads.net/head/${entry.uuid}/64`"
                :alt="entry.username"
                @error="(e: Event) => ((e.target as HTMLImageElement).src = 'https://mc-heads.net/head/MHF_Steve/64')"
              />
            </div>
            <div class="lb-info">
              <span class="lb-name">{{ entry.username }}</span>
              <span class="lb-xp">{{ entry.xp.toLocaleString() }} XP</span>
            </div>
            <button class="lb-profile-btn" @click="openProfile(entry)">{{ $t('quests.profile') }}</button>
          </div>
        </template>
      </div>
    </div>

    <PlayerProfileModal v-model="profileOpen" :player="profilePlayer" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated } from 'vue'
import { useI18n } from 'vue-i18n'
import { useQuestsStore } from '../store/questsStore'
import { useAccountStore } from '../store/accountStore'
import { playClick } from '../composables/useSounds'
import PlayerProfileModal from '../components/friends/PlayerProfileModal.vue'
import type { Quest, LeaderboardEntry, PlayerProfile } from '../types'

const { t, te } = useI18n()
const store        = useQuestsStore()
const accountStore = useAccountStore()

const myUuid   = computed(() => accountStore.selectedAccount?.uuid ?? '')
const claiming = ref<string | null>(null)

const profileOpen   = ref(false)
const profilePlayer = ref<PlayerProfile | null>(null)

function questName(q: Quest): string {
  return te(`quests.names.${q.id}`) ? t(`quests.names.${q.id}`) : q.name
}

function pct(q: Quest): number {
  return Math.min(Math.round((q.progress / q.goal) * 100), 100)
}

async function onClaim(q: Quest) {
  if (claiming.value) return
  claiming.value = q.id
  try {
    const res = await store.claim(q.id)
    if (res?.awarded) playClick()
  } finally {
    claiming.value = null
  }
}

async function openProfile(entry: LeaderboardEntry) {
  // Mojang lookup gives skin/cape for the modal; cracked accounts fall back to basics
  const profile = await window.api.players.lookup(entry.username)
  profilePlayer.value = profile ?? {
    uuid: entry.uuid,
    username: entry.username,
    skinUrl: null,
    capeUrl: null,
    skinModel: 'default',
  }
  profileOpen.value = true
}

function refresh() {
  store.load()
  store.loadLeaderboard()
}

onMounted(refresh)
onActivated(refresh)
</script>

<style lang="scss" scoped>
.quests-page {
  height: 100%;
  display: flex;
  flex-direction: row;
  overflow: hidden;
}

// ── Board column ──────────────────────────────────────────────────────────────
.board-col {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  padding: 14px;
  gap: 0;
  position: relative;
}

.board-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
}

.board-heading {
  font-size: 30px;
  font-weight: 900;
  color: $text-primary;
  margin: 2px 0 8px;
  font-family: 'Plus Jakarta Sans', sans-serif;
}

.week-tag {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
}

.week-label {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  color: #555;
  letter-spacing: 0.12em;
}

.week-resets {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  color: #333;
  letter-spacing: 0.12em;
}

.board-divider {
  height: 1px;
  background: $border;
  margin-bottom: 12px;
  flex-shrink: 0;
}

// ── Quest list ────────────────────────────────────────────────────────────────
.quest-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-bottom: 10px;
  scrollbar-width: thin;
  scrollbar-color: #282828 transparent;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: #282828; }
}

// Animated conic border — same language as friend/news cards
@property --qc-angle {
  syntax: '<angle>';
  initial-value: 45deg;
  inherits: false;
}

.quest-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 16px;
  border-radius: 8px;
  border: 1px solid transparent;
  background-image:
    linear-gradient(#111, #111),
    conic-gradient(
      from var(--qc-angle),
      rgba(255,255,255,.04) 0%,
      rgba(255,255,255,.45) 18%,
      rgba(255,255,255,.04) 36%,
      rgba(255,255,255,.04) 100%
    );
  background-origin: border-box;
  background-clip: padding-box, border-box;
  transition: --qc-angle 500ms cubic-bezier(.2,0,0,1), transform 300ms cubic-bezier(.2,0,0,1);

  &:hover {
    --qc-angle: 135deg;
    transform: translateY(-2px);
  }

  // Claimable card glows with the accent
  &.complete {
    background-image:
      linear-gradient(#111, #111),
      conic-gradient(
        from var(--qc-angle),
        rgba(62,184,255,.10) 0%,
        rgba(62,184,255,.65) 18%,
        rgba(62,184,255,.10) 36%,
        rgba(62,184,255,.10) 100%
      );
    box-shadow: 0 0 18px rgba(62,184,255,0.07);
  }

  &.claimed {
    opacity: 0.45;
    filter: saturate(0.4);
    .quest-name { text-decoration: line-through; }
  }
}

.quest-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.quest-top {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
}

.quest-name {
  font-size: 14px;
  font-weight: 700;
  color: $text-primary;
  font-family: 'Plus Jakarta Sans', sans-serif;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.quest-xp {
  font-size: 13px;
  font-weight: 800;
  color: $accent;
  flex-shrink: 0;
}

.xp-unit {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  letter-spacing: 0.1em;
}

.quest-bar {
  height: 4px;
  background: rgba(255,255,255,0.06);
  border-radius: 2px;
  overflow: hidden;
}

.quest-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, rgba(62,184,255,0.5), $accent);
  border-radius: 2px;
  transition: width 600ms cubic-bezier(.2,0,0,1);
}

.quest-count {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  color: #555;
  letter-spacing: 0.12em;
}

// ── Claim button ──────────────────────────────────────────────────────────────
.claim-btn {
  flex-shrink: 0;
  min-width: 96px;
  padding: 9px 14px;
  border-radius: 6px;
  border: 1px solid $border;
  background: $surface-elevated;
  color: #555;
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  letter-spacing: 0.12em;
  cursor: not-allowed;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background $transition, color $transition, box-shadow $transition, transform $transition-fast;

  &.ready {
    background: $accent;
    border-color: transparent;
    color: #06121c;
    cursor: pointer;
    box-shadow: 0 0 14px rgba(62,184,255,0.35);
    animation: claimPulse 1.8s ease-in-out infinite;

    &:hover  { background: $primary-hover; transform: scale(1.04); }
    &:active { transform: scale(0.97); }
  }

  &:disabled:not(.ready) { opacity: 0.8; }
}

@keyframes claimPulse {
  0%, 100% { box-shadow: 0 0 10px rgba(62,184,255,0.25); }
  50%       { box-shadow: 0 0 20px rgba(62,184,255,0.50); }
}

// ── +XP toast ─────────────────────────────────────────────────────────────────
.xp-toast {
  position: absolute;
  bottom: 28px;
  left: 50%;
  transform: translateX(-50%);
  padding: 10px 22px;
  border-radius: 8px;
  background: #111;
  border: 1px solid rgba(62,184,255,0.4);
  color: $accent;
  font-size: 16px;
  font-weight: 800;
  font-family: 'Plus Jakarta Sans', sans-serif;
  box-shadow: 0 0 24px rgba(62,184,255,0.25), $shadow-lg;
  pointer-events: none;
  z-index: 10;
}

.xp-float-enter-active { transition: opacity 250ms ease, transform 350ms cubic-bezier(.2,0,0,1); }
.xp-float-leave-active { transition: opacity 500ms ease, transform 600ms ease; }
.xp-float-enter-from   { opacity: 0; transform: translateX(-50%) translateY(16px) scale(0.9); }
.xp-float-leave-to     { opacity: 0; transform: translateX(-50%) translateY(-24px); }

// Quest card stagger-in
.quest-pop-enter-active { transition: opacity 400ms ease var(--stagger), transform 450ms cubic-bezier(.2,0,0,1) var(--stagger); }
.quest-pop-enter-from   { opacity: 0; transform: translateY(12px); }

// ── Leaderboard column ────────────────────────────────────────────────────────
.lb-col {
  width: 300px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  padding: 14px 14px 14px 0;
}

.lb-heading {
  font-size: 20px;
  font-weight: 800;
  color: $text-primary;
  margin: 8px 0 2px;
  font-family: 'Plus Jakarta Sans', sans-serif;
}

.lb-rank-label {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  color: #555;
  letter-spacing: 0.12em;
  margin-bottom: 10px;
}

.lb-card {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  border-radius: 8px;
  border: 1px solid $border;
  background: #111;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 3px;
  scrollbar-width: thin;
  scrollbar-color: #282828 transparent;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: #282828; }
}

.lb-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 8px;
  border-radius: 6px;
  border: 1px solid transparent;
  transition: background $transition, border-color $transition;

  &:hover {
    background: rgba(255,255,255,0.03);
    .lb-profile-btn { opacity: 1; transform: translateX(0); }
  }

  &.me {
    border-color: rgba(62,184,255,0.25);
    background: rgba(62,184,255,0.05);
  }

  // Podium rims on the head
  &.rank-1 .lb-head-wrap { box-shadow: 0 0 0 2px #e8c25a, 0 0 10px rgba(232,194,90,0.4); }
  &.rank-2 .lb-head-wrap { box-shadow: 0 0 0 2px #b8c0cc, 0 0 8px rgba(184,192,204,0.3); }
  &.rank-3 .lb-head-wrap { box-shadow: 0 0 0 2px #c08a5a, 0 0 8px rgba(192,138,90,0.3); }
  &.rank-1 .lb-rank { color: #e8c25a; }
  &.rank-2 .lb-rank { color: #b8c0cc; }
  &.rank-3 .lb-rank { color: #c08a5a; }
}

.lb-rank {
  width: 20px;
  flex-shrink: 0;
  text-align: center;
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  color: #555;
}

.lb-head-wrap {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  border-radius: 3px;
  overflow: hidden;
  background: rgba(0,0,0,0.4);
}

.lb-head {
  width: 100%;
  height: 100%;
  display: block;
  image-rendering: pixelated;
}

.lb-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.lb-name {
  font-size: 13px;
  font-weight: 600;
  color: $text-primary;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.lb-xp {
  font-size: 10px;
  font-weight: 700;
  color: $accent;
}

.lb-profile-btn {
  flex-shrink: 0;
  opacity: 0;
  transform: translateX(4px);
  padding: 4px 9px;
  border-radius: 5px;
  border: 1px solid $border;
  background: $surface-elevated;
  color: $text-secondary;
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  letter-spacing: 0.1em;
  cursor: pointer;
  transition: opacity $transition, transform $transition, background $transition, color $transition;

  &:hover { background: $border; color: $text-primary; }
}

// ── Shared states ─────────────────────────────────────────────────────────────
.state-area {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 120px;
}

.state-text {
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  color: #333;
  letter-spacing: 0.14em;
}

.spinner {
  width: 22px;
  height: 22px;
  border: 2px solid $border;
  border-top-color: $text-secondary;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
