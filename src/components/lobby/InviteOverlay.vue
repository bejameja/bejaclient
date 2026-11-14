<template>
  <Transition name="invite-slide">
    <div v-if="visible" class="invite-backdrop" @click.self="$emit('close')">
      <div class="invite-panel">

        <!-- Header -->
        <div class="invite-header">
          <div class="invite-tabs">
            <button class="invite-tab" :class="{ active: tab === 'invite' }" @click="tab = 'invite'"><span class="invite-word invite-word--blue">Friends</span></button>
            <button class="invite-tab" :class="{ active: tab === 'join' }"   @click="tab = 'join'"><span class="invite-word invite-word--green">Code</span></button>
          </div>
          <button class="invite-close" @click="$emit('close')">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <line x1="2" y1="2" x2="14" y2="14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <line x1="14" y1="2" x2="2"  y2="14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
        </div>

        <!-- ── Invite Friends tab ─────────────────────────────────────────── -->
        <template v-if="tab === 'invite'">
          <div class="invite-search-wrap">
            <svg class="invite-search-icon" width="15" height="15" viewBox="0 0 16 16" fill="none">
              <circle cx="6.5" cy="6.5" r="4.5" stroke="currentColor" stroke-width="1.5"/>
              <line x1="10" y1="10" x2="14" y2="14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            <input v-model="search" class="invite-search" placeholder="Search friends..." autofocus />
          </div>

          <div v-if="onlineFriends.length > 0" class="invite-section-label">Online <span class="invite-count">{{ onlineFriends.length }}</span></div>

          <TransitionGroup tag="div" name="invite-item-pop" class="invite-list">
            <div
              v-for="friend in filteredFriends"
              :key="friend.uuid"
              class="invite-item"
              :class="{ offline: !friend.online, invited: invitedSet.has(friend.uuid) }"
            >
              <div class="invite-avatar-wrap">
                <img
                  class="invite-avatar"
                  :src="`https://mc-heads.net/head/${friend.uuid}/64`"
                  alt=""
                  @error="(e: Event) => ((e.target as HTMLImageElement).src = 'https://mc-heads.net/head/MHF_Steve/64')"
                />
                <span class="invite-dot" :class="{ online: friend.online }" />
              </div>
              <div class="invite-info">
                <div class="invite-name">{{ friend.username }}</div>
                <div class="invite-status">{{ friend.online ? 'Online' : 'Offline' }}</div>
              </div>
              <button
                class="invite-btn"
                :class="{ done: invitedSet.has(friend.uuid) }"
                :disabled="!friend.online || invitedSet.has(friend.uuid)"
                @click="invite(friend)"
              >
                {{ invitedSet.has(friend.uuid) ? 'Invited' : 'Invite' }}
              </button>
            </div>

            <div v-if="filteredFriends.length === 0" key="__empty" class="invite-empty">
              <span v-if="search">No results for "{{ search }}"</span>
              <span v-else-if="friendsStore.friends.length === 0">No friends yet</span>
              <span v-else>All friends are offline</span>
            </div>
          </TransitionGroup>
        </template>

        <!-- ── Join by Code tab ──────────────────────────────────────────── -->
        <template v-else>
          <div class="join-body">
            <p class="join-hint">Enter the 6-character party code shared by your friend.</p>

            <div class="join-input-row">
              <input
                v-model="joinCode"
                class="join-code-input"
                placeholder="XXXXXX"
                maxlength="6"
                spellcheck="false"
                autofocus
                @keydown.enter="doJoin"
                @input="joinCode = (joinCode as string).toUpperCase()"
              />
              <button class="join-btn" :disabled="joinCode.length < 6 || joining" @click="doJoin">
                {{ joining ? '…' : 'Join' }}
              </button>
            </div>

            <p v-if="joinError" class="join-error">{{ joinError }}</p>
            <p v-if="joinSuccess" class="join-success">Joined party!</p>
          </div>
        </template>

      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useFriendsStore } from '../../store/friendsStore'
import { useLobbyStore }   from '../../store/lobbyStore'

const emit = defineEmits<{ close: [] }>()
const props = defineProps<{ visible: boolean; initialTab?: 'invite' | 'join' }>()

const friendsStore = useFriendsStore()
const lobbyStore   = useLobbyStore()

// ── Tab state ─────────────────────────────────────────────────────────────────
const tab = ref<'invite' | 'join'>(props.initialTab ?? 'invite')
watch(() => props.visible, v => { if (v) tab.value = props.initialTab ?? 'invite' })

// ── Invite tab ────────────────────────────────────────────────────────────────
const search     = ref('')
const invitedSet = ref<Set<string>>(new Set())

const onlineFriends = computed(() => friendsStore.friends.filter(f => f.online))

const filteredFriends = computed(() => {
  const q = search.value.trim().toLowerCase()
  return friendsStore.friends
    .filter(f => !q || f.username.toLowerCase().includes(q))
    .sort((a, b) => (b.online ? 1 : 0) - (a.online ? 1 : 0))
})

function invite(friend: { uuid: string; username: string }): void {
  invitedSet.value.add(friend.uuid)
  lobbyStore.inviteFriend(friend.uuid)
}

// ── Join tab ──────────────────────────────────────────────────────────────────
const joinCode    = ref('')
const joining     = ref(false)
const joinError   = ref('')
const joinSuccess = ref(false)

async function doJoin(): Promise<void> {
  joinError.value   = ''
  joinSuccess.value = false
  joining.value     = true
  const result = await lobbyStore.joinParty(joinCode.value)
  joining.value = false
  if (result.ok) {
    joinSuccess.value = true
    joinCode.value = ''
    setTimeout(() => emit('close'), 800)
  } else {
    joinError.value = result.error ?? 'Failed to join'
  }
}
</script>

<style lang="scss" scoped>
// Same look as the Social page: near-black surfaces, hairline borders, pill toggles, white primary buttons and the
// coloured Minecrafter words. Corners stay at the launcher's $radius-sm (12px on the pill buttons).

// ── Backdrop ──────────────────────────────────────────────────────────────────
.invite-backdrop {
  position: fixed;
  inset: 0;
  z-index: 500;
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
  padding: 72px 20px 0;
  background: rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(8px);
}

// ── Panel ─────────────────────────────────────────────────────────────────────
.invite-panel {
  width: 340px;
  border-radius: $radius-sm;
  display: flex;
  flex-direction: column;
  max-height: calc(100vh - 92px);
  background: #0a0a0b;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.6);
  overflow: hidden;
  padding: 16px;
  gap: 14px;
}

// ── Header / tabs ─────────────────────────────────────────────────────────────
.invite-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}

.invite-tabs {
  display: flex;
  gap: 5px;
}

.invite-tab {
  padding: 8px 10px;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 12px;
  color: rgba(255, 255, 255, 0.55);
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.03em;
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease, border-color 140ms ease;

  &:hover { color: rgba(255, 255, 255, 0.85); border-color: rgba(255, 255, 255, 0.3); }

  &.active {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.3);
    color: #fff;
  }
}

// the Minecrafter highlight words (same colours as the Social tabs)
.invite-word {
  display: inline-block;
  font-size: 11px;
  font-weight: 800;
  border-radius: $radius-sm;
  padding: 1px 6px;
  font-family: 'Minecrafter', 'Plus Jakarta Sans', sans-serif;
  letter-spacing: 0.08em;

  &--blue { color: #4660FE; background: #090C20; }
  &--green { color: #8BE28B; background: #0E1A10; }
}

.invite-close {
  background: none;
  border: none;
  cursor: pointer;
  color: rgba(255, 255, 255, 0.45);
  padding: 6px;
  border-radius: $radius-sm;
  display: flex;
  align-items: center;
  transition: color 140ms ease, background 140ms ease;

  &:hover { color: #fff; background: rgba(255, 255, 255, 0.07); }
}

// ── Search ────────────────────────────────────────────────────────────────────
.invite-search-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 38px;
  padding: 0 10px;
  background: #131315;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: $radius-sm;
  transition: border-color 140ms ease;
  flex-shrink: 0;

  &:focus-within { border-color: rgba(255, 255, 255, 0.25); }
}

.invite-search-icon { color: rgba(255, 255, 255, 0.5); flex-shrink: 0; }

.invite-search {
  flex: 1;
  min-width: 0;
  background: none;
  border: none;
  outline: none;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 12.5px;
  color: rgba(255, 255, 255, 0.85);
  &::placeholder { color: rgba(255, 255, 255, 0.35); }
}

// ── Section label ─────────────────────────────────────────────────────────────
.invite-section-label {
  font-size: 15px;
  font-weight: 800;
  color: #fff;
  font-family: 'Plus Jakarta Sans', sans-serif;
  flex-shrink: 0;
  margin-bottom: -6px;
}

.invite-count {
  margin-left: 4px;
  font-size: 11px;
  font-weight: 800;
  color: rgba(255, 255, 255, 0.4);
}

// ── Friend list ───────────────────────────────────────────────────────────────
.invite-list {
  overflow-y: auto;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  scrollbar-width: none;
  &::-webkit-scrollbar { display: none; }
}

.invite-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  margin: 0 -8px;
  border-radius: $radius-sm;

  // name dims for offline friends, like the Social list
  .invite-name { color: #fff; transition: color 200ms ease; }
  &.offline .invite-name { color: rgba(255, 255, 255, 0.22); }
}

.invite-avatar-wrap {
  position: relative;
  flex-shrink: 0;
  width: 32px;
  height: 32px;
}

.invite-avatar {
  width: 32px;
  height: 32px;
  border-radius: $radius-sm;
  image-rendering: pixelated;
  display: block;
}

.invite-dot {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #3a3a3f;
  border: 2px solid #0a0a0b;

  &.online { background: #30d158; box-shadow: 0 0 5px rgba(48, 209, 88, 0.65); }
}

.invite-info { flex: 1; min-width: 0; }

.invite-name {
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.invite-status {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  margin-top: 1px;
}

.invite-btn {
  flex-shrink: 0;
  padding: 6px 14px;
  border: none;
  border-radius: $radius-sm;
  background: #fff;
  color: #0a0a0b;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.06em;
  cursor: pointer;
  white-space: nowrap;
  transition: background 140ms ease, opacity 140ms ease, color 140ms ease;

  &:hover:not(:disabled) { background: rgba(255, 255, 255, 0.8); }
  &:disabled { opacity: 0.3; cursor: default; }

  // sent: quiet outline, like the cancel buttons of the request list
  &.done {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.18);
    color: rgba(255, 255, 255, 0.55);
    opacity: 1;
  }
}

.invite-empty {
  padding: 24px 4px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.35);
  font-style: italic;
}

// ── Join by Code ─────────────────────────────────────────────────────────────
.join-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.join-hint {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  margin: 0;
  line-height: 1.7;
}

.join-input-row {
  display: flex;
  gap: 8px;
  width: 100%;
}

.join-code-input {
  flex: 1;
  min-width: 0;
  height: 44px;
  background: #131315;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: $radius-sm;
  padding: 0 14px;
  font-family: 'Minecrafter', 'IBM Plex Mono', monospace;
  font-size: 18px;
  letter-spacing: 0.3em;
  color: #fff;
  text-transform: uppercase;
  text-align: center;
  outline: none;
  transition: border-color 140ms ease;

  &::placeholder { color: rgba(255, 255, 255, 0.15); }
  &:focus { border-color: rgba(255, 255, 255, 0.25); }
}

.join-btn {
  flex-shrink: 0;
  height: 44px;
  padding: 0 18px;
  border: none;
  border-radius: $radius-sm;
  background: #fff;
  color: #0a0a0b;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.06em;
  cursor: pointer;
  white-space: nowrap;
  transition: background 140ms ease, opacity 140ms ease;

  &:hover:not(:disabled) { background: rgba(255, 255, 255, 0.8); }
  &:disabled { opacity: 0.35; cursor: default; }
}

.join-error   { font-size: 12px; color: #f87171; margin: 0; }
.join-success { font-size: 12px; color: #30d158; margin: 0; }

// ── Slide transition ──────────────────────────────────────────────────────────
.invite-slide-enter-active,
.invite-slide-leave-active {
  transition: opacity 200ms ease;
  .invite-panel { transition: transform 200ms cubic-bezier(0.2, 0, 0, 1); }
}
.invite-slide-enter-from { opacity: 0; .invite-panel { transform: translateX(16px); } }
.invite-slide-leave-to   { opacity: 0; .invite-panel { transform: translateX(16px); } }

// Invite row enter/leave/reorder (search filtering, online/offline sort)
.invite-item-pop-enter-active,
.invite-item-pop-leave-active {
  transition: opacity 180ms ease, transform 180ms cubic-bezier(0.25, 1, 0.5, 1);
}
.invite-item-pop-move { transition: transform 220ms cubic-bezier(0.25, 1, 0.5, 1); }
.invite-item-pop-enter-from { opacity: 0; transform: translateY(-6px); }
.invite-item-pop-leave-to   { opacity: 0; }
</style>
