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
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">UI Sounds</span>
          <span class="setting-desc">Play subtle sounds on hover, click, and launch</span>
        </div>
        <Toggle v-model="settings.launcher.soundEnabled" @update:model-value="save" />
      </div>
      <div v-if="settings.launcher.soundEnabled" class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Sound Style</span>
          <span class="setting-desc">Choose the click sound character</span>
        </div>
        <div class="sound-style-picker">
          <button
            class="style-opt"
            :class="{ active: settings.launcher.soundStyle === 'soft' }"
            @click="settings.launcher.soundStyle = 'soft'; save()"
          >Soft</button>
          <button
            class="style-opt"
            :class="{ active: settings.launcher.soundStyle === 'clicky' }"
            @click="settings.launcher.soundStyle = 'clicky'; save()"
          >Clicky</button>
        </div>
      </div>
      <div v-if="settings.launcher.soundEnabled" class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Sound Volume</span>
          <span class="setting-desc">Adjust UI sound effect volume</span>
        </div>
        <div class="setting-control">
          <span class="value-display">{{ settings.launcher.soundVolume }}%</span>
          <input
            v-model.number="settings.launcher.soundVolume"
            type="range"
            min="0"
            max="100"
            step="5"
            class="ram-slider"
            @change="save"
          />
          <button class="test-sound-btn" @click="testSound">Test</button>
        </div>
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
          <input
            v-model.number="settings.launcher.concurrentDownloads"
            type="range"
            min="1"
            max="32"
            step="1"
            class="ram-slider"
            @change="save"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import Toggle from '../../components/common/Toggle.vue'
import { useSettingsStore } from '../../store/settingsStore'
import { playClick } from '../../composables/useSounds'

const settingsStore = useSettingsStore()
const settings = computed(() => settingsStore.settings)

async function save() {
  await settingsStore.save()
}

function testSound() {
  playClick()
}
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

.test-sound-btn {
  padding: 4px 10px;
  background: $surface-elevated;
  border: 1px solid $border;
  border-radius: $radius;
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
