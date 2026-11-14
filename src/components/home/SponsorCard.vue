<template>
  <button class="sponsor" type="button" title="Hosting Haven" @click="open">
    <img class="sponsor-banner" :src="banner" alt="Powered by Hosting Haven" draggable="false" />
  </button>
</template>

<script setup lang="ts">
import banner from '../../assets/sponsors/hosting-haven-banner.png'

const URL = 'https://hostinghaven.us/'

function open(): void {
  void window.api.system.openExternal(URL)
}
</script>

<style lang="scss" scoped>
// Hosting Haven's own banner, shown as supplied.
.sponsor {
  display: block;
  width: 100%;
  padding: 0;
  border: none;
  border-radius: 12px;
  background: none;
  cursor: pointer;
  overflow: hidden;
  transition: opacity 140ms ease, transform 140ms ease;

  &:hover { opacity: 0.88; }
  &:active { transform: scale(0.995); }
}

.sponsor-banner {
  display: block;
  width: 100%;
  height: auto;
}
</style>
