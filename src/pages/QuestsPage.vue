<template>
  <div class="quests-page">

    <WipPage
      v-if="MAINTENANCE"
      label="Quests under maintenance"
      sub="We're working at full speed to bring it back. Sorry for the inconvenience!"
    />

    <template v-else>
    <!-- Tabs -->
    <div class="tab-row">
      <button :ref="(el) => setTabBtnRef('quests', el as HTMLElement | null)" class="tab-btn" :class="{ active: activeTab === 'quests' }" @click="activeTab = 'quests'">Quests</button>
      <button :ref="(el) => setTabBtnRef('leaderboard', el as HTMLElement | null)" class="tab-btn" :class="{ active: activeTab === 'leaderboard' }" @click="showLeaderboard">Leaderboard</button>
      <span class="tab-indicator" :style="indicatorStyle" />
    </div>

    <!-- ── Quests tab ──────────────────────────────────────────────────── -->
    <template v-if="activeTab === 'quests'">
      <div class="quests-header">
        <span class="quests-week" v-if="questsStore.week">Resets weekly · {{ questsStore.week }}</span>
        <div class="quests-toolbar">
          <Input
            v-model="searchQuery"
            placeholder="Search quests…"
            class="quests-search"
          >
            <template #suffix>
              <Icon v-if="!searchQuery" name="search" :size="13" />
              <button v-else class="quests-search-clear" title="Clear" @click="searchQuery = ''">
                <Icon name="close" :size="12" />
              </button>
            </template>
          </Input>
          <div class="status-chip-row">
            <button
              v-for="f in statusFilters"
              :key="f.value"
              class="status-chip"
              :class="{ active: statusFilter === f.value }"
              @click="statusFilter = f.value"
            >
              {{ f.label }}
            </button>
          </div>
        </div>
      </div>

      <div v-if="questsStore.loading" class="empty-tab">
        <span class="btn-spinner" />
      </div>

      <AdvancementTree
        v-else
        class="quest-tree-fill"
        :quests="questsStore.quests"
        :claiming-id="claimingId"
        :search-query="searchQuery"
        :status-filter="statusFilter"
        @claim="claim"
      />
    </template>

    <!-- ── Leaderboard tab ─────────────────────────────────────────────── -->
    <template v-else>
      <div class="leaderboard-scroll">
        <div v-if="questsStore.lbLoading" class="empty-tab">
          <span class="btn-spinner" />
        </div>

        <div v-else class="leaderboard-list">
          <div
            v-for="entry in questsStore.leaderboard"
            :key="entry.uuid"
            class="lb-row"
            :class="{ me: entry.rank === questsStore.myRank }"
          >
            <span class="lb-rank">#{{ entry.rank }}</span>
            <span class="lb-name">{{ entry.username }}</span>
            <span class="lb-xp">{{ entry.xp }} XP</span>
          </div>

          <div
            v-if="questsStore.myRank && !questsStore.leaderboard.some(e => e.rank === questsStore.myRank)"
            class="lb-row lb-row--floating me"
          >
            <span class="lb-rank">#{{ questsStore.myRank }}</span>
            <span class="lb-name">You</span>
          </div>

          <p v-if="!questsStore.leaderboard.length" class="quests-empty-text">No leaderboard data yet.</p>
        </div>
      </div>
    </template>
    </template>

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useQuestsStore } from '../store/questsStore'
import type { Quest } from '../types'
import AdvancementTree from '../components/quests/AdvancementTree.vue'
import Input from '../components/common/Input.vue'
import Icon from '../components/common/Icon.vue'
import WipPage from '../components/common/WipPage.vue'
import { useSlidingTabIndicator } from '../composables/useSlidingTabIndicator'

// Temporary kill-switch while quests/leaderboard backend is being reworked —
// flip back to false to restore the real Quests/Leaderboard tabs.
const MAINTENANCE = true

const questsStore = useQuestsStore()
const activeTab = ref<'quests' | 'leaderboard'>('quests')
const { setTabBtnRef, indicatorStyle } = useSlidingTabIndicator(activeTab)
const claimingId = ref<string | null>(null)

const searchQuery = ref('')
type StatusFilter = 'all' | 'claimable' | 'progress' | 'claimed' | 'locked'
const statusFilter = ref<StatusFilter>('all')
const statusFilters: { label: string; value: StatusFilter }[] = [
  { label: 'All', value: 'all' },
  { label: 'Claimable', value: 'claimable' },
  { label: 'In Progress', value: 'progress' },
  { label: 'Claimed', value: 'claimed' },
]

async function claim(q: Quest) {
  claimingId.value = q.id
  try {
    await questsStore.claim(q.id)
  } finally {
    claimingId.value = null
  }
}

function showLeaderboard() {
  activeTab.value = 'leaderboard'
  if (!questsStore.leaderboard.length) questsStore.loadLeaderboard()
}

onMounted(() => {
  if (MAINTENANCE) return
  questsStore.load()
})
</script>

<style lang="scss" scoped>
.quests-page {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
  overflow: hidden;
  padding: 4px 2px 8px;
}

.quest-tree-fill {
  flex: 1;
  min-height: 0;
}

.leaderboard-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

// ── Tabs (shared pattern with Locker/Mods/Friends/Profiles) ───────────────────
.tab-row {
  position: relative;
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.tab-btn {
  padding: 8px 22px;
  background: #0d0d0d;
  border: none;
  color: #aaa;
  font-family: 'Mojangles', monospace;
  font-size: 13px;
  cursor: pointer;
  letter-spacing: 0.02em;
  transition: background 80ms, color 80ms;

  &:hover { background: #1a1a1a; color: #ccc; }

  &.active {
    background: #111;
    color: #d9d9d9;
  }
}

.tab-indicator {
  position: absolute;
  bottom: 0;
  height: 2px;
  background: rgba(255, 255, 255, 0.3);
  pointer-events: none;
  transition: left 260ms cubic-bezier(0.16, 1, 0.3, 1), width 260ms cubic-bezier(0.16, 1, 0.3, 1);
}

.quests-header {
  flex-shrink: 0;
  padding: 2px 2px 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.quests-week {
  font-size: 11px;
  color: $text-muted;
  letter-spacing: 0.04em;
}

.quests-toolbar {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-wrap: wrap;
}

.quests-search {
  width: 220px;
  flex-shrink: 0;
}

.quests-search-clear {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: $text-muted;
  cursor: pointer;
  padding: 2px;
  margin-right: 6px;
  border-radius: 4px;
  transition: color 140ms $ease-out, background 140ms $ease-out;

  &:hover { color: $text-primary; background: rgba(255, 255, 255, 0.08); }
}

.status-chip-row {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.status-chip {
  padding: 6px 14px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid $border;
  color: $text-secondary;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.02em;
  cursor: pointer;
  transition: background 160ms $ease-out, border-color 160ms $ease-out, color 160ms $ease-out;

  &:hover { background: rgba(255, 255, 255, 0.08); color: $text-primary; }

  &.active {
    background: color-mix(in srgb, var(--accent, #{$primary}) 16%, transparent);
    border-color: var(--accent, #{$primary});
    color: var(--accent, #{$primary});
  }
}

.empty-tab {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 60px 0;
}

.quests-empty-text {
  font-size: 12px;
  color: $text-muted;
  padding: 20px 4px;
}

// ── Leaderboard ───────────────────────────────────────────────────────────────
.leaderboard-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.lb-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 16px;
  background: rgba(10, 10, 11, 0.72);
  border: 1px solid $border;

  &.me { border-color: #ffb300; background: rgba(255, 179, 0, 0.08); }
  &--floating { margin-top: 6px; border-style: dashed; }
}

.lb-rank {
  width: 32px;
  flex-shrink: 0;
  font-family: 'Mojangles', monospace;
  font-size: 13px;
  color: $text-secondary;
}

.lb-name {
  flex: 1;
  font-size: 13px;
  color: $text-primary;
}

.lb-xp {
  font-size: 12px;
  color: $text-secondary;
  flex-shrink: 0;
}

// ── Spinner ───────────────────────────────────────────────────────────────────
.btn-spinner {
  width: 18px;
  height: 18px;
  border: 2px solid rgba(255,255,255,0.25);
  border-top-color: #fff;
  border-radius: 50%;
  animation: quests-spin 0.7s linear infinite;
  display: inline-block;
}

@keyframes quests-spin {
  to { transform: rotate(360deg); }
}
</style>
