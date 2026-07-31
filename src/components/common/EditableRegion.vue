<template>
  <div
    v-show="!rootHidden"
    ref="rootEl"
    class="editable-region"
    :class="{ 'edr-fill': fill, 'edr-flex-fill': flexFill, 'edr-live': dragging || resizing }"
    :style="outerStyle"
  >
    <div
      class="edr-wiggle-layer"
      :class="{ 'edr-wiggle': wiggleActive, 'edr-paused': dragging || resizing }"
      :style="wiggleStyle"
    >
      <slot />

      <div
        v-if="editorModeEnabled"
        class="edr-overlay"
        @pointerdown="onPointerDown"
      >
        <span v-if="override.hidden" class="edr-badge edr-badge--hidden" title="Ausgeblendet — im Panel wieder einblenden">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7-11-7-11-7z" opacity="0.4"/><line x1="2" y1="2" x2="22" y2="22"/>
          </svg>
        </span>
        <span v-else-if="hasOverride" class="edr-badge" title="Angepasst">
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
            <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" />
          </svg>
        </span>
        <div class="edr-resize-handle" @pointerdown.stop="onResizeStart" title="Größe ziehen" />
      </div>
    </div>

    <Teleport to="body">
      <ElementEditorPanel
        v-if="panelOpen"
        :id="id"
        :label="label"
        :features="features"
        :anchor="rootEl"
        @close="panelOpen = false"
      />
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeUnmount } from 'vue'
import { useElementStyle, useEditorMode } from '../../composables/useElementStyle'
import ElementEditorPanel from './ElementEditorPanel.vue'
import type { ElementOverride } from '../../types'

const props = withDefaults(
  defineProps<{
    id: string
    label?: string
    features?: (keyof ElementOverride)[]
    fill?: boolean
    flexFill?: boolean
  }>(),
  {
    features: () => ['radius', 'outline', 'color', 'bgColor', 'fontFamily'],
    fill: false,
    flexFill: false,
  },
)

const { enabled: editorModeEnabled, wiggleEnabled } = useEditorMode()
const { override, cssVars, wrapperStyle, hasOverride, patch } = useElementStyle(props.id)

const rootEl = ref<HTMLElement | null>(null)
const panelOpen = ref(false)

const wiggleActive = computed(() => editorModeEnabled.value && wiggleEnabled.value)
// Fully hidden for normal use; while Editor Mode is on it stays as a dim,
// clickable "ghost" so it can still be found and un-hidden.
const rootHidden = computed(() => !!override.value.hidden && !editorModeEnabled.value)
const isHiddenGhost = computed(() => !!override.value.hidden && editorModeEnabled.value)

// Random per-instance wiggle timing so a whole page of cards doesn't jiggle
// in perfect unison — the same trick iOS uses for home-screen icons. Kept
// deliberately subtle (small angle, slow-ish) after motion-sickness feedback.
const wiggleDuration = (1.1 + Math.random() * 0.2).toFixed(2)
const wiggleDelay = (-(Math.random() * 0.6)).toFixed(2)
const wiggleTilt = (Math.random() > 0.5 ? 1 : -1) * (0.15 + Math.random() * 0.1)

function openPanel() {
  panelOpen.value = true
}

// ── Drag to reposition ───────────────────────────────────────────────────────
const dragging = ref(false)
const liveOffsetX = ref(0)
const liveOffsetY = ref(0)
let dragStartX = 0
let dragStartY = 0
let baseOffsetX = 0
let baseOffsetY = 0
let didDrag = false
const DRAG_THRESHOLD = 5

function onPointerDown(e: PointerEvent) {
  if (e.button !== undefined && e.button !== 0) return
  const target = e.currentTarget as HTMLElement
  dragStartX = e.clientX
  dragStartY = e.clientY
  baseOffsetX = override.value.offsetX ?? 0
  baseOffsetY = override.value.offsetY ?? 0
  liveOffsetX.value = baseOffsetX
  liveOffsetY.value = baseOffsetY
  didDrag = false
  target.setPointerCapture(e.pointerId)
  target.addEventListener('pointermove', onPointerMove)
  target.addEventListener('pointerup', onPointerUp, { once: true })
}

function onPointerMove(e: PointerEvent) {
  const dx = e.clientX - dragStartX
  const dy = e.clientY - dragStartY
  if (!didDrag && Math.hypot(dx, dy) > DRAG_THRESHOLD) {
    didDrag = true
    dragging.value = true
  }
  if (didDrag) {
    liveOffsetX.value = baseOffsetX + dx
    liveOffsetY.value = baseOffsetY + dy
  }
}

function onPointerUp(e: PointerEvent) {
  const target = e.currentTarget as HTMLElement
  target.removeEventListener('pointermove', onPointerMove)
  if (didDrag) {
    patch({ offsetX: liveOffsetX.value, offsetY: liveOffsetY.value })
    dragging.value = false
  } else {
    openPanel()
  }
}

// ── Drag to resize (bottom-right handle) ─────────────────────────────────────
const resizing = ref(false)
const liveScale = ref(1)
let resizeStartX = 0
let resizeBaseWidth = 100
let resizeBaseScale = 1

function onResizeStart(e: PointerEvent) {
  const target = e.currentTarget as HTMLElement
  resizeStartX = e.clientX
  resizeBaseScale = override.value.scale ?? 1
  resizeBaseWidth = (rootEl.value?.offsetWidth || 200) / resizeBaseScale
  liveScale.value = resizeBaseScale
  resizing.value = true
  target.setPointerCapture(e.pointerId)
  target.addEventListener('pointermove', onResizeMove)
  target.addEventListener('pointerup', onResizeEnd, { once: true })
}

function onResizeMove(e: PointerEvent) {
  const dx = e.clientX - resizeStartX
  const next = resizeBaseScale + dx / Math.max(resizeBaseWidth, 40)
  liveScale.value = Math.max(0.4, Math.min(2.5, next))
}

function onResizeEnd(e: PointerEvent) {
  const target = e.currentTarget as HTMLElement
  target.removeEventListener('pointermove', onResizeMove)
  patch({ scale: liveScale.value })
  resizing.value = false
}

onBeforeUnmount(() => {
  dragging.value = false
  resizing.value = false
})

// ── Combined transform (position + scale) — kept on the OUTER element so the
// wiggle keyframes (which animate the INNER layer's transform) never fight it.
const outerStyle = computed(() => {
  const ox = dragging.value ? liveOffsetX.value : (override.value.offsetX ?? 0)
  const oy = dragging.value ? liveOffsetY.value : (override.value.offsetY ?? 0)
  const sc = resizing.value ? liveScale.value : (override.value.scale ?? 1)
  const style: Record<string, string> = { ...cssVars.value, ...wrapperStyle.value }
  if (ox || oy || sc !== 1) {
    style.transform = `translate(${ox}px, ${oy}px) scale(${sc})`
    style.transformOrigin = 'top left'
  }
  if (dragging.value || resizing.value) style.zIndex = '80'
  if (isHiddenGhost.value) style.opacity = '0.35'
  return style
})

const wiggleStyle = computed(() => ({
  '--edr-wiggle-duration': `${wiggleDuration}s`,
  '--edr-wiggle-delay': `${wiggleDelay}s`,
  '--edr-wiggle-tilt': `${wiggleTilt}deg`,
}))
</script>

<style lang="scss" scoped>
.editable-region {
  display: block;
  position: relative;
  flex-shrink: 0;
  transition: transform 140ms cubic-bezier(0.2, 0, 0, 1), opacity 140ms ease, box-shadow 140ms ease;

  &.edr-fill { height: 100%; }
  &.edr-flex-fill { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  // No transition while actively dragging/resizing — the wrapper must track
  // the pointer 1:1, a settle-transition here reads as input lag.
  &.edr-live { transition: opacity 140ms ease; }
}

.edr-wiggle-layer {
  position: relative;
  height: 100%;

  .edr-flex-fill > & { flex: 1; min-height: 0; display: flex; flex-direction: column; }
}

@keyframes edr-wiggle {
  0%, 100% { transform: rotate(calc(var(--edr-wiggle-tilt, 1.5deg) * -1)); }
  50%      { transform: rotate(var(--edr-wiggle-tilt, 1.5deg)); }
}

.edr-wiggle {
  animation: edr-wiggle var(--edr-wiggle-duration, 0.6s) ease-in-out var(--edr-wiggle-delay, 0s) infinite;
  will-change: transform;

  html.reduce-motion & { animation: none; }
}

.edr-paused { animation-play-state: paused; }

.edr-overlay {
  position: absolute;
  inset: 0;
  z-index: 30;
  cursor: grab;
  border-radius: inherit;
  outline: 1px dashed rgba(120, 170, 255, 0.4);
  outline-offset: -1px;
  transition: background 100ms ease, outline-color 100ms ease;

  &:hover { background: rgba(120, 170, 255, 0.1); outline-color: rgba(120, 170, 255, 0.7); }
  &:active { cursor: grabbing; }
}

.edr-badge {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 19px;
  height: 19px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(49, 89, 163, 0.92);
  border: 1px solid #78aaff;
  color: #eaf1ff;
  border-radius: 5px;
  pointer-events: none;
  z-index: 31;

  &--hidden {
    background: rgba(120, 40, 40, 0.9);
    border-color: #ff9a9a;
  }
}

.edr-resize-handle {
  position: absolute;
  bottom: -5px;
  right: -5px;
  width: 16px;
  height: 16px;
  background: #3159a3;
  border: 2px solid #eaf1ff;
  border-radius: 4px;
  cursor: nwse-resize;
  z-index: 32;
  touch-action: none;
  transition: transform 100ms ease;

  &:hover { transform: scale(1.15); }
}
</style>
