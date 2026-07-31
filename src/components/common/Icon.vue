<template>
  <img
    v-if="iconImages[name]"
    class="bc-icon bc-icon--img"
    :src="iconImages[name]"
    :width="size"
    :height="size"
    alt=""
    draggable="false"
  />
  <svg
    v-else
    class="bc-icon"
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    :stroke-width="strokeWidth"
    stroke-linecap="round"
    stroke-linejoin="round"
    v-html="icons[name]"
  />
</template>

<script setup lang="ts">
import { icons, iconImages, type IconName } from './icons'

withDefaults(
  defineProps<{ name: IconName; size?: number; strokeWidth?: number }>(),
  { size: 16, strokeWidth: 2 },
)
</script>

<style lang="scss" scoped>
.bc-icon {
  flex-shrink: 0;
  display: inline-block;
}

.bc-icon--img {
  object-fit: contain;
  image-rendering: pixelated;
}
</style>
