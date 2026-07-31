<template>
  <div ref="panelEl" class="edp-panel" :style="panelStyle" @click.stop>
    <div class="edp-header">
      <span class="edp-title">{{ label || id }}</span>
      <button class="edp-close" title="Schließen" @click="$emit('close')">
        <svg width="10" height="10" viewBox="0 0 10 10"><line x1="0.5" y1="0.5" x2="9.5" y2="9.5" stroke="currentColor" stroke-width="1.4"/><line x1="9.5" y1="0.5" x2="0.5" y2="9.5" stroke="currentColor" stroke-width="1.4"/></svg>
      </button>
    </div>

    <div class="edp-body">

      <!-- Quick presets -->
      <div v-if="showPresets" class="edp-section">
        <span class="edp-section-title">Schnellstile</span>
        <div class="edp-presets">
          <button
            v-for="p in STYLE_PRESETS"
            :key="p.name"
            class="edp-preset-chip"
            @click="applyPreset(p.patch)"
          >{{ p.name }}</button>
        </div>
      </div>

      <!-- Form -->
      <div v-if="features.includes('radius') || features.includes('outline')" class="edp-section">
        <span class="edp-section-title">Form</span>

        <div v-if="features.includes('radius')" class="edp-row">
          <span class="edp-label">Ecken</span>
          <div class="edp-seg">
            <button class="edp-seg-btn" :class="{ active: (override.radius ?? 'default') === 'sharp' }" @click="patch({ radius: 'sharp' })">Eckig</button>
            <button class="edp-seg-btn" :class="{ active: (override.radius ?? 'default') === 'default' }" @click="patch({ radius: 'default' })">Standard</button>
            <button class="edp-seg-btn" :class="{ active: (override.radius ?? 'default') === 'rounded' }" @click="patch({ radius: 'rounded' })">Rund</button>
          </div>
        </div>

        <div v-if="features.includes('outline')" class="edp-row">
          <span class="edp-label">Outline</span>
          <div class="edp-inline">
            <div class="edp-toggle" :class="{ on: !!override.outline }" @click="patch({ outline: !override.outline })" />
            <input
              type="color"
              class="edp-color-input"
              :disabled="!override.outline"
              :value="override.outlineColor || '#3159a3'"
              @input="patch({ outlineColor: ($event.target as HTMLInputElement).value })"
            />
            <input
              type="range" min="1" max="6" step="1"
              class="edp-slider"
              :disabled="!override.outline"
              :value="override.outlineWidth ?? 2"
              @input="patch({ outlineWidth: Number(($event.target as HTMLInputElement).value) })"
            />
            <span class="edp-slider-value">{{ override.outlineWidth ?? 2 }}px</span>
          </div>
        </div>
      </div>

      <!-- Color & typography -->
      <div v-if="features.includes('color') || features.includes('bgColor') || features.includes('fontFamily')" class="edp-section">
        <span class="edp-section-title">Farbe &amp; Typografie</span>

        <div v-if="features.includes('color')" class="edp-row">
          <span class="edp-label">Textfarbe</span>
          <div class="edp-inline">
            <input
              type="color"
              class="edp-color-input"
              :value="override.color || '#f0f0f0'"
              @input="patch({ color: ($event.target as HTMLInputElement).value })"
            />
            <button class="edp-mini-btn" @click="patch({ color: undefined })">Standard</button>
          </div>
        </div>

        <div v-if="features.includes('bgColor')" class="edp-row">
          <span class="edp-label">Hintergrundfarbe</span>
          <div class="edp-inline">
            <input
              type="color"
              class="edp-color-input"
              :value="override.bgColor || '#222222'"
              @input="patch({ bgColor: ($event.target as HTMLInputElement).value })"
            />
            <button class="edp-mini-btn" @click="patch({ bgColor: undefined })">Standard</button>
          </div>
        </div>

        <div v-if="features.includes('fontFamily')" class="edp-row">
          <span class="edp-label">Schriftart</span>
          <select
            class="edp-select"
            :value="override.fontFamily || ''"
            @change="patch({ fontFamily: ($event.target as HTMLSelectElement).value || undefined })"
          >
            <option v-for="f in AVAILABLE_FONTS" :key="f.label" :value="f.value">{{ f.label }}</option>
          </select>
        </div>

        <div v-if="features.includes('fontFamily')" class="edp-row">
          <span class="edp-label">Schriftgröße</span>
          <div class="edp-inline">
            <input
              type="range" min="8" max="32" step="1"
              class="edp-slider"
              :value="override.fontSize ?? 13"
              @input="patch({ fontSize: Number(($event.target as HTMLInputElement).value) })"
            />
            <span class="edp-slider-value">{{ override.fontSize ?? 'Std.' }}</span>
          </div>
        </div>

        <div v-if="features.includes('fontFamily')" class="edp-row">
          <span class="edp-label">Schriftstärke</span>
          <div class="edp-seg">
            <button class="edp-seg-btn" :class="{ active: (override.fontWeight ?? 'normal') === 'normal' }" @click="patch({ fontWeight: 'normal' })">Normal</button>
            <button class="edp-seg-btn" :class="{ active: override.fontWeight === 'medium' }" @click="patch({ fontWeight: 'medium' })">Medium</button>
            <button class="edp-seg-btn" :class="{ active: override.fontWeight === 'bold' }" @click="patch({ fontWeight: 'bold' })">Fett</button>
          </div>
        </div>
      </div>

      <!-- Effects -->
      <div class="edp-section">
        <span class="edp-section-title">Effekte</span>

        <div class="edp-row">
          <span class="edp-label">Deckkraft</span>
          <div class="edp-inline">
            <input
              type="range" min="10" max="100" step="5"
              class="edp-slider"
              :value="override.opacity ?? 100"
              @input="patch({ opacity: Number(($event.target as HTMLInputElement).value) })"
            />
            <span class="edp-slider-value">{{ override.opacity ?? 100 }}%</span>
          </div>
        </div>

        <div class="edp-row">
          <span class="edp-label">Weichzeichnen</span>
          <div class="edp-inline">
            <input
              type="range" min="0" max="24" step="1"
              class="edp-slider"
              :value="override.blur ?? 0"
              @input="patch({ blur: Number(($event.target as HTMLInputElement).value) || undefined })"
            />
            <span class="edp-slider-value">{{ override.blur ?? 0 }}px</span>
          </div>
        </div>

        <div class="edp-row">
          <span class="edp-label">Schatten</span>
          <div class="edp-seg">
            <button class="edp-seg-btn" :class="{ active: (override.shadow ?? 'none') === 'none' }" @click="patch({ shadow: 'none' })">Aus</button>
            <button class="edp-seg-btn" :class="{ active: override.shadow === 'soft' }" @click="patch({ shadow: 'soft' })">Weich</button>
            <button class="edp-seg-btn" :class="{ active: override.shadow === 'glow' }" @click="patch({ shadow: 'glow' })">Glow</button>
            <button class="edp-seg-btn" :class="{ active: override.shadow === 'strong' }" @click="patch({ shadow: 'strong' })">Stark</button>
          </div>
        </div>
      </div>

      <!-- Background media -->
      <div v-if="features.includes('bgVideo') || features.includes('bgImage')" class="edp-section">
        <span class="edp-section-title">Hintergrundmedien</span>

        <div v-if="features.includes('bgVideo')" class="edp-row">
          <span class="edp-label">Hintergrundvideo</span>
          <div class="edp-inline">
            <button class="edp-mini-btn" :disabled="pickingMedia" @click="pickMedia('video')">
              {{ override.bgVideo ? 'Video ändern…' : 'Video wählen…' }}
            </button>
            <button v-if="override.bgVideo" class="edp-mini-btn" @click="patch({ bgVideo: undefined })">Entfernen</button>
          </div>
        </div>

        <div v-if="features.includes('bgImage')" class="edp-row">
          <span class="edp-label">Hintergrundbild</span>
          <div class="edp-inline">
            <button class="edp-mini-btn" :disabled="pickingMedia" @click="pickMedia('image')">
              {{ override.bgImage ? 'Bild ändern…' : 'Bild wählen…' }}
            </button>
            <button v-if="override.bgImage" class="edp-mini-btn" @click="patch({ bgImage: undefined })">Entfernen</button>
          </div>
        </div>
      </div>

      <!-- Visibility -->
      <div class="edp-section">
        <span class="edp-section-title">Sichtbarkeit</span>
        <div class="edp-row">
          <span class="edp-label">Element ausblenden</span>
          <div class="edp-toggle" :class="{ on: !!override.hidden }" @click="patch({ hidden: !override.hidden })" />
        </div>
      </div>

      <!-- Position / size -->
      <div v-if="hasMoved" class="edp-section">
        <span class="edp-section-title">Position &amp; Größe</span>
        <div class="edp-row">
          <span class="edp-label">Verschoben / skaliert</span>
          <button class="edp-mini-btn" @click="patch({ offsetX: 0, offsetY: 0, scale: 1 })">Zurücksetzen</button>
        </div>
      </div>

    </div>

    <div class="edp-footer">
      <button class="edp-reset-btn" :disabled="!hasOverride" @click="reset">Alles zurücksetzen</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useElementStyle, AVAILABLE_FONTS, STYLE_PRESETS } from '../../composables/useElementStyle'
import type { ElementOverride } from '../../types'

const props = defineProps<{
  id: string
  label?: string
  features: (keyof ElementOverride)[]
  anchor: HTMLElement | null
}>()

const emit = defineEmits<{ close: [] }>()

const { override, hasOverride, patch, applyPreset, reset } = useElementStyle(props.id)

const showPresets = computed(() =>
  props.features.includes('radius') || props.features.includes('outline') || props.features.includes('bgColor')
)

const hasMoved = computed(() =>
  (override.value.offsetX ?? 0) !== 0 ||
  (override.value.offsetY ?? 0) !== 0 ||
  (override.value.scale ?? 1) !== 1
)

const panelEl = ref<HTMLElement | null>(null)
const pickingMedia = ref(false)

const panelStyle = computed(() => {
  if (!props.anchor) return {}
  const rect = props.anchor.getBoundingClientRect()
  const width = 268
  let left = rect.right - width
  left = Math.max(8, Math.min(left, window.innerWidth - width - 8))
  const maxHeight = window.innerHeight - 16
  const top = Math.min(rect.bottom + 6, maxHeight - 420)
  return { left: `${left}px`, top: `${Math.max(8, top)}px`, width: `${width}px` }
})

async function pickMedia(kind: 'video' | 'image') {
  pickingMedia.value = true
  try {
    const url = await window.api.video.pickCustomBg(kind)
    if (url) patch(kind === 'video' ? { bgVideo: url } : { bgImage: url })
  } finally {
    pickingMedia.value = false
  }
}

let closeHandler: ((e: MouseEvent) => void) | null = null

onMounted(() => {
  closeHandler = (e: MouseEvent) => {
    const t = e.target as Node
    if (panelEl.value && !panelEl.value.contains(t) && props.anchor && !props.anchor.contains(t)) {
      emit('close')
    }
  }
  document.addEventListener('mousedown', closeHandler, true)
})

onUnmounted(() => {
  if (closeHandler) document.removeEventListener('mousedown', closeHandler, true)
})
</script>

<style lang="scss" scoped>
.edp-panel {
  position: fixed;
  z-index: 3000;
  background: rgba(15, 15, 15, 0.97);
  backdrop-filter: blur(16px);
  border: 1px solid rgba(120, 170, 255, 0.35);
  border-radius: 10px;
  box-shadow: 0 20px 48px rgba(0, 0, 0, 0.6);
  font-size: 12px;
  overflow: hidden;
  animation: edp-in 130ms cubic-bezier(0.2, 0, 0, 1) both;
}

@keyframes edp-in {
  from { opacity: 0; transform: translateY(-4px) scale(0.98); }
  to   { opacity: 1; transform: translateY(0) scale(1); }
}

.edp-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.02);
}

.edp-title {
  font-weight: 700;
  font-size: 11.5px;
  letter-spacing: 0.02em;
  color: #cfe0ff;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.edp-close {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  border-radius: 5px;
  &:hover { background: rgba(255, 255, 255, 0.1); color: #fff; }
}

.edp-body {
  padding: 4px 12px 10px;
  display: flex;
  flex-direction: column;
  max-height: 420px;
  overflow-y: auto;
  scrollbar-width: thin;
}

.edp-section {
  padding: 10px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  flex-direction: column;
  gap: 9px;

  &:last-child { border-bottom: none; }
}

.edp-section-title {
  color: #8fb4ff;
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.edp-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.edp-label {
  color: rgba(255, 255, 255, 0.65);
  font-size: 11px;
  flex-shrink: 0;
}

.edp-inline {
  display: flex;
  align-items: center;
  gap: 6px;
}

.edp-presets {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.edp-preset-chip {
  padding: 5px 10px;
  background: rgba(120, 170, 255, 0.1);
  border: 1px solid rgba(120, 170, 255, 0.3);
  color: #cfe0ff;
  border-radius: 999px;
  cursor: pointer;
  font-size: 11px;
  font-weight: 600;
  transition: background 100ms ease, transform 100ms ease;

  &:hover { background: rgba(120, 170, 255, 0.22); transform: translateY(-1px); }
  &:active { transform: translateY(0); }
}

.edp-seg {
  display: flex;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  overflow: hidden;
}

.edp-seg-btn {
  flex: 1;
  padding: 5px 7px;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  font-size: 10.5px;
  white-space: nowrap;
  &.active { background: rgba(49, 89, 163, 0.55); color: #fff; }
  &:hover:not(.active) { background: rgba(255, 255, 255, 0.06); }
}

.edp-toggle {
  width: 32px;
  height: 18px;
  border-radius: 9px;
  background: rgba(255, 255, 255, 0.12);
  cursor: pointer;
  position: relative;
  flex-shrink: 0;
  transition: background 120ms ease;

  &::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #fff;
    transition: transform 120ms ease;
  }

  &.on { background: var(--accent, #3159a3); }
  &.on::after { transform: translateX(14px); }
}

.edp-color-input {
  width: 26px;
  height: 22px;
  padding: 0;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 4px;
  background: none;
  cursor: pointer;
  flex-shrink: 0;
  &:disabled { opacity: 0.35; cursor: not-allowed; }
}

.edp-slider {
  width: 78px;
  accent-color: var(--accent, #3159a3);
  flex-shrink: 0;
}

.edp-slider-value {
  min-width: 32px;
  text-align: right;
  color: rgba(255, 255, 255, 0.55);
  font-size: 10.5px;
  flex-shrink: 0;
}

.edp-mini-btn {
  padding: 5px 8px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.75);
  border-radius: 5px;
  cursor: pointer;
  font-size: 11px;
  white-space: nowrap;
  &:hover:not(:disabled) { background: rgba(255, 255, 255, 0.12); color: #fff; }
  &:disabled { opacity: 0.5; cursor: not-allowed; }
}

.edp-select {
  width: 100%;
  padding: 5px 6px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #fff;
  border-radius: 5px;
  font-size: 11px;
}

.edp-footer {
  padding: 9px 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.edp-reset-btn {
  width: 100%;
  padding: 7px;
  background: none;
  border: 1px solid rgba(255, 90, 90, 0.35);
  color: #ff8a8a;
  border-radius: 5px;
  cursor: pointer;
  font-size: 11px;
  font-weight: 600;
  &:hover:not(:disabled) { background: rgba(255, 90, 90, 0.1); }
  &:disabled { opacity: 0.35; cursor: not-allowed; }
}
</style>
