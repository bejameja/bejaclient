<template>
  <div class="weekly-panel">
    <h2 class="weekly-heading">Weekly tasks:</h2>
    <div class="weekly-divider" />

    <div class="task-list">
      <div
        v-for="task in tasks"
        :key="task.id"
        class="task-item"
        :class="{ done: task.done }"
      >
        <div class="task-check">
          <svg v-if="task.done" width="9" height="9" viewBox="0 0 10 10">
            <polyline points="1.5,5 4,7.5 8.5,2" stroke="currentColor" stroke-width="1.6" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <span v-else class="task-dot" />
        </div>
        <div class="task-body">
          <span class="task-name">{{ task.name }}</span>
          <div class="task-bar-wrap">
            <div class="task-bar-fill" :style="{ width: task.progress + '%' }" />
          </div>
        </div>
        <span class="task-xp">+{{ task.xp }}</span>
      </div>
    </div>

    <div class="weekly-footer">
      <div class="xp-row">
        <span class="xp-text">{{ earnedXp }} / {{ totalXp }} XP</span>
        <span class="xp-pct">{{ xpPct }}%</span>
      </div>
      <div class="xp-bar">
        <div class="xp-fill" :style="{ width: xpPct + '%' }" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const tasks = [
  { id: 1, name: 'Play 3 matches',   xp: 500,  progress: 66,  done: false },
  { id: 2, name: 'Win a match',      xp: 1000, progress: 0,   done: false },
  { id: 3, name: 'Deal 1000 damage', xp: 750,  progress: 100, done: true  },
  { id: 4, name: 'Apply a cosmetic', xp: 250,  progress: 100, done: true  },
  { id: 5, name: 'Join a server',    xp: 200,  progress: 100, done: true  },
]

const earnedXp = computed(() => tasks.filter(t => t.done).reduce((a, t) => a + t.xp, 0))
const totalXp  = computed(() => tasks.reduce((a, t) => a + t.xp, 0))
const xpPct    = computed(() => Math.round(earnedXp.value / totalXp.value * 100))
</script>

<style lang="scss" scoped>
.weekly-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 0;
}

.weekly-heading {
  font-size: 30px;
  font-weight: 900;
  color: $text-primary;
  line-height: normal;
  white-space: nowrap;
  margin: 16px 14px 8px;
  padding: 0;
}

.weekly-divider {
  height: 1px;
  background: $border;
  margin: 0 14px 10px;
}

// ── Task list ─────────────────────────────────────────────────────────────────
.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;

  &::-webkit-scrollbar { width: 3px; }
  &::-webkit-scrollbar-track { background: transparent; }
  &::-webkit-scrollbar-thumb { background: $border; border-radius: 2px; }
}

.task-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px 4px;
  border-radius: 4px;
  transition: background $transition;

  &:hover { background: $surface-elevated; }

  &.done {
    .task-name { color: $muted; text-decoration: line-through; }
    .task-check { color: $text-secondary; }
    .task-bar-fill { background: $border; }
    .task-xp { color: $muted; }
  }
}

.task-check {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: $text-secondary;
  margin-top: 1px;
}

.task-dot {
  display: block;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  border: 1.5px solid $border;
}

.task-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.task-name {
  font-size: 11px;
  font-weight: 500;
  color: $text-secondary;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.task-bar-wrap {
  height: 2px;
  background: $surface-elevated;
  border-radius: 1px;
  overflow: hidden;
}

.task-bar-fill {
  height: 100%;
  background: $border-strong;
  border-radius: 1px;
  transition: width 0.5s ease;
}

.task-xp {
  font-size: 10px;
  font-weight: 700;
  color: $text-secondary;
  flex-shrink: 0;
  margin-top: 2px;
}

// ── Footer XP ─────────────────────────────────────────────────────────────────
.weekly-footer {
  padding: 10px 14px 14px;
  border-top: 1px solid $border;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.xp-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.xp-text {
  font-size: 10px;
  font-weight: 600;
  color: $muted;
}

.xp-pct {
  font-size: 10px;
  font-weight: 700;
  color: $text-secondary;
}

.xp-bar {
  height: 3px;
  background: $surface-elevated;
  border-radius: 2px;
  overflow: hidden;
}

.xp-fill {
  height: 100%;
  background: $border-strong;
  border-radius: 2px;
  transition: width 0.5s ease;
}
</style>
