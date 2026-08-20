<template>
  <Teleport to="body">
    <div class="toast-stack" aria-live="polite">
      <TransitionGroup name="toast-pop">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="toast-card"
          :class="`variant-${toast.variant}`"
        >
          <div class="toast-icon-chip">
            <Icon :name="iconFor(toast.variant)" :size="13" class="toast-icon" />
          </div>

          <div class="toast-info">
            <div class="toast-title">{{ toast.title }}</div>
            <div v-if="toast.body" class="toast-body">{{ toast.body }}</div>
          </div>

          <button
            v-if="toast.action"
            class="toast-action"
            @click="runAction(toast)"
          >{{ toast.action.label }}</button>

          <button class="toast-close" @click="dismissToast(toast.id)">
            <Icon name="close" :size="12" />
          </button>

          <div class="toast-bar" :style="{ animationDuration: toast.duration + 'ms' }" />
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useToasts, type Toast, type ToastVariant } from '../../composables/useToasts'
import Icon from './Icon.vue'
import type { IconName } from './icons'

const { toasts, dismissToast } = useToasts()

function iconFor(variant: ToastVariant): IconName {
  switch (variant) {
    case 'success': return 'success'
    case 'warning': return 'warning'
    case 'error':   return 'error'
    default:        return 'info'
  }
}

function runAction(toast: Toast) {
  toast.action?.onClick()
  dismissToast(toast.id)
}
</script>

<style lang="scss" scoped>
.toast-stack {
  position: fixed;
  top: 24px;
  right: 24px;
  z-index: 9000;
  display: flex;
  flex-direction: column;
  gap: 10px;
  pointer-events: none;
}

.toast-card {
  pointer-events: all;
  display: flex;
  align-items: flex-start;
  gap: 9px;
  width: 264px;
  padding: 10px 11px 12px;
  background: #000;
  border: 1px solid $border-strong;
  border-radius: 4px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  position: relative;
  overflow: hidden;
}

.toast-icon-chip {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.06);
}

.toast-icon { color: $text-secondary; }

.variant-success .toast-icon-chip { background: rgba(48, 209, 88, 0.14); }
.variant-warning .toast-icon-chip { background: rgba(232, 160, 48, 0.14); }
.variant-error   .toast-icon-chip { background: rgba(224, 80, 80, 0.14); }
.variant-info    .toast-icon-chip { background: rgba(255, 255, 255, 0.07); }

.variant-success .toast-icon { color: $success; }
.variant-warning .toast-icon { color: $warning; }
.variant-error   .toast-icon { color: $error; }
.variant-info    .toast-icon { color: $accent; }

.toast-info {
  flex: 1;
  min-width: 0;
  padding-top: 1px;
}

.toast-title {
  font-size: 12px;
  font-weight: 600;
  color: $text-primary;
  line-height: 1.35;
}

.toast-body {
  margin-top: 2px;
  font-size: 11px;
  color: $text-secondary;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

.toast-action {
  flex-shrink: 0;
  padding: 3px 8px;
  border-radius: 4px;
  border: 1px solid $border-strong;
  background: transparent;
  color: $text-primary;
  font-size: 10.5px;
  font-weight: 600;
  cursor: pointer;
  transition: background $transition;

  &:hover { background: rgba(255, 255, 255, 0.08); }
}

.toast-close {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  border-radius: 4px;
  border: none;
  background: transparent;
  color: $text-muted;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color $transition, background $transition;

  &:hover { color: $text-primary; background: rgba(255, 255, 255, 0.08); }
}

.toast-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 2px;
  width: auto;
  border-radius: 4px;
  background: $accent;
  transform-origin: left;
  animation: toast-shrink linear forwards;
  opacity: 0.55;
}

@keyframes toast-shrink {
  from { transform: scaleX(1); }
  to   { transform: scaleX(0); }
}
</style>
