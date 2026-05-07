<template>
  <label class="bc-toggle" :class="{ disabled }">
    <input
      type="checkbox"
      :checked="modelValue"
      :disabled="disabled"
      @change="emit('update:modelValue', ($event.target as HTMLInputElement).checked)"
    />
    <span class="track">
      <span class="thumb" />
    </span>
    <span v-if="label" class="toggle-label">{{ label }}</span>
  </label>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{ modelValue: boolean; label?: string; disabled?: boolean }>(),
  { disabled: false },
)
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()
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

    &:checked + .track {
      background: $text-primary;

      .thumb {
        background: $bg;
        transform: translateX(14px);
      }
    }
  }
}

.track {
  position: relative;
  width: 32px;
  height: 18px;
  background: $border-strong;
  border-radius: 9px;
  transition: background $transition;
  flex-shrink: 0;
}

.thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  background: $text-primary;
  border-radius: 50%;
  transition: transform $transition, background $transition;
}

.toggle-label {
  font-size: 13px;
  font-weight: 500;
  color: $text-primary;
}
</style>
