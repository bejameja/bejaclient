<template>
  <Transition name="pad-hints">
    <div v-if="padActive" class="pad-hints">
      <span class="hint"><i>A</i>Select</span>
      <span class="hint"><i>B</i>Back</span>
      <span class="hint"><i>LB</i><i>RB</i>Tabs</span>
      <span class="hint"><i>Y</i>Search</span>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { padActive } from '../../composables/useGamepad'
</script>

<style lang="scss" scoped>
.pad-hints {
  position: fixed;
  left: 20px;
  bottom: 18px;
  z-index: 9500;
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 14px;
  border-radius: 12px;
  background: #13141A;
  pointer-events: none;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  letter-spacing: 0.02em;
}

.hint {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

i {
  font-style: normal;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 20px;
  padding: 0 5px;
  border-radius: 10px;
  background: #2a2a30;
  color: #fff;
  font-size: 11px;
}

.pad-hints-enter-active, .pad-hints-leave-active { transition: opacity 160ms ease, transform 160ms ease; }
.pad-hints-enter-from, .pad-hints-leave-to { opacity: 0; transform: translateY(6px); }
</style>
