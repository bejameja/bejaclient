<template>
  <Transition name="invite-slide">
    <div v-if="visible" class="invite-backdrop" @click.self="$emit('close')">
      <div class="invite-panel">

        <!-- Header -->
        <div class="invite-header">
          <div class="invite-tabs">
            <button class="invite-tab" :class="{ active: tab === 'invite' }" @click="tab = 'invite'">Invite</button>
            <button class="invite-tab" :class="{ active: tab === 'join' }"   @click="tab = 'join'">Join by Code</button>
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
            <svg class="invite-search-icon" width="16" height="16" viewBox="0 0 16 16" fill="none">
              <circle cx="6.5" cy="6.5" r="4.5" stroke="currentColor" stroke-width="1.5"/>
              <line x1="10" y1="10" x2="14" y2="14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            <input v-model="search" class="invite-search" placeholder="Search friends..." autofocus />
          </div>

          <div v-if="onlineFriends.length > 0" class="invite-section-label">Online — {{ onlineFriends.length }}</div>

          <div class="invite-list">
            <div
              v-for="friend in filteredFriends"
              :key="friend.uuid"
              class="invite-item"
              :class="{ offline: !friend.online, invited: invitedSet.has(friend.uuid) }"
            >
              <div class="invite-avatar">{{ friend.username[0].toUpperCase() }}</div>
              <div class="invite-info">
                <div class="invite-name">{{ friend.username }}</div>
                <div class="invite-status">
                  <span class="invite-dot" :class="{ online: friend.online }" />
                  {{ friend.online ? 'Online' : 'Offline' }}
                </div>
              </div>
              <button
                class="invite-btn"
                :disabled="!friend.online || invitedSet.has(friend.uuid)"
                @click="invite(friend)"
              >
                {{ invitedSet.has(friend.uuid) ? 'Invited ✓' : 'Invite' }}
              </button>
            </div>

            <div v-if="filteredFriends.length === 0" class="invite-empty">
              <span v-if="search">No results for "{{ search }}"</span>
              <span v-else-if="friendsStore.friends.length === 0">No friends yet</span>
              <span v-else>All friends are offline</span>
            </div>
          </div>
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
@property --invite-angle {
  syntax: '<angle>';
  initial-value: 45deg;
  inherits: false;
}

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
  width: 360px;
  border-radius: $radius-lg;
  display: flex;
  flex-direction: column;
  max-height: calc(100vh - 92px);
  box-shadow:
    0 40px 100px rgba(0, 0, 0, 0.75),
    0 0 0 1px rgba(255, 255, 255, 0.04) inset;

  border: 1px solid transparent;
  background-image:
    linear-gradient(160deg, rgba(255,255,255,0.035) 0%, $surface-elevated 30%, $surface-elevated 100%),
    conic-gradient(
      from var(--invite-angle),
      rgba(255, 255, 255, 0.03) 0%,
      rgba(255, 255, 255, 0.5)  10%,
      rgba(255, 255, 255, 0.03) 20%,
      rgba(255, 255, 255, 0.03) 50%,
      rgba(255, 255, 255, 0.5)  60%,
      rgba(255, 255, 255, 0.03) 70%,
      rgba(255, 255, 255, 0.03) 100%
    );
  background-origin: border-box;
  background-clip: padding-box, border-box;
  transition: --invite-angle 500ms cubic-bezier(0.2, 0, 0, 1);
  overflow: hidden;

  &:hover { --invite-angle: 160deg; }
}

// ── Header / tabs ─────────────────────────────────────────────────────────────
.invite-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 18px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.invite-tabs {
  display: flex;
  gap: 4px;
}

.invite-tab {
  background: none;
  border: none;
  cursor: pointer;
  font-family: $font-family;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: $text-muted;
  padding: 8px 14px 12px;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: color 200ms, border-color 200ms;

  &.active {
    color: $text-primary;
    border-bottom-color: $accent;
  }
  &:not(.active):hover { color: $text-secondary; }
}

.invite-close {
  background: none;
  border: none;
  cursor: pointer;
  color: $text-muted;
  padding: 6px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  transition: color 200ms, background 200ms;
  margin-bottom: 8px;

  &:hover { color: $text-primary; background: rgba(255,255,255,0.07); }
}

// ── Search ────────────────────────────────────────────────────────────────────
.invite-search-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 14px 18px;
  padding: 9px 14px;
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 8px;
  transition: border-color 200ms;
  flex-shrink: 0;

  &:focus-within {
    border-color: rgba(62, 184, 255, 0.35);
    background: rgba(62, 184, 255, 0.04);
  }
}

.invite-search-icon { color: $text-muted; flex-shrink: 0; }

.invite-search {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 13px;
  color: $text-primary;
  font-family: $font-family;
  &::placeholder { color: rgba(255,255,255,0.22); }
}

// ── Section label ─────────────────────────────────────────────────────────────
.invite-section-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1.4px;
  text-transform: uppercase;
  color: $text-muted;
  padding: 4px 18px 8px;
}

// ── Friend list ───────────────────────────────────────────────────────────────
.invite-list {
  overflow-y: auto;
  flex: 1;
  scrollbar-width: thin;
  scrollbar-color: rgba(255,255,255,0.08) transparent;
  padding-bottom: 8px;
}

.invite-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  transition: background 150ms;

  &:last-child { border-bottom: none; }
  &:hover:not(.offline) { background: rgba(255,255,255,0.04); }
  &.offline { opacity: 0.4; }
}

.invite-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: linear-gradient(135deg, rgba(62,184,255,0.18), rgba(62,184,255,0.05));
  border: 1px solid rgba(62, 184, 255, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 800;
  color: $accent;
  flex-shrink: 0;
}

.invite-info { flex: 1; min-width: 0; }

.invite-name {
  font-size: 13px;
  font-weight: 600;
  color: $text-primary;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.invite-status {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  color: $text-muted;
  margin-top: 2px;
}

.invite-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: rgba(255,255,255,0.2);
  flex-shrink: 0;

  &.online { background: $success; box-shadow: 0 0 6px rgba(48, 209, 88, 0.7); }
}

.invite-btn {
  padding: 6px 16px;
  border-radius: $radius-sm;
  border: 1px solid rgba(255,255,255,0.12);
  background: rgba(255,255,255,0.05);
  color: $text-secondary;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  cursor: pointer;
  white-space: nowrap;
  transition: background 150ms, color 150ms, border-color 150ms;
  font-family: $font-family;

  &:not(:disabled):hover {
    background: rgba(62, 184, 255, 0.18);
    border-color: rgba(62, 184, 255, 0.5);
    color: $accent;
  }
  &:disabled { opacity: 0.35; cursor: default; }
}

.invite-empty {
  padding: 40px 20px;
  text-align: center;
  font-size: 13px;
  color: $text-muted;
  font-style: italic;
}

// ── Join by Code ─────────────────────────────────────────────────────────────
.join-body {
  padding: 32px 24px 28px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.join-hint {
  font-size: 12px;
  color: $text-muted;
  margin: 0;
  line-height: 1.6;
  text-align: center;
}

.join-input-row {
  display: flex;
  gap: 10px;
  width: 100%;
}

.join-code-input {
  flex: 1;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: $radius-sm;
  padding: 12px 16px;
  font-family: 'IBM Plex Mono', monospace;
  font-size: 20px;
  font-weight: 700;
  letter-spacing: 0.35em;
  color: $text-primary;
  text-transform: uppercase;
  outline: none;
  transition: border-color 200ms, box-shadow 200ms, background 200ms;
  text-align: center;

  &::placeholder { color: rgba(255,255,255,0.15); }
  &:focus {
    border-color: $accent;
    background: rgba(62, 184, 255, 0.05);
    box-shadow: 0 0 0 3px rgba(62, 184, 255, 0.12);
  }
}

.join-btn {
  padding: 12px 22px;
  border-radius: $radius-sm;
  border: 1px solid $accent;
  background: rgba(62, 184, 255, 0.15);
  color: $accent;
  font-family: $font-family;
  font-size: 12px;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  cursor: pointer;
  transition: background 150ms, box-shadow 150ms;
  white-space: nowrap;

  &:not(:disabled):hover {
    background: rgba(62, 184, 255, 0.28);
    box-shadow: 0 0 16px rgba(62, 184, 255, 0.25);
  }
  &:disabled { opacity: 0.35; cursor: default; }
}

.join-error   { font-size: 12px; color: #ff453a; margin: 0; text-align: center; }
.join-success { font-size: 12px; color: $success; margin: 0; text-align: center; }

// ── Slide transition ──────────────────────────────────────────────────────────
.invite-slide-enter-active,
.invite-slide-leave-active {
  transition: opacity 200ms ease, transform 200ms cubic-bezier(0.2, 0, 0, 1);
  .invite-panel { transition: transform 200ms cubic-bezier(0.2, 0, 0, 1); }
}
.invite-slide-enter-from { opacity: 0; .invite-panel { transform: translateX(16px) scale(0.98); } }
.invite-slide-leave-to   { opacity: 0; .invite-panel { transform: translateX(16px) scale(0.98); } }
</style>
