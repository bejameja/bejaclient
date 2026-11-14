<template>
  <div class="settings-section">
    <h2 class="section-heading">Launcher Settings</h2>
    <p class="section-desc">Configure launcher behavior and updates.</p>

    <div class="settings-group">
      <h3 class="group-title">Behavior</h3>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Close on Launch</span>
          <span class="setting-desc">Hide launcher when game starts</span>
        </div>
        <Toggle v-model="settings.launcher.closeOnLaunch" @update:model-value="save" />
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Keep Launcher Open</span>
          <span class="setting-desc">Reopen launcher when game closes</span>
        </div>
        <Toggle v-model="settings.launcher.keepLauncherOpen" @update:model-value="save" />
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Auto Update</span>
          <span class="setting-desc">Automatically check for launcher updates</span>
        </div>
        <Toggle v-model="settings.launcher.autoUpdate" @update:model-value="save" />
      </div>
    </div>

    <div class="settings-group">
      <h3 class="group-title">Downloads</h3>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Concurrent Downloads</span>
          <span class="setting-desc">Parallel download threads</span>
        </div>
        <div class="setting-control">
          <span class="value-display">{{ settings.launcher.concurrentDownloads }}</span>
          <Slider
            v-model="settings.launcher.concurrentDownloads"
            :min="1"
            :max="32"
            :step="1"
            class="ram-slider"
            @change="save"
          />
        </div>
      </div>
    </div>

    <div class="settings-group">
      <h3 class="group-title">BejaClient Version (MC 1.21.x)</h3>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Adapter / Sodium Version</span>
          <span class="setting-desc">
            Pin a specific release if the latest one conflicts with mods like Iris. Pinned
            versions are never auto-updated.
          </span>
        </div>
        <div class="setting-control">
          <select
            class="version-select"
            :disabled="versionsLoading || installing"
            :value="pinnedVersion ?? ''"
            @change="onVersionPicked(($event.target as HTMLSelectElement).value)"
          >
            <option value="">Latest (auto-update)</option>
            <option v-for="v in versions" :key="v.version" :value="v.version">
              {{ v.version }}{{ v.isCurrent ? ' (current)' : '' }} - {{ v.releaseDate }}
            </option>
          </select>
          <span v-if="installing" class="version-status">installing…</span>
        </div>
      </div>
    </div>

    <div class="settings-group">
      <h3 class="group-title">World Backups</h3>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Auto-backup Worlds</span>
          <span class="setting-desc">Zip every singleplayer world right after each play session</span>
        </div>
        <Toggle v-model="settings.launcher.autoBackupWorlds" @update:model-value="save" />
      </div>
      <div v-if="settings.launcher.autoBackupWorlds" class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Keep Last</span>
          <span class="setting-desc">Older automatic backups beyond this count are deleted per world</span>
        </div>
        <div class="setting-control">
          <span class="value-display">{{ settings.launcher.autoBackupRetention }}</span>
          <Slider v-model="settings.launcher.autoBackupRetention" :min="1" :max="10" :step="1" @change="save" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import Toggle from '../../components/common/Toggle.vue'
import Slider from '../../components/common/Slider.vue'
import { useSettingsStore } from '../../store/settingsStore'
import { showToast } from '../../composables/useToasts'
import type { ClientJarVersion } from '../../types'

const settingsStore = useSettingsStore()
const settings = computed(() => settingsStore.settings)

async function save() {
  await settingsStore.save()
}


const CLIENT_FAMILY = 'v1_21'
const versions = ref<ClientJarVersion[]>([])
const versionsLoading = ref(false)
const installing = ref(false)
const pinnedVersion = ref<string | null>(null)

async function loadVersions() {
  versionsLoading.value = true
  try {
    const [list, pinned] = await Promise.all([
      window.api.clientVersions.list(CLIENT_FAMILY),
      window.api.clientVersions.pinned(CLIENT_FAMILY),
    ])
    versions.value = list
    pinnedVersion.value = pinned
  } catch (e) {
    showToast({ title: 'Could not load client versions', body: String(e), variant: 'error' })
  } finally {
    versionsLoading.value = false
  }
}

async function onVersionPicked(version: string) {
  installing.value = true
  try {
    if (version === '') {
      await window.api.clientVersions.useLatest(CLIENT_FAMILY)
      pinnedVersion.value = null
      showToast({ title: 'Switched to latest BejaClient version', variant: 'success' })
    } else {
      await window.api.clientVersions.install(CLIENT_FAMILY, version)
      pinnedVersion.value = version
      showToast({ title: `Pinned BejaClient to ${version}`, variant: 'success' })
    }
  } catch (e) {
    showToast({ title: 'Failed to switch client version', body: String(e), variant: 'error' })
  } finally {
    installing.value = false
  }
}

onMounted(loadVersions)
</script>

<style lang="scss" scoped>
.settings-section {
  display: flex;
  flex-direction: column;
  gap: $sp-6;
  max-width: 640px;
}

.section-heading {
  font-size: 18px;
  font-weight: 800;
  color: $text-primary;
}

.section-desc {
  font-size: 13px;
  color: $muted;
  margin-top: -$sp-4;
}

.settings-group {
  display: flex;
  flex-direction: column;
  gap: $sp-1;
}

.group-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: $muted;
  padding-bottom: $sp-2;
  border-bottom: 1px solid $border;
  margin-bottom: $sp-2;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $sp-4;
  padding: $sp-3 0;
  border-bottom: 1px solid $border;

  &:last-child { border-bottom: none; }
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.setting-label {
  font-size: 13px;
  font-weight: 600;
  color: $text-primary;
}

.setting-desc {
  font-size: 11px;
  color: $muted;
}

.setting-control {
  display: flex;
  align-items: center;
  gap: $sp-2;
}

.value-display {
  font-size: 12px;
  font-weight: 600;
  color: $text-primary;
  min-width: 28px;
  text-align: right;
}

.ram-slider {
  width: 120px;
  accent-color: $primary;
  cursor: pointer;
}

.sound-style-picker {
  display: flex;
  border: 1px solid $border;
  border-radius: $radius;
  overflow: hidden;
}

.style-opt {
  padding: 5px 16px;
  font-size: 12px;
  font-weight: 500;
  color: $text-secondary;
  background: transparent;
  border: none;
  cursor: pointer;
  transition: background $transition, color $transition;

  & + & {
    border-left: 1px solid $border;
  }

  &:hover {
    background: $surface-elevated;
    color: $text-primary;
  }

  &.active {
    background: $border;
    color: $text-primary;
    font-weight: 600;
  }
}

.version-select {
  padding: 6px 10px;
  font-size: 12px;
  color: $text-primary;
  background: $surface-elevated;
  border: 1px solid $border;
  border-radius: $radius;
  cursor: pointer;
  min-width: 220px;

  &:disabled { opacity: 0.6; cursor: wait; }
}

.version-status {
  font-size: 11px;
  color: $muted;
  white-space: nowrap;
}

.test-sound-btn {
  padding: 4px 10px;
  background: $surface-elevated;
  border: 1px solid $border;
  border-radius: 12px;
  color: $text-secondary;
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: background $transition, color $transition;
  white-space: nowrap;

  &:hover {
    background: $border;
    color: $text-primary;
  }
}
</style>
