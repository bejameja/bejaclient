<template>
  <div
    class="bc-skeleton"
    :class="`variant-${variant}`"
    :style="{
      width: toCss(width),
      height: toCss(height),
      borderRadius: radius,
    }"
  />
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'text' | 'block' | 'circle'
    width?: string | number
    height?: string | number
    radius?: string
  }>(),
  { variant: 'block' },
)

function toCss(v: string | number | undefined): string | undefined {
  if (v === undefined) return undefined
  return typeof v === 'number' ? `${v}px` : v
}
</script>

<style lang="scss" scoped>
.bc-skeleton {
  background: linear-gradient(100deg, $surface-elevated 30%, rgba(62, 184, 255, 0.09) 50%, $surface-elevated 70%);
  background-size: 200% 100%;
  animation: bc-shimmer 1.6s ease-in-out infinite;

  html.reduce-motion & {
    animation: none;
    background: $surface-elevated;
  }
}

.variant-text {
  width: 100%;
  height: 12px;
  border-radius: 4px;
}

.variant-block {
  width: 100%;
  height: 100%;
  border-radius: $radius;
}

.variant-circle {
  border-radius: 50%;
}

@keyframes bc-shimmer {
  0%   { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}
</style>
