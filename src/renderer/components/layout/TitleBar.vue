<template>
  <div class="titlebar" :class="{ maximized }">
    <div class="titlebar-brand">
      <img class="brand-logo" src="/src/assets/logo.png" alt="BC" />
      <span class="brand-name">BejaClient</span>
      <span class="brand-version">v{{ version }}</span>
    </div>

    <div class="titlebar-drag" />

    <div class="titlebar-controls">
      <button class="ctrl-btn" title="Minimize" @click="minimize">
        <svg width="10" height="1" viewBox="0 0 10 1"><rect width="10" height="1" fill="currentColor"/></svg>
      </button>
      <button class="ctrl-btn" title="Maximize" @click="toggleMaximize">
        <svg v-if="!maximized" width="9" height="9" viewBox="0 0 9 9">
          <rect x="0.5" y="0.5" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1"/>
        </svg>
        <svg v-else width="10" height="10" viewBox="0 0 10 10">
          <rect x="2" y="0" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1"/>
          <rect x="0" y="2" width="8" height="8" fill="$surface" stroke="currentColor" stroke-width="1"/>
        </svg>
      </button>
      <button class="ctrl-btn close-btn" title="Close" @click="close">
        <svg width="10" height="10" viewBox="0 0 10 10">
          <line x1="0.5" y1="0.5" x2="9.5" y2="9.5" stroke="currentColor" stroke-width="1.2"/>
          <line x1="9.5" y1="0.5" x2="0.5" y2="9.5" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

const maximized = ref(false)
const version = '1.0'

onMounted(async () => {
  maximized.value = await window.api.window.isMaximized()
  window.api.window.onMaximized((v: boolean) => { maximized.value = v })
})

function minimize()       { window.api.window.minimize() }
function toggleMaximize() { window.api.window.maximize() }
function close()          { window.api.window.close() }
</script>

<style lang="scss" scoped>
.titlebar {
  display: flex;
  align-items: center;
  height: $titlebar-height;
  background: $surface;
  border-bottom: 1px solid $border;
  -webkit-app-region: drag;
  flex-shrink: 0;
  z-index: 200;
}

// ── Brand ─────────────────────────────────────────────────────────────────────
.titlebar-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 14px 0 16px;
  -webkit-app-region: no-drag;
  pointer-events: none;
  width: $sidebar-width;
  flex-shrink: 0;
  border-right: 1px solid $border;
  height: 100%;
}

.brand-logo {
  width: 18px;
  height: 18px;
  object-fit: contain;
  flex-shrink: 0;
}

.brand-name {
  font-family: $font-display;
  font-size: 13px;
  font-weight: 700;
  color: $text-primary;
  letter-spacing: 0.04em;
}

.brand-version {
  font-size: 10px;
  font-weight: 500;
  color: $muted;
  margin-top: 1px;
}

// ── Drag ──────────────────────────────────────────────────────────────────────
.titlebar-drag {
  flex: 1;
  height: 100%;
  -webkit-app-region: drag;
}

// ── Controls ──────────────────────────────────────────────────────────────────
.titlebar-controls {
  display: flex;
  align-items: stretch;
  height: 100%;
  -webkit-app-region: no-drag;
}

.ctrl-btn {
  width: 46px;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  color: $muted;
  transition: background $transition, color $transition;

  &:hover {
    background: $surface-elevated;
    color: $text-primary;
  }

  &.close-btn:hover {
    background: $error;
    color: #fff;
  }
}
</style>
