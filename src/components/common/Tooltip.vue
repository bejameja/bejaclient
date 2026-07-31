<template>
  <span ref="triggerRef" class="tooltip-trigger" @mouseenter="onEnter" @mouseleave="onLeave">
    <slot />
    <Teleport to="body">
      <Transition name="tooltip-pop">
        <div v-if="visible" ref="tipRef" class="tooltip-bubble" :style="{ left: pos.x + 'px', top: pos.y + 'px' }">
          {{ text }}
        </div>
      </Transition>
    </Teleport>
  </span>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref } from 'vue'
import { computePosition, autoUpdate, offset, flip, shift } from '@floating-ui/dom'

const props = withDefaults(
  defineProps<{
    text: string
    placement?: 'top' | 'bottom' | 'left' | 'right'
    delay?: number
    disabled?: boolean
  }>(),
  { placement: 'top', delay: 400, disabled: false },
)

const triggerRef = ref<HTMLElement | null>(null)
const tipRef = ref<HTMLElement | null>(null)
const visible = ref(false)
const pos = ref({ x: 0, y: 0 })

let showTimer: ReturnType<typeof setTimeout> | null = null
let stopAutoUpdate: (() => void) | null = null

function updatePosition() {
  if (!triggerRef.value || !tipRef.value) return
  computePosition(triggerRef.value, tipRef.value, {
    placement: props.placement,
    middleware: [offset(8), flip(), shift({ padding: 8 })],
  }).then((result) => {
    pos.value = { x: result.x, y: result.y }
  })
}

function onEnter() {
  if (props.disabled || !props.text) return
  if (showTimer) clearTimeout(showTimer)
  showTimer = setTimeout(async () => {
    visible.value = true
    await nextTick()
    if (triggerRef.value && tipRef.value) {
      stopAutoUpdate = autoUpdate(triggerRef.value, tipRef.value, updatePosition)
    }
  }, props.delay)
}

function onLeave() {
  if (showTimer) { clearTimeout(showTimer); showTimer = null }
  visible.value = false
  stopAutoUpdate?.()
  stopAutoUpdate = null
}

onBeforeUnmount(() => {
  if (showTimer) clearTimeout(showTimer)
  stopAutoUpdate?.()
})
</script>

<style lang="scss" scoped>
@import '../../styles/motion';

.tooltip-trigger {
  display: contents;
}

.tooltip-bubble {
  @extend %glass-panel;
  position: fixed;
  z-index: 9500;
  color: $text-primary;
  font-size: 12px;
  font-weight: 500;
  padding: 5px 10px;
  border-radius: $radius-sm;
  white-space: nowrap;
  pointer-events: none;
}
</style>
