<template>
  <Teleport to="#toast-dock">
    <div class="toast-stack" aria-live="polite">
      <TransitionGroup name="toast-pop">
        <div
          v-for="toast in activeToasts"
          :key="toast.id"
          class="toast-card"
        >
          <!-- Avatar -->
          <img class="toast-avatar" :src="`https://mc-heads.net/head/${toast.uuid}/80`" alt="" @error="(e: Event) => ((e.target as HTMLImageElement).src = 'https://mc-heads.net/head/MHF_Steve/80')" />

          <!-- Info -->
          <div class="toast-info">
            <span class="toast-word toast-word--blue">Friend Request</span>
            <div class="toast-name">{{ toast.username }}</div>
            <div v-if="toast.id === activeToasts[0]?.id && !lobbyStore.pendingInvite" class="toast-hint">Hold Esc to accept</div>
          </div>

          <!-- Buttons -->
          <div class="toast-btns">
            <button class="toast-accept" :class="{ holding: holding && toast.id === activeToasts[0]?.id }" @click="accept(toast)">Accept</button>
            <button class="toast-decline" @click="dismiss(toast.id)"><svg width="12" height="12" viewBox="0 0 16 16" fill="none"><line x1="3" y1="3" x2="13" y2="13" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><line x1="13" y1="3" x2="3" y2="13" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg></button>
          </div>

          <!-- Progress bar auto-dismiss indicator -->
          <div class="toast-bar" :style="{ animationDuration: TOAST_MS + 'ms' }" />
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { useNotificationsStore } from '../../store/notificationsStore'
import { useFriendsStore } from '../../store/friendsStore'
import { useLobbyStore } from '../../store/lobbyStore'

const TOAST_MS = 8000
const HOLD_MS = 700

interface Toast {
  id: string
  uuid: string
  username: string
  timerId: ReturnType<typeof setTimeout>
}

const notifStore   = useNotificationsStore()
const friendsStore = useFriendsStore()
const lobbyStore   = useLobbyStore()

const activeToasts = ref<Toast[]>([])
const seenIds      = new Set<string>()

// Watch notifications for new friend_request entries
watch(
  () => notifStore.notifications,
  (notifications) => {
    for (const n of notifications) {
      if (n.type !== 'friend_request' || seenIds.has(n.id)) continue
      if (!n.data?.uuid || !n.data?.username) continue
      seenIds.add(n.id)
      addToast(n.id, n.data.uuid, n.data.username)
    }
  },
  { deep: true },
)

function addToast(id: string, uuid: string, username: string): void {
  // Cap visible toasts at 3
  if (activeToasts.value.length >= 3) {
    const oldest = activeToasts.value[0]
    clearTimeout(oldest.timerId)
    activeToasts.value.shift()
  }

  const timerId = setTimeout(() => dismiss(id), TOAST_MS)
  activeToasts.value.push({ id, uuid, username, timerId })
}

function dismiss(id: string): void {
  const idx = activeToasts.value.findIndex(t => t.id === id)
  if (idx === -1) return
  clearTimeout(activeToasts.value[idx].timerId)
  activeToasts.value.splice(idx, 1)
}

async function accept(toast: Toast): Promise<void> {
  await friendsStore.acceptRequest(toast.uuid)
  notifStore.remove(toast.id)
  dismiss(toast.id)
}

// ── Hold Esc to accept the oldest request (a party invite on screen goes first) ──
const holding = ref(false)
let holdTimer: ReturnType<typeof setTimeout> | null = null

function stopHold(): void {
  holding.value = false
  if (holdTimer) { clearTimeout(holdTimer); holdTimer = null }
}

function onKeyDown(e: KeyboardEvent): void {
  if (e.key !== 'Escape' || e.repeat || holdTimer) return
  if (lobbyStore.pendingInvite || !activeToasts.value.length) return
  holding.value = true
  holdTimer = setTimeout(() => {
    stopHold()
    const first = activeToasts.value[0]
    if (first) void accept(first)
  }, HOLD_MS)
}
function onKeyUp(e: KeyboardEvent): void { if (e.key === 'Escape') stopHold() }

onMounted(() => { window.addEventListener('keydown', onKeyDown); window.addEventListener('keyup', onKeyUp) })
onBeforeUnmount(() => { window.removeEventListener('keydown', onKeyDown); window.removeEventListener('keyup', onKeyUp); stopHold() })
</script>

<style lang="scss" scoped>
.toast-stack {
  display: flex;
  flex-direction: column-reverse;
  gap: 8px;
  pointer-events: none;
}

// Same look as the Social page: near-black surface, hairline border, 12px corners, white primary pill.
.toast-card {
  pointer-events: all;
  display: flex;
  align-items: center;
  gap: 12px;
  width: 340px;
  padding: 12px 12px 14px;
  background: #13141A;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.55);
  position: relative;
  overflow: hidden;
}

.toast-avatar {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  background: #232328;
  image-rendering: pixelated;
  flex-shrink: 0;
  display: block;
}

.toast-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 5px;
}

// the Minecrafter highlight word (same colours as the Social tabs)
.toast-word {
  display: inline-block;
  padding: 1px 6px;
  border-radius: 8px;
  font-family: 'Minecrafter', 'Plus Jakarta Sans', sans-serif;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.08em;
  line-height: 1.5;
  white-space: nowrap;

  &--blue { color: #4660FE; background: #090C20; }
  &--green { color: #8BE28B; background: #0E1A10; }
}

.toast-name {
  max-width: 100%;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.toast-btns {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.toast-accept {
  height: 30px;
  padding: 0 14px;
  border-radius: 12px;
  border: none;
  background: #fff;
  color: #0a0a0b;
  font-size: 12px;
  font-weight: 800;
  cursor: pointer;
  font-family: $font-family;
  transition: background 140ms ease;

  &:hover { background: rgba(255, 255, 255, 0.8); }
}

.toast-decline {
  width: 30px;
  height: 30px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.18);
  background: transparent;
  color: rgba(255, 255, 255, 0.55);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 140ms ease, background 140ms ease, border-color 140ms ease;

  &:hover { color: #fff; background: rgba(255, 255, 255, 0.08); border-color: rgba(255, 255, 255, 0.3); }
}

// Shrinking progress line at the bottom
.toast-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 2px;
  width: 100%;
  background: rgba(255, 255, 255, 0.35);
  transform-origin: left;
  animation: shrink linear forwards;
}

@keyframes shrink {
  from { transform: scaleX(1); }
  to   { transform: scaleX(0); }
}

// hold Esc to accept: the button fills while the key is down
.toast-accept {
  position: relative;
  overflow: hidden;
  isolation: isolate;

  &::before {
    content: '';
    position: absolute;
    inset: 0;
    z-index: -1;
    background: #8BE28B;
    transform: scaleX(0);
    transform-origin: left;
  }
  &.holding::before { animation: hold-fill 700ms linear forwards; }
}

.toast-hint {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
}

@keyframes hold-fill {
  from { transform: scaleX(0); }
  to   { transform: scaleX(1); }
}

</style>
