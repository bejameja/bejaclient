<template>
  <div class="friends-page page-content">

    <div class="page-header">
      <h1 class="page-title">Friends</h1>
      <button class="add-btn" @click="showAdd = !showAdd">
        <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        Add Friend
      </button>
    </div>

    <!-- Add friend form -->
    <Transition name="slide-down">
      <div v-if="showAdd" class="add-card">
        <p class="add-label">Search by Minecraft gamertag</p>
        <div class="add-row">
          <input
            ref="inputEl"
            v-model="gamertag"
            class="add-input"
            placeholder="Enter gamertag…"
            maxlength="32"
            @keydown.enter="send"
            @keydown.escape="showAdd = false"
          />
          <button class="send-btn" :disabled="!gamertag.trim()" @click="send">Send Request</button>
        </div>
        <Transition name="fade">
          <p v-if="feedback" class="feedback" :class="feedbackType">{{ feedback }}</p>
        </Transition>
      </div>
    </Transition>

    <!-- Incoming requests -->
    <template v-if="store.incomingRequests.length > 0">
      <h2 class="section-title">
        Incoming Requests
        <span class="badge">{{ store.incomingRequests.length }}</span>
      </h2>
      <div class="card-list">
        <div v-for="req in store.incomingRequests" :key="req.id" class="friend-card">
          <div class="avatar">{{ req.gamertag[0].toUpperCase() }}</div>
          <div class="info">
            <span class="name">{{ req.gamertag }}</span>
            <span class="sub">Wants to be your friend</span>
          </div>
          <div class="actions">
            <button class="action-btn accept" @click="store.acceptRequest(req.id)">Accept</button>
            <button class="action-btn decline" @click="store.declineRequest(req.id)">Decline</button>
          </div>
        </div>
      </div>
    </template>

    <!-- Outgoing requests -->
    <template v-if="store.outgoingRequests.length > 0">
      <h2 class="section-title">Sent Requests</h2>
      <div class="card-list">
        <div v-for="req in store.outgoingRequests" :key="req.id" class="friend-card">
          <div class="avatar">{{ req.gamertag[0].toUpperCase() }}</div>
          <div class="info">
            <span class="name">{{ req.gamertag }}</span>
            <span class="sub">Pending…</span>
          </div>
          <button class="action-btn decline" @click="store.cancelRequest(req.id)">Cancel</button>
        </div>
      </div>
    </template>

    <!-- All friends -->
    <h2 class="section-title">
      All Friends
      <span class="count">{{ store.friends.length }}</span>
    </h2>

    <div v-if="store.friends.length === 0" class="empty-state">
      <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
        <circle cx="9" cy="7" r="4"/>
        <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
        <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
      </svg>
      <span class="empty-title">No friends yet</span>
      <span class="empty-text">Add a friend by searching their Minecraft gamertag above.</span>
    </div>

    <div v-else class="card-list">
      <div v-for="friend in store.friends" :key="friend.id" class="friend-card">
        <div class="avatar">{{ friend.gamertag[0].toUpperCase() }}</div>
        <div class="info">
          <span class="name">{{ friend.gamertag }}</span>
          <span class="sub offline">Offline</span>
        </div>
        <button class="action-btn decline" @click="store.removeFriend(friend.id)">Remove</button>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useFriendsStore } from '../store/friendsStore'

const store    = useFriendsStore()
const showAdd  = ref(false)
const gamertag = ref('')
const feedback = ref('')
const feedbackType = ref<'ok' | 'err'>('ok')
const inputEl  = ref<HTMLInputElement | null>(null)

let feedbackTimer: ReturnType<typeof setTimeout>

watch(showAdd, (v) => { if (v) nextTick(() => inputEl.value?.focus()) })

function send() {
  const tag = gamertag.value.trim()
  if (!tag) return
  const result = store.sendRequest(tag)
  clearTimeout(feedbackTimer)
  if (result === 'sent') {
    feedbackType.value = 'ok'
    feedback.value = `Friend request sent to ${tag}!`
    gamertag.value = ''
  } else if (result === 'already_friends') {
    feedbackType.value = 'err'
    feedback.value = `You're already friends with ${tag}.`
  } else {
    feedbackType.value = 'err'
    feedback.value = 'A request to this player is already pending.'
  }
  feedbackTimer = setTimeout(() => { feedback.value = '' }, 4000)
}
</script>

<style lang="scss" scoped>
.friends-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow-y: auto;
  padding: 28px 32px;
  gap: 0;

  &::-webkit-scrollbar { width: 5px; }
  &::-webkit-scrollbar-track { background: transparent; }
  &::-webkit-scrollbar-thumb { background: $border-strong; border-radius: 4px; }
}

// ── Header ─────────────────────────────────────────────────────────────────────
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.page-title {
  font-size: 20px;
  font-weight: 700;
  color: $text-primary;
}

.add-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  background: $text-primary;
  color: $bg;
  border: none;
  border-radius: $radius;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background $transition;
  &:hover { background: $text-secondary; }
}

// ── Add card ──────────────────────────────────────────────────────────────────
.add-card {
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius-lg;
  padding: 16px 18px;
  margin-bottom: 24px;
}

.add-label {
  font-size: 12px;
  color: $text-secondary;
  margin-bottom: 10px;
}

.add-row {
  display: flex;
  gap: 8px;
}

.add-input {
  flex: 1;
  background: $bg;
  border: 1px solid $border-strong;
  border-radius: $radius;
  padding: 8px 12px;
  color: $text-primary;
  font-size: 13px;
  outline: none;
  transition: border-color $transition;

  &::placeholder { color: $muted; }
  &:focus { border-color: $primary; }
}

.send-btn {
  padding: 8px 16px;
  background: $text-primary;
  color: $bg;
  border: none;
  border-radius: $radius;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: background $transition;

  &:hover:not(:disabled) { background: $text-secondary; }
  &:disabled { opacity: 0.4; cursor: not-allowed; }
}

.feedback {
  margin-top: 8px;
  font-size: 12px;
  &.ok  { color: $success; }
  &.err { color: $error; }
}

// ── Sections ──────────────────────────────────────────────────────────────────
.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 600;
  color: $text-secondary;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin: 20px 0 10px;
}

.badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  background: $border-strong;
  color: $text-primary;
  font-size: 10px;
  font-weight: 700;
  border-radius: 9px;
}

.count {
  font-size: 11px;
  color: $muted;
  font-weight: 400;
}

// ── Friend cards ──────────────────────────────────────────────────────────────
.card-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.friend-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius-lg;
  transition: border-color $transition;

  &:hover { border-color: $border-strong; }
}

.avatar {
  width: 36px;
  height: 36px;
  border-radius: 9px;
  background: $surface-elevated;
  border: 1px solid $border;
  color: $text-primary;
  font-size: 15px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.name {
  font-size: 13px;
  font-weight: 600;
  color: $text-primary;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sub {
  font-size: 11px;
  color: $muted;
  &.offline { color: $muted; }
}

.actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.action-btn {
  padding: 5px 12px;
  border-radius: $radius;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  transition: background $transition, color $transition;

  &.accept {
    background: $surface-elevated;
    border-color: $border;
    color: $text-primary;
    &:hover { background: $border; }
  }

  &.decline {
    background: $surface-elevated;
    border-color: $border;
    color: $text-secondary;
    &:hover { background: $border; }
  }
}

// ── Empty state ───────────────────────────────────────────────────────────────
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 48px 0;
  color: $muted;
}

.empty-title {
  font-size: 14px;
  font-weight: 600;
  color: $text-secondary;
}

.empty-text {
  font-size: 12px;
  color: $muted;
  text-align: center;
  max-width: 260px;
}

// ── Transitions ───────────────────────────────────────────────────────────────
.slide-down-enter-active, .slide-down-leave-active { transition: opacity 200ms, transform 200ms; }
.slide-down-enter-from, .slide-down-leave-to       { opacity: 0; transform: translateY(-8px); }

.fade-enter-active, .fade-leave-active { transition: opacity 200ms; }
.fade-enter-from, .fade-leave-to       { opacity: 0; }
</style>
