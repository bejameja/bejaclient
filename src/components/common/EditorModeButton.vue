<template>
  <button
    class="emb-btn"
    :class="{ 'emb-btn--active': enabled }"
    @click="setEnabled(!enabled)"
  >
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" />
    </svg>
    <span>{{ enabled ? 'Exit Editor Mode' : 'Enter Editor Mode' }}</span>
  </button>
</template>

<script setup lang="ts">
import { useEditorMode } from '../../composables/useElementStyle'

const { enabled, setEnabled } = useEditorMode()
</script>

<style lang="scss" scoped>
.emb-btn {
  position: fixed;
  bottom: 18px;
  right: 18px;
  z-index: 400;
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 9px 14px;
  background: rgba(20, 20, 20, 0.9);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.8);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
  transition: background 140ms ease, border-color 140ms ease, transform 140ms ease;

  &:hover { transform: translateY(-1px); border-color: rgba(120, 170, 255, 0.5); }

  &--active {
    background: rgba(49, 89, 163, 0.95);
    border-color: #78aaff;
    color: #fff;
    animation: emb-pulse 2s ease-in-out infinite;
  }
}

@keyframes emb-pulse {
  0%, 100% { box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45), 0 0 0 0 rgba(120, 170, 255, 0.4); }
  50%      { box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45), 0 0 0 6px rgba(120, 170, 255, 0); }
}
</style>
