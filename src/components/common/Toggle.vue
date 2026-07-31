<template>
  <label class="bc-toggle" :class="{ disabled }">
    <input
      type="checkbox"
      :checked="modelValue"
      :disabled="disabled"
      @change="onChange"
    />
    <img :src="modelValue ? toggleOn : toggleOff" alt="" class="track" draggable="false" />
    <span v-if="label" class="toggle-label">{{ label }}</span>
  </label>
</template>

<script setup lang="ts">
import { playToggle } from '../../composables/useSounds'
import toggleOn from '../../assets/icons/toggle-on.png'
import toggleOff from '../../assets/icons/toggle-off.png'

withDefaults(
  defineProps<{ modelValue: boolean; label?: string; disabled?: boolean }>(),
  { disabled: false },
)
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()

function onChange(e: Event) {
  emit('update:modelValue', (e.target as HTMLInputElement).checked)
  playToggle()
}
</script>

<style lang="scss" scoped>
.bc-toggle {
  display: inline-flex;
  align-items: center;
  gap: $sp-2;
  cursor: pointer;

  &.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  input {
    display: none;
  }
}

.track {
  width: 34px;
  height: auto;
  aspect-ratio: 768 / 511;
  flex-shrink: 0;
  image-rendering: pixelated;
  transition: filter 160ms $ease-out, transform 160ms $ease-out;
}

.bc-toggle:hover .track {
  transform: scale(1.04);
}

.bc-toggle input:checked ~ .track {
  filter: drop-shadow(0 0 5px color-mix(in srgb, var(--accent, #{$primary}) 45%, transparent));
}

.toggle-label {
  font-size: 13px;
  font-weight: 500;
  color: $text-primary;
}
</style>
