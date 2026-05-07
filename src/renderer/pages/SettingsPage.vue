<template>
  <div class="settings-page">
    <div class="settings-sidebar">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="settings-tab"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        <component :is="tab.icon" class="tab-icon" />
        <span>{{ tab.label }}</span>
      </button>
    </div>
    <div class="settings-content">
      <Transition name="fade" mode="out-in">
        <component :is="currentComponent" />
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h } from 'vue'
import GameSettings from './settings/GameSettings.vue'
import LauncherSettings from './settings/LauncherSettings.vue'
import AccountsSettings from './settings/AccountsSettings.vue'
import AppearanceSettings from './settings/AppearanceSettings.vue'
import AboutSettings from './settings/AboutSettings.vue'
import ProfilesSettings from './settings/ProfilesSettings.vue'

const activeTab = ref('profiles')

const GameIcon = () =>
  h('svg', { width: 14, height: 14, viewBox: '0 0 14 14', fill: 'none' }, [
    h('rect', { x: 1, y: 3, width: 12, height: 8, rx: 1, stroke: 'currentColor', 'stroke-width': '1.2' }),
    h('line', { x1: 4.5, y1: 6, x2: 4.5, y2: 8, stroke: 'currentColor', 'stroke-width': '1.2', 'stroke-linecap': 'round' }),
    h('line', { x1: 3.5, y1: 7, x2: 5.5, y2: 7, stroke: 'currentColor', 'stroke-width': '1.2', 'stroke-linecap': 'round' }),
    h('circle', { cx: 9.5, cy: 6.5, r: 0.7, fill: 'currentColor' }),
    h('circle', { cx: 11, cy: 7.5, r: 0.7, fill: 'currentColor' }),
  ])

const LauncherIcon = () =>
  h('svg', { width: 14, height: 14, viewBox: '0 0 14 14', fill: 'none' }, [
    h('path', { d: 'M7 1.5L11.5 7L7 12.5L2.5 7L7 1.5Z', stroke: 'currentColor', 'stroke-width': '1.2', 'stroke-linejoin': 'round' }),
    h('circle', { cx: 7, cy: 7, r: 1.5, fill: 'currentColor' }),
  ])

const AccountIcon = () =>
  h('svg', { width: 14, height: 14, viewBox: '0 0 14 14', fill: 'none' }, [
    h('circle', { cx: 7, cy: 4.5, r: 2.5, stroke: 'currentColor', 'stroke-width': '1.2' }),
    h('path', { d: 'M1.5 12.5C1.5 10.015 4.015 8 7 8s5.5 2.015 5.5 4.5', stroke: 'currentColor', 'stroke-width': '1.2', 'stroke-linecap': 'round' }),
  ])

const AppearIcon = () =>
  h('svg', { width: 14, height: 14, viewBox: '0 0 14 14', fill: 'none' }, [
    h('circle', { cx: 7, cy: 7, r: 5.5, stroke: 'currentColor', 'stroke-width': '1.2' }),
    h('path', { d: 'M7 1.5C7 1.5 10 4 10 7s-3 5.5-3 5.5', stroke: 'currentColor', 'stroke-width': '1.2' }),
  ])

const AboutIcon = () =>
  h('svg', { width: 14, height: 14, viewBox: '0 0 14 14', fill: 'none' }, [
    h('circle', { cx: 7, cy: 7, r: 5.5, stroke: 'currentColor', 'stroke-width': '1.2' }),
    h('line', { x1: 7, y1: 5.5, x2: 7, y2: 9.5, stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }),
    h('circle', { cx: 7, cy: 3.8, r: 0.7, fill: 'currentColor' }),
  ])

const ProfilesIcon = () =>
  h('svg', { width: 14, height: 14, viewBox: '0 0 14 14', fill: 'none' }, [
    h('rect', { x: 1, y: 2, width: 12, height: 3, rx: 1, stroke: 'currentColor', 'stroke-width': '1.2' }),
    h('rect', { x: 1, y: 6.5, width: 12, height: 3, rx: 1, stroke: 'currentColor', 'stroke-width': '1.2' }),
    h('rect', { x: 1, y: 11, width: 12, height: 1.5, rx: 0.75, stroke: 'currentColor', 'stroke-width': '1.2' }),
  ])

const tabs = [
  { id: 'profiles', label: 'Profiles', icon: ProfilesIcon },
  { id: 'game', label: 'Game', icon: GameIcon },
  { id: 'launcher', label: 'Launcher', icon: LauncherIcon },
  { id: 'accounts', label: 'Accounts', icon: AccountIcon },
  { id: 'appearance', label: 'Appearance', icon: AppearIcon },
  { id: 'about', label: 'About', icon: AboutIcon },
]

const componentMap: Record<string, unknown> = {
  profiles: ProfilesSettings,
  game: GameSettings,
  launcher: LauncherSettings,
  accounts: AccountsSettings,
  appearance: AppearanceSettings,
  about: AboutSettings,
}

const currentComponent = computed(() => componentMap[activeTab.value])
</script>

<style lang="scss" scoped>
.settings-page {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.settings-sidebar {
  width: 160px;
  flex-shrink: 0;
  background: $surface;
  border-right: 1px solid $border;
  padding: $sp-4 $sp-2;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.settings-tab {
  display: flex;
  align-items: center;
  gap: $sp-2;
  padding: $sp-2 $sp-3;
  font-size: 13px;
  font-weight: 500;
  color: $text-secondary;
  background: transparent;
  border: none;
  border-radius: $radius;
  cursor: pointer;
  text-align: left;
  width: 100%;
  transition: background $transition, color $transition;

  &:hover {
    background: $surface-elevated;
    color: $text-primary;
  }

  &.active {
    background: $surface-elevated;
    color: $text-primary;
    font-weight: 600;

    .tab-icon {
      color: $text-primary;
    }
  }

  .tab-icon {
    color: $muted;
    flex-shrink: 0;
  }
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: $sp-8;
}
</style>
