<template>
  <Teleport to="body">

    <!-- Backdrop -->
    <Transition name="fade">
      <div v-if="store.drawerOpen" class="drawer-backdrop" @click="store.closeDrawer()" />
    </Transition>

    <!-- Drawer -->
    <Transition name="drawer-slide">
      <div v-if="store.drawerOpen" class="notifications-drawer">

        <div class="drawer-header">
          <span class="drawer-title">Notifications</span>
          <div class="drawer-header-actions">
            <button v-if="store.notifications.length" class="clear-btn" @click="store.notifications = []">
              Clear all
            </button>
            <button class="close-btn" @click="store.closeDrawer()">
              <svg width="12" height="12" viewBox="0 0 7 7" shape-rendering="crispEdges" aria-hidden="true"><path d="M0 0h2v1H0zM5 0h2v1H5zM1 1h2v1H1zM4 1h2v1H4zM2 2h3v1H2zM3 3h1v1H3zM2 4h3v1H2zM1 5h2v1H1zM4 5h2v1H4zM0 6h2v1H0zM5 6h2v1H5z" fill="currentColor"/></svg>
            </button>
          </div>
        </div>

        <div class="drawer-body">
          <div v-if="store.notifications.length === 0" class="empty-state">
            <svg class="empty-env" width="40" height="30" viewBox="0 0 8 6" shape-rendering="crispEdges" aria-hidden="true"><path d="M0 0h8v6H0z" fill="#f3efe3"/><path d="M1 1h1v1H1zM2 2h1v1H2zM3 3h2v1H3zM5 2h1v1H5zM6 1h1v1H6z" fill="#8a8575"/><path d="M3 4h2v2H3z" fill="#d11f1f"/></svg>
            <span>No notifications</span>
          </div>

          <div v-for="group in groupedNotifications" :key="group.label" class="notif-group">
            <span class="notif-group-label">{{ group.label }}</span>
            <TransitionGroup tag="div" name="notif-pop" class="notif-list">
            <div v-for="notif in group.items" :key="notif.id" class="notif-card">

              <!-- Friend request -->
              <template v-if="notif.type === 'friend_request'">
                <div class="notif-avatar">{{ notif.data?.username?.[0]?.toUpperCase() }}</div>
                <div class="notif-content">
                  <p class="notif-title">{{ notif.title }}</p>
                  <p class="notif-body">{{ notif.body }}</p>
                  <div class="notif-actions">
                    <button class="btn-accept" @click="acceptRequest(notif)">Accept</button>
                    <button class="btn-decline" @click="declineRequest(notif)">Decline</button>
                  </div>
                </div>
                <span class="notif-time">{{ timeAgo(notif.timestamp) }}</span>
              </template>

              <!-- System -->
              <template v-else-if="notif.type === 'system'">
                <div class="notif-icon">
                  <svg class="notif-seal" width="26" height="26" viewBox="0 0 7 7" shape-rendering="crispEdges" aria-hidden="true"><path d="M2 0h3v1H2zM1 1h5v1H1zM0 2h7v3H0zM1 5h5v1H1zM2 6h3v1H2z" fill="#b81818"/><path d="M2 1h2v1H2zM1 2h1v2H1z" fill="#e43a3a"/><path d="M3 2h1v3H3zM2 3h3v1H2z" fill="#8f1010"/></svg>
                </div>
                <div class="notif-content">
                  <p class="notif-title">{{ notif.title }}</p>
                  <p class="notif-body">{{ notif.body }}</p>
                </div>
                <div class="notif-right">
                  <span class="notif-time">{{ timeAgo(notif.timestamp) }}</span>
                  <button class="notif-dismiss" @click="store.remove(notif.id)">×</button>
                </div>
              </template>

            </div>
            </TransitionGroup>
          </div>
        </div>

      </div>
    </Transition>

  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useNotificationsStore } from '../../store/notificationsStore'
import { useFriendsStore } from '../../store/friendsStore'
import type { BejaNotification } from '../../store/notificationsStore'

const store        = useNotificationsStore()
const friendsStore = useFriendsStore()

// ── Grouping - Today / Earlier this week / Older, newest-first ──────────────
function dayBucket(ts: number): string {
  const diffDays = Math.floor((Date.now() - ts) / 86400000)
  if (diffDays <= 0) return 'Today'
  if (diffDays < 7)  return 'Earlier this week'
  return 'Older'
}

const groupedNotifications = computed(() => {
  const order = ['Today', 'Earlier this week', 'Older']
  const buckets = new Map<string, BejaNotification[]>()
  for (const notif of store.notifications) {
    const label = dayBucket(notif.timestamp)
    if (!buckets.has(label)) buckets.set(label, [])
    buckets.get(label)!.push(notif)
  }
  return order
    .filter(label => buckets.has(label))
    .map(label => ({ label, items: buckets.get(label)! }))
})

async function acceptRequest(notif: BejaNotification) {
  if (!notif.data?.uuid) return
  await friendsStore.acceptRequest(notif.data.uuid)
  store.remove(notif.id)
}

async function declineRequest(notif: BejaNotification) {
  if (!notif.data?.uuid) return
  await friendsStore.declineRequest(notif.data.uuid)
  store.remove(notif.id)
}

function timeAgo(ts: number): string {
  const diff = Math.floor((Date.now() - ts) / 1000)
  if (diff < 60)   return 'just now'
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
  return `${Math.floor(diff / 86400)}d ago`
}
</script>

<style lang="scss" scoped>
// The drawer is the content of a letter: a sheet of paper with ink text, fold lines and a wax seal.
// Square corners and stepped animation like the rest of the hub.
$px-font: 'Minecrafter', 'Plus Jakarta Sans', sans-serif;
$paper: #f1ecde;
$paper-shade: #e4dec9;
$ink: #2a2318;
$ink-soft: rgba(42, 35, 24, 0.68);
$ink-faint: rgba(42, 35, 24, 0.45);
$rule: repeating-linear-gradient(90deg, rgba(42, 35, 24, 0.4) 0 4px, transparent 4px 8px);   // a dotted pixel line

.drawer-backdrop {
  position: fixed;
  inset: 0;
  z-index: 200;
  background: rgba(0, 0, 0, 0.45);
}

.notifications-drawer {
  position: fixed;
  top: 0;
  right: $right-panel-width;
  width: 360px;
  height: 100vh;
  z-index: 201;
  display: flex;
  flex-direction: column;
  color: $ink;
  // paper with two faint fold lines (a letter folded in three) and a hard drop shadow
  background:
    linear-gradient(to bottom, transparent calc(33.3% - 1px), rgba(42, 35, 24, 0.12) calc(33.3% - 1px) 33.3%, rgba(255, 255, 255, 0.5) 33.3% calc(33.3% + 1px), transparent calc(33.3% + 1px)),
    linear-gradient(to bottom, transparent calc(66.6% - 1px), rgba(42, 35, 24, 0.12) calc(66.6% - 1px) 66.6%, rgba(255, 255, 255, 0.5) 66.6% calc(66.6% + 1px), transparent calc(66.6% + 1px)),
    $paper;
  border-left: 4px solid $paper-shade;
  box-shadow: -6px 0 0 rgba(0, 0, 0, 0.5);
}

// ── Header: seal + title, like the top of a letter ────────────────────────────
.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 20px 16px 24px;
  flex-shrink: 0;
  position: relative;
  &::after {
    content: '';
    position: absolute;
    left: 24px;
    right: 20px;
    bottom: 0;
    height: 2px;
    background: $ink;
  }
}

.drawer-title {
  font-family: $px-font;
  font-size: 26px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: $ink;
}

.drawer-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.clear-btn {
  height: 26px;
  padding: 0 10px;
  border: 2px solid $ink;
  border-radius: 0;
  background: transparent;
  font-family: $px-font;
  font-size: 11px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: $ink;
  cursor: pointer;
  transition: background 120ms steps(2), color 120ms steps(2);
  &:hover { background: $ink; color: $paper; }
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  padding: 0;
  border: 2px solid $ink;
  border-radius: 0;
  background: transparent;
  color: $ink;
  cursor: pointer;
  transition: background 120ms steps(2), color 120ms steps(2);
  &:hover { background: $ink; color: $paper; }
}

// ── Body ──────────────────────────────────────────────────────────────────────
.drawer-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px 20px 28px 24px;
  display: flex;
  flex-direction: column;
  gap: 8px;

  &::-webkit-scrollbar { width: 6px; }
  &::-webkit-scrollbar-track { background: transparent; }
  &::-webkit-scrollbar-thumb { background: rgba(42, 35, 24, 0.35); border-radius: 0; }
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  font-family: $px-font;
  font-size: 14px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: $ink-faint;
  padding: 40px 0;
}
.empty-env { box-shadow: 0 0 0 2px $ink; opacity: 0.8; }

// ── Notification groups: dated sections of the letter ────────────────────────
.notif-group + .notif-group { margin-top: 18px; }

.notif-group-label {
  display: block;
  font-family: $px-font;
  font-size: 12px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: $ink-soft;
  margin-bottom: 10px;
}

.notif-list {
  display: flex;
  flex-direction: column;
}

// ── One notification = one paragraph of the letter, separated by a dotted line ─
.notif-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 0;
  background: none;
  border: 0;
  border-radius: 0;
  position: relative;
  & + .notif-card::before {
    content: '';
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    height: 2px;
    background: $rule;
  }
}

.notif-avatar {
  width: 36px;
  height: 36px;
  border-radius: 0;
  background: $ink;
  color: $paper;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-family: $px-font;
  font-size: 18px;
}

.notif-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.notif-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.notif-title {
  font-family: $px-font;
  font-size: 16px;
  letter-spacing: 0.04em;
  color: $ink;
  margin: 0;
}

.notif-body {
  font-size: 13px;
  color: $ink-soft;
  margin: 0 0 8px;
  line-height: 1.5;
}

.notif-actions {
  display: flex;
  gap: 6px;
}

.btn-accept,
.btn-decline {
  height: 26px;
  padding: 0 12px;
  border-radius: 0;
  font-family: $px-font;
  font-size: 12px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  cursor: pointer;
  transition: background 120ms steps(2), color 120ms steps(2);
}

.btn-accept {
  border: 2px solid $ink;
  background: $ink;
  color: $paper;
  &:hover { background: #b81818; border-color: #b81818; }   // wax red on hover
}

.btn-decline {
  border: 2px solid $ink;
  background: transparent;
  color: $ink;
  &:hover { background: $ink; color: $paper; }
}

.notif-right {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
  flex-shrink: 0;
}

.notif-time {
  font-family: $px-font;
  font-size: 11px;
  letter-spacing: 0.06em;
  color: $ink-faint;
  white-space: nowrap;
  flex-shrink: 0;
}

.notif-dismiss {
  background: none;
  border: 0;
  border-radius: 0;
  color: $ink-faint;
  font-size: 20px;
  cursor: pointer;
  padding: 0 2px;
  line-height: 1;
  transition: color 120ms steps(2);
  &:hover { color: #b81818; }
}

// ── Transitions (stepped, like the rest of the hub) ───────────────────────────
.fade-enter-active, .fade-leave-active { transition: opacity 160ms steps(4); }
.fade-enter-from,  .fade-leave-to      { opacity: 0; }

.drawer-slide-enter-active, .drawer-slide-leave-active { transition: transform 200ms steps(6); }
.drawer-slide-enter-from,  .drawer-slide-leave-to      { transform: translateX(100%); }

.notif-pop-enter-active,
.notif-pop-leave-active { transition: opacity 160ms steps(4), transform 160ms steps(4); }
.notif-pop-move { transition: transform 200ms steps(5); }
.notif-pop-enter-from { opacity: 0; transform: translateY(-6px); }
.notif-pop-leave-to   { opacity: 0; transform: translateX(12px); }
</style>
