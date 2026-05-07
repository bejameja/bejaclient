<template>
  <div class="avatar-wrap" :style="{ width: `${size}px`, height: `${size}px` }">
    <img
      v-if="src && !errored"
      :src="src"
      :width="size"
      :height="size"
      class="avatar-img"
      @error="errored = true"
    />
    <div v-else class="avatar-letter" :style="{ fontSize: `${Math.round(size * 0.42)}px` }">
      {{ (username ?? 'P')[0].toUpperCase() }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const props = withDefaults(
  defineProps<{
    uuid?: string | null
    skinUrl?: string | null
    username?: string
    size?: number
  }>(),
  { username: 'Player', size: 32 },
)

const errored = ref(false)
const src = computed(() =>
  props.uuid
    ? `https://crafatar.com/avatars/${props.uuid}?size=${props.size * 2}&overlay=true`
    : '',
)
</script>

<style lang="scss" scoped>
.avatar-wrap {
  border-radius: 4px;
  overflow: hidden;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}
.avatar-img {
  display: block;
  width: 100%;
  height: 100%;
  image-rendering: pixelated;
}
.avatar-letter {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: $surface-elevated;
  color: $text-primary;
  font-weight: 800;
}
</style>
