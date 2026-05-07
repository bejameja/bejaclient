<template>
  <div class="settings-section">
    <h2 class="section-heading">Appearance</h2>
    <p class="section-desc">Customize how BejaClient looks.</p>

    <div class="settings-group">
      <h3 class="group-title">Accent Color</h3>
      <div class="color-presets">
        <button
          v-for="color in accentColors"
          :key="color.value"
          class="color-swatch"
          :class="{ active: currentAccent === color.value }"
          :style="{ background: color.value }"
          :title="color.label"
          @click="setAccent(color.value)"
        />
        <div class="custom-color">
          <input
            v-model="customColor"
            type="color"
            class="color-picker"
            @input="setAccent(customColor)"
          />
          <span class="custom-label">Custom</span>
        </div>
      </div>
    </div>

    <div class="settings-group">
      <h3 class="group-title">Preview</h3>
      <div class="preview-panel">
        <div class="preview-nav">
          <div class="preview-tab active" :style="{ '--accent': currentAccent }">Home</div>
          <div class="preview-tab">Versions</div>
          <div class="preview-tab">Mods</div>
        </div>
        <div class="preview-btn" :style="{ background: currentAccent }">Launch Game</div>
      </div>
    </div>

    <div class="settings-group">
      <h3 class="group-title">Language</h3>
      <Select
        v-model="settings.appearance.language"
        :options="languages"
        style="max-width: 240px"
        @update:model-value="save"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import Select from '../../components/common/Select.vue'
import { useSettingsStore } from '../../store/settingsStore'

const settingsStore = useSettingsStore()
const settings = computed(() => settingsStore.settings)

const customColor = ref('#F97316')
const currentAccent = computed(() => settings.value.appearance.accentColor)

const accentColors = [
  { label: 'Orange', value: '#F97316' },
  { label: 'Blue', value: '#3B82F6' },
  { label: 'Green', value: '#22C55E' },
  { label: 'Purple', value: '#A855F7' },
  { label: 'Red', value: '#EF4444' },
  { label: 'Cyan', value: '#06B6D4' },
  { label: 'Yellow', value: '#F59E0B' },
]

const languages = [
  { label: 'English', value: 'en' },
  { label: 'Deutsch', value: 'de' },
  { label: 'Français', value: 'fr' },
  { label: 'Español', value: 'es' },
  { label: 'Português', value: 'pt' },
  { label: '中文', value: 'zh' },
  { label: '日本語', value: 'ja' },
]

async function setAccent(color: string) {
  settings.value.appearance.accentColor = color
  document.documentElement.style.setProperty('--accent', color)
  await settingsStore.save()
}

async function save() {
  await settingsStore.save()
}

onMounted(() => {
  customColor.value = currentAccent.value
})
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
  gap: $sp-4;
}

.group-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: $muted;
  padding-bottom: $sp-2;
  border-bottom: 1px solid $border;
}

.color-presets {
  display: flex;
  align-items: center;
  gap: $sp-2;
  flex-wrap: wrap;
}

.color-swatch {
  width: 28px;
  height: 28px;
  border-radius: $radius;
  border: 2px solid transparent;
  cursor: pointer;
  transition: transform $transition, border-color $transition;

  &:hover { transform: scale(1.1); }
  &.active { border-color: #fff; }
}

.custom-color {
  display: flex;
  align-items: center;
  gap: $sp-2;
}

.color-picker {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: $radius;
  cursor: pointer;
  background: none;
  padding: 0;
}

.custom-label {
  font-size: 12px;
  color: $muted;
}

.preview-panel {
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius;
  overflow: hidden;
  padding: $sp-4;
  display: flex;
  flex-direction: column;
  gap: $sp-3;
}

.preview-nav {
  display: flex;
  gap: $sp-1;
}

.preview-tab {
  padding: $sp-1 $sp-3;
  font-size: 12px;
  font-weight: 600;
  color: $muted;
  border-bottom: 2px solid transparent;

  &.active {
    color: $text-primary;
    border-bottom-color: v-bind(currentAccent);
  }
}

.preview-btn {
  align-self: flex-start;
  padding: $sp-2 $sp-5;
  border-radius: $radius;
  font-size: 12px;
  font-weight: 700;
  color: #fff;
}
</style>
