<template>
  <div class="bc-slider" :class="{ disabled }">
    <div class="bc-slider-track" :style="{ '--fill': fillPercent + '%' }">
      <input
        type="range"
        :min="min"
        :max="max"
        :step="step"
        :value="modelValue"
        :disabled="disabled"
        class="bc-slider-input"
        @input="onInput"
        @change="emit('change')"
        @pointerdown="dragging = true"
        @pointerup="dragging = false"
        @pointercancel="dragging = false"
      />
    </div>
    <Transition name="bubble-pop">
      <div v-if="dragging" class="bc-slider-bubble" :style="{ left: fillPercent + '%' }">
        {{ format ? format(modelValue) : modelValue }}
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue: number
    min: number
    max: number
    step?: number
    disabled?: boolean
    format?: (v: number) => string
  }>(),
  { step: 1, disabled: false },
)

const emit = defineEmits<{ 'update:modelValue': [value: number]; change: [] }>()

const dragging = ref(false)

const fillPercent = computed(() => {
  const range = props.max - props.min
  if (range <= 0) return 0
  return ((props.modelValue - props.min) / range) * 100
})

function onInput(e: Event) {
  emit('update:modelValue', Number((e.target as HTMLInputElement).value))
}
</script>

<style lang="scss" scoped>
.bc-slider {
  position: relative;

  &.disabled {
    opacity: 0.4;
    pointer-events: none;
  }
}

.bc-slider-track {
  position: relative;
  height: 20px;
  display: flex;
  align-items: center;

  &::before {
    content: '';
    position: absolute;
    left: 0;
    right: 0;
    height: 4px;
    border-radius: 4px;
    background: linear-gradient(
      to right,
      var(--accent, #{$primary}) var(--fill),
      $border-strong var(--fill)
    );
    pointer-events: none;
  }
}

.bc-slider-input {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 20px;
  background: transparent;
  margin: 0;
  cursor: pointer;

  &::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 15px;
    height: 15px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.5);
    transition: transform 120ms $ease-out, box-shadow 120ms $ease-out;
    cursor: pointer;
  }

  &:hover::-webkit-slider-thumb {
    transform: scale(1.15);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.5), 0 0 0 5px color-mix(in srgb, var(--accent, #{$primary}) 16%, transparent);
  }

  &:active::-webkit-slider-thumb {
    transform: scale(1.25);
    box-shadow: 0 0 0 6px color-mix(in srgb, var(--accent, #{$primary}) 25%, transparent);
  }

  &::-moz-range-thumb {
    width: 15px;
    height: 15px;
    border: none;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.5);
    cursor: pointer;
  }

  &::-moz-range-track {
    height: 4px;
    background: transparent;
  }
}

.bc-slider-bubble {
  position: absolute;
  top: -26px;
  transform: translateX(-50%);
  background: $surface-elevated;
  border: 1px solid $border-strong;
  color: $text-primary;
  font-size: 11px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 4px;
  white-space: nowrap;
  pointer-events: none;
  box-shadow: $shadow-sm;
}

.bubble-pop-enter-active,
.bubble-pop-leave-active {
  transition: opacity 100ms ease, transform 100ms ease;
}
.bubble-pop-enter-from,
.bubble-pop-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(4px);
}
</style>
