<template>
  <div class="bc-input-wrapper">
    <label v-if="label" class="bc-label">{{ label }}</label>
    <div class="bc-input-inner" :class="{ disabled, focused }">
      <input
        v-bind="$attrs"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :type="type"
        class="bc-input"
        @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        @focus="focused = true"
        @blur="focused = false"
      />
      <slot name="suffix" />
    </div>
    <span v-if="hint" class="bc-hint">{{ hint }}</span>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

withDefaults(
  defineProps<{
    modelValue: string | number
    label?: string
    placeholder?: string
    hint?: string
    disabled?: boolean
    type?: string
  }>(),
  { type: 'text', disabled: false },
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
const focused = ref(false)
</script>

<style lang="scss" scoped>
.bc-input-wrapper {
  display: flex;
  flex-direction: column;
  gap: $sp-2;
}

.bc-label {
  font-size: 11px;
  font-weight: 600;
  color: $muted;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.bc-input-inner {
  display: flex;
  align-items: center;
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius;
  transition: border-color $transition;

  &.focused {
    border-color: $border-strong;
  }

  &.disabled {
    opacity: 0.4;
  }
}

.bc-input {
  flex: 1;
  height: 34px;
  padding: 0 $sp-3;
  background: transparent;
  border: none;
  outline: none;
  font-size: 13px;
  font-weight: 500;
  color: $text-primary;

  &::placeholder {
    color: $muted;
  }

  &:disabled {
    cursor: not-allowed;
  }
}

.bc-hint {
  font-size: 11px;
  color: $muted;
}
</style>
