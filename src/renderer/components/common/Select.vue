<template>
  <div class="bc-select-wrapper">
    <label v-if="label" class="bc-label">{{ label }}</label>
    <div class="bc-select-inner" :class="{ open: isOpen, disabled }">
      <button class="bc-select-trigger" :disabled="disabled" @click="toggle">
        <span class="selected-text">{{ selectedLabel }}</span>
        <svg class="chevron" width="10" height="6" viewBox="0 0 10 6">
          <path d="M1 1L5 5L9 1" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" />
        </svg>
      </button>
      <Transition name="dropdown">
        <div v-if="isOpen" class="bc-dropdown">
          <button
            v-for="opt in options"
            :key="String(opt.value)"
            class="bc-option"
            :class="{ active: opt.value === modelValue }"
            @click="select(opt.value)"
          >
            {{ opt.label }}
          </button>
        </div>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

interface Option {
  label: string
  value: string | number
}

const props = withDefaults(
  defineProps<{
    modelValue: string | number | null
    options: Option[]
    label?: string
    placeholder?: string
    disabled?: boolean
  }>(),
  {
    placeholder: 'Select...',
    disabled: false,
  },
)

const emit = defineEmits<{ 'update:modelValue': [value: string | number] }>()
const isOpen = ref(false)

const selectedLabel = computed(
  () => props.options.find(o => o.value === props.modelValue)?.label ?? props.placeholder,
)

function toggle() {
  if (!props.disabled) isOpen.value = !isOpen.value
}

function select(value: string | number) {
  emit('update:modelValue', value)
  isOpen.value = false
}
</script>

<style lang="scss" scoped>
.bc-select-wrapper {
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

.bc-select-inner {
  position: relative;

  &.disabled .bc-select-trigger {
    opacity: 0.4;
    cursor: not-allowed;
  }
}

.bc-select-trigger {
  width: 100%;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 $sp-3;
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius;
  font-size: 13px;
  font-weight: 500;
  color: $text-primary;
  cursor: pointer;
  transition: border-color $transition;

  &:hover:not(:disabled) {
    border-color: $border-strong;
  }
}

.selected-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chevron {
  color: $muted;
  flex-shrink: 0;
  transition: transform $transition;

  .open & {
    transform: rotate(180deg);
  }
}

.bc-dropdown {
  position: absolute;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  background: $surface-elevated;
  border: 1px solid $border-strong;
  border-radius: $radius;
  z-index: 500;
  max-height: 200px;
  overflow-y: auto;
  padding: 2px;
}

.bc-option {
  width: 100%;
  padding: $sp-2 $sp-3;
  text-align: left;
  font-size: 13px;
  font-weight: 500;
  color: $text-secondary;
  cursor: pointer;
  border-radius: $radius-sm;
  transition: background $transition, color $transition;

  &:hover {
    background: $border;
    color: $text-primary;
  }

  &.active {
    color: $text-primary;
    background: $border;
  }
}

.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity $transition-fast, transform $transition-fast;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
