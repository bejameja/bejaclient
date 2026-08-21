<template>
  <div class="launch-wrap" ref="rootEl">
  <div class="launch-split" :class="status">

    <!-- Main launch area -->
    <button
      class="launch-main"
      :class="status"
      :disabled="status === 'starting' || status === 'stopping' || (status === 'idle' && isMaintenance) || (status === 'idle' && isPartyLeader && !lobbyStore.canLaunch)"
      @click="onLaunch"
    >
      <!-- Idle + BejaClient profile selected: blocked, maintenance clock -->
      <template v-if="status === 'idle' && isMaintenance">
        <svg class="launch-rocket launch-rocket--maint" width="27" height="27" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="9" />
          <polyline points="12 7 12 12 15.5 13.8" />
        </svg>
        <div class="launch-text">
          <span class="launch-label launch-label--title launch-label--maint" style="font-size: 14px;">MAINTENANCE</span>
          <span v-if="activeProfile" class="launch-version launch-version--maint">{{ versionLabel }}</span>
        </div>
      </template>

      <!-- Idle: rocket icon + title/subtitle text block, left-aligned row -->
      <template v-else-if="status === 'idle'">
        <svg class="launch-rocket" width="27" height="27" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/>
          <path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/>
          <path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/>
          <path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/>
        </svg>
        <div class="launch-text">
          <span class="launch-label launch-label--title">{{ props.readyMode ? (lobbyStore.isReady ? 'Ready' : 'Ready up') : 'Launch' }}</span>
          <span v-if="activeProfile" class="launch-version">{{ versionLabel }}</span>
        </div>
      </template>

      <!-- Starting -->
      <template v-else-if="status === 'starting'">
        <div class="launch-state-overlay">
          <svg class="launch-icon spin" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
          </svg>
          <span class="launch-label launch-label--status">{{ store.statusMsg || $t('launch.starting') }}</span>
        </div>
      </template>

      <!-- Running -->
      <template v-else-if="status === 'running'">
        <div class="launch-state-overlay">
          <span class="launch-label">{{ $t('launch.running') }}</span>
        </div>
      </template>

      <!-- Error -->
      <template v-else-if="status === 'error'">
        <div class="launch-state-overlay">
          <span class="launch-label launch-label--error">{{ $t('launch.retry') }}</span>
        </div>
      </template>
    </button>

    <!-- Dropdown chevron — swaps to a cancel/close "X" while starting or running -->
    <button
      class="launch-chevron"
      :class="{ cancel: showCancelX }"
      :title="status === 'starting' ? 'Cancel launch' : status === 'running' ? 'Close Minecraft' : undefined"
      @click.stop="showCancelX ? store.killGame() : toggleDropdown()"
      :disabled="status === 'stopping'"
    >
      <svg v-if="showCancelX" xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
      <svg v-else xmlns="http://www.w3.org/2000/svg" width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
    </button>

  </div>

  <!-- Profile dropdown — sibling of .launch-split (not a child), since
       .launch-split has overflow:hidden for its conic-gradient border and
       would otherwise clip the dropdown, which sits entirely above it. -->
  <Transition name="dropdown-fade">
    <div v-if="dropdownOpen" class="profile-dropdown">
      <div class="dropdown-label">{{ $t('launch.profile') }}</div>
      <button
        v-for="profile in profiles"
        :key="profile.id"
        class="dropdown-item"
        :class="{ active: profile.id === activeProfile?.id }"
        @click="selectProfile(profile.id)"
      >
        <span class="dropdown-item-name">{{ profile.name }}</span>
        <span v-if="profile.useBejaClient" class="dropdown-item-maint">Maintenance</span>
        <span v-else class="dropdown-item-version">{{ profile.version }}</span>
      </button>
      <div v-if="profiles.length === 0" class="dropdown-empty">{{ $t('launch.noProfiles') }}</div>
    </div>
  </Transition>

  <Transition name="err-fade">
    <div v-if="status === 'error' && store.lastError" class="launch-error">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><circle cx="6" cy="6" r="5" stroke="currentColor" stroke-width="1.2"/><line x1="6" y1="3.5" x2="6" y2="6.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/><circle cx="6" cy="8.5" r="0.6" fill="currentColor"/></svg>
      <span class="launch-error-text">{{ store.lastError }}</span>
    </div>
  </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useLauncherStore } from '../../store/launcherStore'
import { useLobbyStore } from '../../store/lobbyStore'

const props = defineProps<{ readyMode?: boolean }>()

const store = useLauncherStore()
const lobbyStore = useLobbyStore()
const status = computed(() => store.status)
const profiles = computed(() => store.profiles)
const activeProfile = computed(() => store.activeProfile)
const isMaintenance = computed(() => store.isBejaMaintenance)
const dropdownOpen = ref(false)
const rootEl = ref<HTMLElement | null>(null)
const showCancelX = computed(() => status.value === 'starting' || status.value === 'running')
const isPartyLeader = computed(() => !!lobbyStore.party && lobbyStore.isLeader)

const LOADER_NAMES: Record<string, string> = {
  fabric: 'Fabric',
  forge: 'Forge',
  neoforge: 'NeoForge',
  quilt: 'Quilt',
  vanilla: '',
}

const versionLabel = computed(() => {
  const p = activeProfile.value
  if (!p) return ''
  if (p.useBejaClient) return `BJC ${p.version}`
  const name = LOADER_NAMES[p.loader] ?? p.loader
  return name ? `${name} ${p.version}` : p.version
})

function onLaunch() {
  if (status.value === 'running') {
    store.killGame()
  } else if (status.value === 'idle' && props.readyMode) {
    lobbyStore.toggleReady()
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

onMounted(() => {
  document.addEventListener('mousedown', onClickOutside, true)

  // WebView2/Chromium sometimes fails to composite backdrop-filter at all on
  // first paint — it just renders as if the property weren't there — and only
  // starts working once some unrelated repaint elsewhere happens to trigger
  // proper layer creation. Nudging a real (sub-pixel, invisible) transform
  // forces that layer creation immediately instead of leaving it to chance.
  requestAnimationFrame(() => {
    const el = rootEl.value?.querySelector<HTMLElement>('.launch-split')
    if (!el) return
    el.style.transform = 'translateZ(0.01px)'
    requestAnimationFrame(() => { el.style.transform = '' })
  })
})
onUnmounted(() => document.removeEventListener('mousedown', onClickOutside, true))
</script>

<style lang="scss" scoped>
// LabyMod-style reference — blue/indigo gradient glass pill, play icon +
// left-aligned title/subtitle text block, swap icon on the right.
$btn-blue-1: #7d8ae6;
$btn-blue-2: #4a4fc4;
$btn-error:  #ff453a;

.launch-wrap {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.launch-error {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  padding: 6px 10px;
  background: $surface-elevated;
  border: 1px solid $border;
  border-radius: $radius;
  max-width: 340px;
  color: $text-secondary;
  cursor: pointer;
  user-select: text;

  &--maint {
    background: #0a0a0a;
    border-color: rgba(255, 255, 255, 0.08);
    border-radius: 4px;

    .launch-error-text { color: $btn-error; font-weight: 600; }
  }
}

.launch-error-text {
  font-size: 11px;
  color: $text-secondary;
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.45;
}

.err-fade-enter-active, .err-fade-leave-active { transition: opacity 0.2s ease; }
.err-fade-enter-from, .err-fade-leave-to { opacity: 0; }

.launch-split {
  position: relative;
  display: flex;
  align-items: stretch;
  width: 272px;
  height: 72px;
  border-radius: 4px;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(24, 24, 27, 0.2);
  backdrop-filter: blur(10px) saturate(160%);
  -webkit-backdrop-filter: blur(10px) saturate(160%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.14),
    inset 0 -1px 2px rgba(0, 0, 0, 0.4),
    0 24px 60px rgba(0, 0, 0, 0.65),
    $glass-shadow;
  // A neutral, always-on filter (not just set on :hover below) — without
  // this, WebView2/Chromium seems to only create the compositing layer that
  // backdrop-filter also needs the first time `filter` actually gets a real
  // value, i.e. whenever the mouse happens to first hover this element,
  // rather than on initial paint. That's what made the blur look like it
  // "eventually" appeared at random instead of being there from the start.
  filter: brightness(1);
  transition: filter 150ms ease-out;

  &:hover {
    filter: brightness(1.1);
  }

  &.error {
    background: rgba(30, 19, 19, 0.45);
  }
}

.launch-main {
  flex: 1;
  position: relative;
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: flex-start;
  gap: 12px;
  padding: 0 16px;
  background: none;
  border: none;
  outline: none;
  cursor: pointer;
  overflow: hidden;
  transition: transform 100ms cubic-bezier(0.34, 1.56, 0.64, 1);

  &:active:not(:disabled) { transform: scale(0.98); }
  &:disabled { cursor: not-allowed; opacity: 0.7; }
}

.launch-icon {
  flex-shrink: 0;
  color: #fff;
  &.spin { animation: spin 0.9s linear infinite; }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@keyframes rocket-nudge {
  0%, 100% { transform: translateY(0) rotate(0deg); }
  50% { transform: translateY(-2px) rotate(-4deg); }
}

.launch-rocket {
  flex-shrink: 0;
  color: #fff;

  .launch-main:hover:not(:disabled) & {
    animation: rocket-nudge 900ms ease-in-out infinite;
  }
}

.launch-text {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 1px;
}

.launch-label {
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 17px;
  font-weight: 800;
  letter-spacing: 0.02em;
  line-height: 1.2;
  color: #fff;

  &--status {
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
    color: #fff;
  }

  &--error { color: #fff; }

  // Same font/weight as the top bar's Hub/Locker/etc. nav labels (TopBar.vue's
  // .nav-label, which just inherits the app's default $font-family/weight 500)
  // instead of the branded Plus Jakarta Sans 800 used for "LAUNCH".
  &--maint {
    font-family: $font-family;
    font-weight: 500;
  }
}

.launch-version {
  text-align: left;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 12px;
  font-weight: 500;
  color: #fff;
  line-height: 1.2;
  background: transparent;
}


.launch-state-overlay {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: flex-start;
  gap: 10px;
  width: 100%;
  height: 100%;
}

.launch-chevron {
  width: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  outline: none;
  color: #fff;
  cursor: pointer;
  transition: color 120ms, background 120ms;
  flex-shrink: 0;

  &:disabled { cursor: not-allowed; opacity: 0.5; }

  &.cancel:hover { color: $btn-error; background: rgba(255, 69, 58, 0.12); }
}

// ── Dropdown ──────────────────────────────────────────────────────────────────
.profile-dropdown {
  position: absolute;
  bottom: calc(100% + 10px);
  left: 50%;
  width: 300px;
  transform: translateX(-50%);
  background: #0a0a0a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 4px;
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
    background: #1a1a1a;
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

.dropdown-item-maint {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: $btn-error;
}

.dropdown-empty {
  padding: 12px 14px;
  font-size: 12px;
  color: $muted;
  font-style: italic;
}

.dropdown-fade-enter-active, .dropdown-fade-leave-active { transition: opacity 0.15s ease, transform 0.15s ease; }
.dropdown-fade-enter-from, .dropdown-fade-leave-to { opacity: 0; transform: translateX(-50%) translateY(6px); }
</style>
