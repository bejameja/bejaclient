<template>
  <div class="settings-section">
    <h2 class="section-heading">Game Settings</h2>
    <p class="section-desc">Configure Minecraft directories, Java, and memory allocation.</p>

    <div class="settings-groups">
      <div class="settings-group">
        <h3 class="group-title">Directories</h3>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Game Directory</span>
            <span class="setting-desc">Where Minecraft files are stored</span>
          </div>
          <div class="setting-control dir-control">
            <span class="dir-path">{{ truncatePath(settings.game.defaultGameDir) }}</span>
            <Button variant="secondary" size="sm" @click="chooseDir">Browse</Button>
          </div>
        </div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Java Executable</span>
            <span class="setting-desc">Leave empty to auto-detect</span>
          </div>
          <div class="setting-control dir-control">
            <span class="dir-path">{{ settings.game.defaultJavaPath || 'Auto-detect' }}</span>
            <Button variant="secondary" size="sm" @click="chooseJava">Browse</Button>
          </div>
        </div>
      </div>

      <div class="settings-group">
        <h3 class="group-title">Memory</h3>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Minimum RAM</span>
            <span class="setting-desc">Initial memory allocation</span>
          </div>
          <div class="setting-control">
            <span class="ram-value">{{ formatRam(settings.game.minRam) }}</span>
            <input
              v-model.number="settings.game.minRam"
              type="range"
              min="256"
              max="2048"
              step="256"
              class="ram-slider"
              @change="save"
            />
          </div>
        </div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Maximum RAM</span>
            <span class="setting-desc">Maximum memory allocation</span>
          </div>
          <div class="setting-control">
            <span class="ram-value">{{ formatRam(settings.game.maxRam) }}</span>
            <input
              v-model.number="settings.game.maxRam"
              type="range"
              min="512"
              max="16384"
              step="256"
              class="ram-slider"
              @change="save"
            />
          </div>
        </div>
      </div>

      <div class="settings-group">
        <h3 class="group-title">Resolution</h3>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Window Size</span>
            <span class="setting-desc">Default game window resolution</span>
          </div>
          <div class="setting-control res-control">
            <Input
              v-model="settings.game.resolution.width"
              type="number"
              style="width: 80px"
              @blur="save"
            />
            <span class="res-sep">×</span>
            <Input
              v-model="settings.game.resolution.height"
              type="number"
              style="width: 80px"
              @blur="save"
            />
          </div>
        </div>
      </div>

      <div class="settings-group">
        <h3 class="group-title">JVM Arguments</h3>
        <div class="setting-row vertical">
          <span class="setting-label">Additional JVM flags</span>
          <Input
            v-model="settings.game.jvmArgs"
            placeholder="-XX:+UseG1GC -XX:MaxGCPauseMillis=200"
            @blur="save"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import Button from '../../components/common/Button.vue'
import Input from '../../components/common/Input.vue'
import { useSettingsStore } from '../../store/settingsStore'

const settingsStore = useSettingsStore()
const settings = computed(() => settingsStore.settings)

async function save() {
  await settingsStore.save()
}

async function chooseDir() {
  await settingsStore.chooseGameDir()
}

async function chooseJava() {
  await settingsStore.chooseJava()
}

function formatRam(mb: number) {
  return mb >= 1024 ? `${(mb / 1024).toFixed(mb % 1024 === 0 ? 0 : 1)} GB` : `${mb} MB`
}

function truncatePath(path: string) {
  if (path.length <= 42) return path
  const parts = path.split(/[/\\]/)
  return '...' + [parts[parts.length - 2], parts[parts.length - 1]].join('/')
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

.settings-groups {
  display: flex;
  flex-direction: column;
  gap: $sp-6;
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
  &.vertical { flex-direction: column; align-items: flex-start; gap: $sp-2; }
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
  flex-shrink: 0;

  &.dir-control {
    max-width: 280px;
    min-width: 0;
  }

  &.res-control {
    align-items: center;
  }
}

.dir-path {
  font-size: 11px;
  color: $text-secondary;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}

.ram-value {
  font-size: 12px;
  font-weight: 600;
  color: $text-primary;
  min-width: 52px;
  text-align: right;
}

.ram-slider {
  width: 120px;
  accent-color: $primary;
  cursor: pointer;
}

.res-sep {
  font-size: 14px;
  color: $muted;
}
</style>
