<template>
  <div class="launch-wrap">
  <div class="launch-split" ref="rootEl">

    <!-- Main launch area -->
    <button
      class="launch-main"
      :class="status"
      :disabled="status === 'starting' || status === 'stopping'"
      @click="onLaunch"
    >
      <!-- Idle -->
      <template v-if="status === 'idle'">
        <svg class="launch-icon" xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/>
          <path d="m12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/>
          <path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/>
          <path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/>
        </svg>
        <span class="launch-label">Launch</span>
      </template>

      <!-- Starting -->
      <template v-else-if="status === 'starting'">
        <svg class="launch-icon spin" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
        </svg>
        <span class="launch-label" :class="{ 'launch-label--status': store.statusMsg }">{{ store.statusMsg || 'Starting…' }}</span>
      </template>

      <!-- Running -->
      <template v-else-if="status === 'running'">
        <svg class="launch-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <rect x="4" y="4" width="16" height="16" rx="2"/>
        </svg>
        <span class="launch-label">Running</span>
      </template>

      <!-- Error -->
      <template v-else-if="status === 'error'">
        <svg class="launch-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        <span class="launch-label">Retry</span>
      </template>
    </button>

    <!-- Divider -->
    <div class="launch-divider" />

    <!-- Dropdown chevron -->
    <button class="launch-chevron" @click.stop="toggleDropdown" :disabled="status === 'starting' || status === 'stopping'">
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
    </button>

    <!-- Profile dropdown -->
    <Transition name="dropdown-fade">
      <div v-if="dropdownOpen" class="profile-dropdown">
        <div class="dropdown-label">Profile</div>
        <button
          v-for="profile in profiles"
          :key="profile.id"
          class="dropdown-item"
          :class="{ active: profile.id === activeProfile?.id }"
          @click="selectProfile(profile.id)"
        >
          <span class="dropdown-item-name">{{ profile.name }}</span>
          <span class="dropdown-item-version">{{ profile.version }}</span>
        </button>
        <div v-if="profiles.length === 0" class="dropdown-empty">No profiles</div>
      </div>
    </Transition>
  </div>

  <Transition name="err-fade">
    <div v-if="status === 'error' && store.lastError" class="launch-error">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><circle cx="6" cy="6" r="5" stroke="currentColor" stroke-width="1.2"/><line x1="6" y1="3.5" x2="6" y2="6.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/><circle cx="6" cy="8.5" r="0.6" fill="currentColor"/></svg>
      <span class="launch-error-text">{{ store.lastError.split('\n')[0] }}</span>
    </div>
  </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useLauncherStore } from '../../store/launcherStore'

const store = useLauncherStore()
const status = computed(() => store.status)
const profiles = computed(() => store.profiles)
const activeProfile = computed(() => store.activeProfile)
const dropdownOpen = ref(false)
const rootEl = ref<HTMLElement | null>(null)

function onLaunch() {
  if (status.value === 'running') {
    store.killGame()
  } else if (status.value === 'idle' || status.value === 'error') {
    store.launch()
  }
}

function toggleDropdown() {
  dropdownOpen.value = !dropdownOpen.value
}

async function selectProfile(id: string) {
  await store.setActiveProfile(id)
  dropdownOpen.value = false
}

function onClickOutside(e: MouseEvent) {
  if (dropdownOpen.value && rootEl.value && !rootEl.value.contains(e.target as Node)) {
    dropdownOpen.value = false
  }
}

onMounted(() => document.addEventListener('mousedown', onClickOutside, true))
onUnmounted(() => document.removeEventListener('mousedown', onClickOutside, true))
</script>

<style lang="scss" scoped>
.launch-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.launch-error {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  background: $surface-elevated;
  border: 1px solid $border;
  border-radius: $radius;
  max-width: 275px;
  color: $text-secondary;
}

.launch-error-text {
  font-size: 11px;
  color: $text-secondary;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.err-fade-enter-active, .err-fade-leave-active { transition: opacity 0.2s ease; }
.err-fade-enter-from, .err-fade-leave-to { opacity: 0; }

.launch-split {
  position: relative;
  display: flex;
  align-items: stretch;
  width: 275px;
  height: 70px;
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius-lg;
  overflow: visible;
}

.launch-main {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  background: none;
  border: none;
  border-radius: $radius-lg 0 0 $radius-lg;
  cursor: pointer;
  color: $text-primary;
  transition: background 150ms;

  &:hover:not(:disabled) {
    background: $surface-elevated;
  }

  &:disabled {
    cursor: not-allowed;
    opacity: 0.7;
  }

  &.running {
    color: $text-secondary;
    .launch-icon { color: $text-secondary; }
  }

  &.error {
    color: $text-secondary;
  }
}

.launch-icon {
  flex-shrink: 0;
  &.spin {
    animation: spin 0.9s linear infinite;
  }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.launch-label {
  font-size: 28px;
  font-weight: 700;
  letter-spacing: -0.3px;
  line-height: 1;

  &--status {
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 180px;
  }
}

.launch-divider {
  width: 1px;
  background: $border;
  flex-shrink: 0;
  margin: 12px 0;
}

.launch-chevron {
  width: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  border-radius: 0 $radius-lg $radius-lg 0;
  color: $text-secondary;
  cursor: pointer;
  transition: background 150ms, color 150ms;
  flex-shrink: 0;

  &:hover:not(:disabled) {
    background: $surface-elevated;
    color: $text-primary;
  }

  &:disabled { cursor: not-allowed; }
}

// ── Dropdown ──────────────────────────────────────────────────────────────────
.profile-dropdown {
  position: absolute;
  bottom: calc(100% + 8px);
  left: 0;
  right: 0;
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius-lg;
  overflow: hidden;
  z-index: 10;
}

.dropdown-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: $muted;
  padding: 10px 14px 6px;
}

.dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 9px 14px;
  background: none;
  border: none;
  color: $text-secondary;
  cursor: pointer;
  font-size: 13px;
  transition: background 120ms, color 120ms;

  &:hover {
    background: $surface-elevated;
    color: $text-primary;
  }

  &.active {
    color: $text-primary;
    .dropdown-item-name { font-weight: 600; }
  }
}

.dropdown-item-name {
  font-weight: 500;
}

.dropdown-item-version {
  font-size: 11px;
  color: $muted;
}

.dropdown-empty {
  padding: 12px 14px;
  font-size: 12px;
  color: $muted;
  font-style: italic;
}

.dropdown-fade-enter-active, .dropdown-fade-leave-active { transition: opacity 0.15s ease, transform 0.15s ease; }
.dropdown-fade-enter-from, .dropdown-fade-leave-to { opacity: 0; transform: translateY(6px); }
</style>
