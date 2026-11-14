<template>
  <canvas ref="canvasRef" class="cape-portrait" :width="10 * scale" :height="16 * scale" />
</template>

<script setup lang="ts">
// A crisp flat view of a cape as it hangs on a player's back, cut out of the cape texture (the outer side of the cape is
// the 10x16 area at (1,1) of the usual 64x32 layout; bigger textures are the same layout at a multiple).
import { ref, watch, onMounted } from 'vue'

const props = withDefaults(defineProps<{
  capeUrl: string | null
  scale?: number
}>(), { scale: 8 })

const canvasRef = ref<HTMLCanvasElement | null>(null)
let current: HTMLImageElement | null = null

function draw(img: HTMLImageElement | null): void {
  const cv = canvasRef.value
  const ctx = cv?.getContext('2d')
  if (!cv || !ctx) return
  ctx.clearRect(0, 0, cv.width, cv.height)
  if (!img) return
  ctx.imageSmoothingEnabled = false
  const f = img.width / 64
  ctx.drawImage(img, 1 * f, 1 * f, 10 * f, 16 * f, 0, 0, cv.width, cv.height)

  // a little form: light along the top edge, shade along the bottom and the right side
  const k = props.scale
  ctx.fillStyle = 'rgba(255, 255, 255, 0.16)'
  ctx.fillRect(0, 0, cv.width, k)
  ctx.fillStyle = 'rgba(0, 0, 0, 0.18)'
  ctx.fillRect(0, cv.height - k, cv.width, k)
  ctx.fillRect(cv.width - k, 0, k, cv.height)
}

function load(): void {
  const url = props.capeUrl
  if (!url) { current = null; draw(null); return }
  const img = new Image()
  img.onload = () => { current = img; draw(img) }
  img.onerror = () => { current = null; draw(null) }
  img.src = url
}

onMounted(load)
watch(() => props.capeUrl, load)
watch(() => props.scale, () => draw(current))
</script>

<style scoped>
.cape-portrait {
  display: block;
  image-rendering: pixelated;
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
</style>
