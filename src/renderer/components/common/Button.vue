<template>
  <button
    class="bc-btn"
    :class="[`variant-${variant}`, `size-${size}`, { loading, disabled: disabled || loading, full }]"
    :disabled="disabled || loading"
    v-bind="$attrs"
  >
    <span v-if="loading" class="btn-spinner" />
    <slot v-if="!loading" name="icon" />
    <span v-if="$slots.default" class="btn-label"><slot /></span>
  </button>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
    size?: 'sm' | 'md' | 'lg'
    loading?: boolean
    disabled?: boolean
    full?: boolean
  }>(),
  {
    variant: 'primary',
    size: 'md',
    loading: false,
    disabled: false,
    full: false,
  },
)
</script>

<style lang="scss" scoped>
.bc-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: $sp-2;
  border: none;
  border-radius: $radius;
  font-family: $font-family;
  font-weight: 600;
  cursor: pointer;
  transition: background $transition, color $transition, border-color $transition, opacity $transition;
  white-space: nowrap;
  flex-shrink: 0;

  &.disabled {
    opacity: 0.4;
    cursor: not-allowed;
    pointer-events: none;
  }

  &.full {
    width: 100%;
  }

  // Sizes
  &.size-sm {
    height: 28px;
    padding: 0 $sp-3;
    font-size: 11px;
  }
  &.size-md {
    height: 34px;
    padding: 0 $sp-4;
    font-size: 13px;
  }
  &.size-lg {
    height: 42px;
    padding: 0 $sp-6;
    font-size: 14px;
    letter-spacing: 0.02em;
  }

  // Variants — white bg + black text for primary; neutral for the rest
  &.variant-primary {
    background: $text-primary;
    color: $bg;
    &:hover:not(.disabled) { background: $text-secondary; }
    &:active:not(.disabled) { background: $border-strong; }
  }

  &.variant-secondary {
    background: $surface-elevated;
    border: 1px solid $border;
    color: $text-primary;
    &:hover:not(.disabled) {
      background: $border;
      border-color: $border-strong;
    }
  }

  &.variant-ghost {
    background: transparent;
    color: $text-secondary;
    &:hover:not(.disabled) {
      background: $surface-elevated;
      color: $text-primary;
    }
  }

  &.variant-danger {
    background: $surface-elevated;
    border: 1px solid $border;
    color: $text-secondary;
    &:hover:not(.disabled) {
      background: $border;
      color: $text-primary;
    }
  }
}

.btn-label {
  line-height: 1;
}

.btn-spinner {
  width: 12px;
  height: 12px;
  border: 1.5px solid $border-strong;
  border-top-color: $text-primary;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
