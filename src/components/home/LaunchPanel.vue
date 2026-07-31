<template>
  <!-- Matches Figma instance-version embed (242×162px at 0.30 scale) -->
  <div class="launch-root">

    <!-- Selected instance panel (top portion) -->
    <div class="selected-instance">
      <div class="instance-label">SELECTED INSTANCE:</div>
      <Select
        v-model="selectedProfileId"
        :options="profileOptions"
        placeholder="Select instance…"
        class="instance-select"
      />
      <div class="version-row">
        <span class="version-key">version:</span>
        <span class="version-val">{{ activeProfile?.version ?? '—' }}</span>
        <div class="version-underline" />
        <span class="loader-chip">{{ loaderBadge }}</span>
      </div>
      <div class="ram-row">
        <span class="ram-key">RAM</span>
        <input
          :value="localRam"
          type="range"
          :min="512"
          :max="maxRam"
          :step="256"
          class="ram-slider"
          @input="onRamInput"
        />
        <span class="ram-val">{{ ramLabel }}</span>
      </div>
    </div>

    <!-- Launch row: LAUNCH button + change instance -->
    <div class="launch-row">

      <!-- Account indicator -->
      <div class="account-row">
        <span class="account-dot" :class="{ online: !!account }" />
        <span class="account-name">{{ account?.username ?? 'Not signed in' }}</span>
      </div>

      <div class="btn-row">
        <button
          class="launch-btn"
          :class="{ running: isRunning, launching: isLaunching }"
          :disabled="!canLaunch"
          @click="handleLaunch"
        >
          <span v-if="isLaunching" class="launch-spinner" />
          <span class="launch-text">
            {{ isRunning ? 'STOP' : isLaunching ? '…' : 'LAUNCH' }}
          </span>
        </button>

        <button class="change-btn" title="Next instance" @click="cycleProfile">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="6 9 12 15 18 9"/>
          </svg>
        </button>
      </div>

    </div>

  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Select from '../common/Select.vue'
import { useAccountStore } from '../../store/accountStore'
import { useLauncherStore } from '../../store/launcherStore'
import { useSettingsStore } from '../../store/settingsStore'

const accountStore  = useAccountStore()
const launcherStore = useLauncherStore()
const settingsStore = useSettingsStore()

const account       = computed(() => accountStore.selectedAccount)
const activeProfile = computed(() => launcherStore.activeProfile)
const isRunning     = computed(() => launcherStore.isRunning)
const isLaunching   = computed(() => launcherStore.isLaunching)

const selectedProfileId = ref(activeProfile.value?.id ?? null)
watch(selectedProfileId, async id => { if (id) await launcherStore.setActiveProfile(id) })
watch(() => launcherStore.activeProfile?.id, id => { if (id) selectedProfileId.value = id })

const profileOptions = computed(() =>
  launcherStore.profiles.map(p => ({ label: p.name, value: p.id }))
)
const loaderBadge = computed(() => activeProfile.value?.loader ?? 'vanilla')

function cycleProfile() {
  const profiles = launcherStore.profiles
  if (profiles.length < 2) return
  const idx = profiles.findIndex(p => p.id === selectedProfileId.value)
  selectedProfileId.value = profiles[(idx + 1) % profiles.length].id
}

const maxRam  = computed(() => Math.max(((window.navigator as any)?.deviceMemory ?? 8) * 1024, 8192))
const localRam = ref(activeProfile.value?.maxRam ?? settingsStore.settings.game.maxRam)
watch(() => activeProfile.value?.maxRam, val => { if (val !== undefined) localRam.value = val })

const ramLabel = computed(() => {
  const v = localRam.value
  return v >= 1024 ? `${(v / 1024).toFixed(v % 1024 === 0 ? 0 : 1)}G` : `${v}M`
})

let ramDebounce: ReturnType<typeof setTimeout> | null = null
function onRamInput(e: Event) {
  const val = Number((e.target as HTMLInputElement).value)
  localRam.value = val
  if (ramDebounce) clearTimeout(ramDebounce)
  ramDebounce = setTimeout(() => {
    if (activeProfile.value) launcherStore.updateProfile(activeProfile.value.id, { maxRam: val })
  }, 300)
}

const canLaunch = computed(
  () => (account.value !== null || isRunning.value) &&
        (activeProfile.value !== null || isRunning.value) &&
        !isLaunching.value
)

async function handleLaunch() {
  if (isRunning.value) await launcherStore.killGame()
  else await launcherStore.launch()
}
</script>

<style lang="scss" scoped>
.launch-root {
  width: 237px;
  display: flex;
  flex-direction: column;
}

// ── Selected instance panel ───────────────────────────────────────────────────
.selected-instance {
  background: $surface;
  border: 1px solid $border;
  border-bottom: none;
  padding: 10px 12px 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.instance-label {
  font-size: 12px;
  font-weight: 700;
  color: $text-secondary;
  letter-spacing: 0.04em;
  line-height: 1;
}

.instance-select {
  width: 100%;
}

.version-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.version-key {
  font-size: 11px;
  color: $muted;
  flex-shrink: 0;
}

.version-val {
  font-size: 11px;
  color: $text-primary;
}

.version-underline {
  flex: 1;
  height: 1px;
  background: $border;
}

.loader-chip {
  font-size: 8px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  padding: 2px 5px;
  border-radius: $radius-sm;
  flex-shrink: 0;
  background: $surface-elevated;
  color: $text-secondary;
  border: 1px solid $border;
}

.ram-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.ram-key {
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  color: $muted;
  width: 28px;
  flex-shrink: 0;
}

.ram-slider {
  flex: 1;
  cursor: pointer;
}

.ram-val {
  font-size: 10px;
  font-weight: 600;
  color: $text-primary;
  width: 28px;
  text-align: right;
  flex-shrink: 0;
}

// ── Account indicator ─────────────────────────────────────────────────────────
.account-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  background: $surface-elevated;
  border: 1px solid $border;
  border-bottom: none;
}

.account-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: $border;
  flex-shrink: 0;

  &.online {
    background: $text-secondary;
  }
}

.account-name {
  font-size: 10px;
  color: $muted;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

// ── Launch button row ─────────────────────────────────────────────────────────
.launch-row {
  display: flex;
  flex-direction: column;
}

.btn-row {
  display: flex;
  align-items: stretch;
}

.launch-btn {
  flex: 1;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  background: $text-primary;
  border: none;
  cursor: pointer;
  transition: background $transition;

  &:hover:not(:disabled) {
    background: $text-secondary;
  }

  &:active:not(:disabled) {
    background: $border-strong;
  }

  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  &.running {
    background: $surface-elevated;
    border: 1px solid $border;
    .launch-text { color: $text-primary; font-size: 36px; }
  }

  &.launching {
    background: $border;
    cursor: not-allowed;
  }
}

.launch-text {
  font-size: 60px;
  font-weight: 900;
  color: $bg;
  line-height: 1;
  white-space: nowrap;
}

.launch-spinner {
  width: 18px;
  height: 18px;
  border: 2px solid $border;
  border-top-color: $bg;
  border-radius: 50%;
  animation: spin 0.65s linear infinite;
}

.change-btn {
  width: 51px;
  height: 80px;
  background: $surface-elevated;
  border: 1px solid $border;
  border-left: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: $text-primary;
  transition: background $transition;
  flex-shrink: 0;

  &:hover { background: $border; }
}

@keyframes spin { to { transform: rotate(360deg); } }
</style>
