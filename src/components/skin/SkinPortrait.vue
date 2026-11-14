<template>
  <canvas ref="canvasRef" class="skin-portrait" :width="16 * scale" :height="32 * scale" />
</template>

<script setup lang="ts">
// A crisp, flat front view of a Minecraft skin, drawn straight from the skin texture (no WebGL): head, body, arms and
// legs with their second layers. Cheap enough for a whole shelf of skins and every portrait is rendered at the same
// scale, so a grid of them lines up evenly.
import { ref, watch, onMounted } from 'vue'

const props = withDefaults(defineProps<{
  skinUrl: string | null
  model?: 'default' | 'slim' | 'auto-detect'
  scale?: number
}>(), { model: 'default', scale: 6 })

const canvasRef = ref<HTMLCanvasElement | null>(null)

function draw(img: HTMLImageElement | null): void {
  const cv = canvasRef.value
  if (!cv) return
  const ctx = cv.getContext('2d')
  if (!ctx) return
  const k = props.scale
  ctx.clearRect(0, 0, cv.width, cv.height)
  if (!img) return
  ctx.imageSmoothingEnabled = false

  const legacy = img.height <= 32                    // 64x32: one arm and one leg texture, mirrored
  const slim = props.model === 'slim'
  const armW = slim ? 3 : 4

  const part = (sx: number, sy: number, w: number, h: number, dx: number, dy: number, mirror = false) => {
    if (!mirror) {
      ctx.drawImage(img, sx, sy, w, h, dx * k, dy * k, w * k, h * k)
      return
    }
    ctx.save()
    ctx.translate((dx + w) * k, dy * k)
    ctx.scale(-1, 1)
    ctx.drawImage(img, sx, sy, w, h, 0, 0, w * k, h * k)
    ctx.restore()
  }

  // legs
  part(4, 20, 4, 12, 4, 20)
  if (legacy) part(4, 20, 4, 12, 8, 20, true)
  else part(20, 52, 4, 12, 8, 20)
  if (!legacy) { part(4, 36, 4, 12, 4, 20); part(4, 52, 4, 12, 8, 20) }

  // body
  part(20, 20, 8, 12, 4, 8)
  if (!legacy) part(20, 36, 8, 12, 4, 8)

  // arms: the skin's right arm is on the viewer's left
  part(44, 20, armW, 12, slim ? 1 : 0, 8)
  if (legacy) part(44, 20, armW, 12, 12, 8, true)
  else part(36, 52, armW, 12, 12, 8)
  if (!legacy) { part(44, 36, armW, 12, slim ? 1 : 0, 8); part(52, 52, armW, 12, 12, 8) }

  // head and hat layer
  part(8, 8, 8, 8, 4, 0)
  part(40, 8, 8, 8, 4, 0)
}

let current: HTMLImageElement | null = null

function load(): void {
  const url = props.skinUrl
  if (!url) { current = null; draw(null); return }
  const img = new Image()
  img.onload = () => { current = img; draw(img) }
  img.onerror = () => { current = null; draw(null) }
  img.src = url
}

onMounted(load)
watch(() => props.skinUrl, load)
watch(() => [props.model, props.scale], () => draw(current))
</script>

<style scoped>
.skin-portrait {
  display: block;
  image-rendering: pixelated;
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
</style>
