<template>
  <div class="launch-wrap" ref="rootEl">
  <div class="launch-split" :class="status">

    <!-- Apex-style hover notch: dim at rest, snaps to bright white on hover
         (see the reference clip — this reads as the "fun" part of hovering). -->
    <div class="launch-notch" aria-hidden="true"></div>

    <!-- Main launch area -->
    <button
      class="launch-main"
      :class="status"
      :disabled="status === 'starting' || status === 'stopping' || (status === 'idle' && isMaintenance)"
      @click="onLaunch"
    >
      <!-- Idle + BejaClient profile selected: blocked, maintenance clock -->
      <template v-if="status === 'idle' && isMaintenance">
        <svg class="launch-rocket" width="27" height="27" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="9" />
          <polyline points="12 7 12 12 15.5 13.8" />
        </svg>
        <div class="launch-text">
          <span class="launch-label launch-label--title" style="font-size: 14px;">MAINTENANCE</span>
          <span v-if="activeProfile" class="launch-version">{{ versionLabel }}</span>
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
          <span class="launch-label launch-label--title">LAUNCH</span>
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

    <!-- Dropdown chevron -->
    <button class="launch-chevron" @click.stop="toggleDropdown" :disabled="status === 'starting' || status === 'stopping'">
      <svg xmlns="http://www.w3.org/2000/svg" width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
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
    <div
      v-if="(status === 'error' && store.lastError) || (status === 'idle' && isMaintenance)"
      class="launch-error"
      :class="{ 'launch-error--maint': status === 'idle' && isMaintenance }"
    >
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><circle cx="6" cy="6" r="5" stroke="currentColor" stroke-width="1.2"/><line x1="6" y1="3.5" x2="6" y2="6.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/><circle cx="6" cy="8.5" r="0.6" fill="currentColor"/></svg>
      <span class="launch-error-text">{{ status === 'idle' && isMaintenance ? maintenanceMessage : store.lastError }}</span>
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
const isMaintenance = computed(() => store.isBejaMaintenance)
const maintenanceMessage = "BejaClient versions are under maintenance right now. We're working at full speed to bring them back. Sorry for the inconvenience!"
const dropdownOpen = ref(false)
const rootEl = ref<HTMLElement | null>(null)

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
// LabyMod-style reference — blue/indigo gradient glass pill, play icon +
// left-aligned title/subtitle text block, swap icon on the right.
$btn-blue-1: #7d8ae6;
$btn-blue-2: #4a4fc4;
$btn-error:  #ff453a;

@property --launch-angle {
  syntax: '<angle>';
  initial-value: 45deg;
  inherits: false;
}

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
    border-radius: 0;

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
  width: 360px;
  height: 96px;
  border-radius: 0;
  overflow: hidden;
  border: 1px solid transparent;
  background-image:
    linear-gradient(#131315, #131315),
    conic-gradient(
      from var(--launch-angle),
      rgba(255, 255, 255, 0.04) 0%,
      rgba(255, 255, 255, 0.4) 10%,
      rgba(255, 255, 255, 0.04) 20%,
      rgba(255, 255, 255, 0.04) 100%
    );
  background-origin: border-box;
  background-clip: padding-box, border-box;
  box-shadow:
    inset 0 1px 2px rgba(0, 0, 0, 0.6),
    inset 0 -1px 0 rgba(255, 255, 255, 0.03);
  transition: filter 150ms ease-out, --launch-angle 2000ms cubic-bezier(0.2, 0, 0, 1);

  &:hover {
    filter: brightness(1.1);
    --launch-angle: 135deg;
  }

  &.error {
    background-image:
      linear-gradient(#191313, #191313),
      conic-gradient(
        from var(--launch-angle),
        rgba(255, 69, 58, 0.08) 0%,
        rgba(255, 69, 58, 0.5) 10%,
        rgba(255, 69, 58, 0.08) 20%,
        rgba(255, 69, 58, 0.08) 100%
      );
  }
}

// Dim salmon at rest → snaps to bright glowing white on hover, fast (~240ms),
// matching the reference's quick reactive flash rather than a slow fade.
.launch-notch {
  position: absolute;
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 46%;
  height: 6px;
  background: rgba(255, 190, 180, 0.4);
  clip-path: polygon(10% 0, 90% 0, 76% 100%, 24% 100%);
  transition: background 240ms cubic-bezier(0.2, 0, 0, 1), box-shadow 240ms cubic-bezier(0.2, 0, 0, 1);
  pointer-events: none;
  z-index: 2;
}

.launch-split:hover .launch-notch {
  background: #fff;
  box-shadow: 0 0 12px rgba(255, 255, 255, 0.7);
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
  color: #6b6b70;
  filter:
    drop-shadow(-1px -1px 0.5px rgba(0, 0, 0, 0.85))
    drop-shadow(1px 1px 0.5px rgba(255, 255, 255, 0.1));

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
  text-shadow:
    -1px -1px 1px rgba(0, 0, 0, 0.5),
    1px 1px 1px rgba(255, 255, 255, 0.15);

  &--status {
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
    color: rgba(255, 255, 255, 0.9);
    text-shadow: none;
  }

  &--error { color: #fff; text-shadow: none; }
}

.launch-version {
  text-align: left;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 12px;
  font-weight: 500;
  color: #6b6b70;
  text-shadow:
    -1px -1px 1px rgba(0, 0, 0, 0.85),
    1px 1px 1px rgba(255, 255, 255, 0.08);
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
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: color 120ms;
  flex-shrink: 0;

  &:hover:not(:disabled) { color: #fff; }
  &:disabled { cursor: not-allowed; opacity: 0.5; }
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
  border-radius: 0;
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
