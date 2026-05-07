<template>
  <div ref="containerRef" class="hero-viewer">
    <canvas ref="canvasRef" class="viewer-canvas" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { SkinViewer, WalkingAnimation, IdleAnimation, createOrbitControls } from 'skinview3d'
import type { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'

const props = withDefaults(defineProps<{
  skinUrl?:   string | null
  capeUrl?:   string | null
  // 'default' = classic/Steve arms, 'slim' = Alex arms
  model?:     'default' | 'slim' | 'auto-detect'
  animation?: 'walk' | 'idle'
}>(), {
  model:     'auto-detect',
  animation: 'walk',
})

const containerRef = ref<HTMLDivElement | null>(null)
const canvasRef    = ref<HTMLCanvasElement | null>(null)
let viewer:   SkinViewer   | null = null
let controls: OrbitControls | null = null
let ro:       ResizeObserver | null = null

onMounted(() => {
  const canvas    = canvasRef.value!
  const container = containerRef.value!

  viewer = new SkinViewer({
    canvas,
    width:  container.clientWidth  || 300,
    height: container.clientHeight || 500,
    alpha:  true,
    zoom:   0.9,
    fov:    40,
  })

  // Walking = LabyMod-style live character; idle = subtle bob only
  viewer.animations.add(props.animation === 'walk' ? WalkingAnimation : IdleAnimation)
  viewer.animations.speed = 0.7

  // Softer lighting — tuned for dark launcher backgrounds
  viewer.globalLight.intensity = 0.8
  viewer.cameraLight.intensity = 0.6

  // Mouse drag-to-rotate via skinview3d's OrbitControls helper
  controls = createOrbitControls(viewer)
  controls.enableZoom  = false
  controls.enablePan   = false
  controls.rotateSpeed = 0.6

  if (props.skinUrl) viewer.loadSkin(props.skinUrl, props.model)
  if (props.capeUrl) viewer.loadCape(props.capeUrl)

  ro = new ResizeObserver(entries => {
    const { width, height } = entries[0].contentRect
    if (width > 0 && height > 0) viewer?.setSize(width, height)
  })
  ro.observe(container)
})

// Reload when URL or model type changes (e.g. account switch)
watch([() => props.skinUrl, () => props.model], ([url, model]) => {
  if (!viewer) return
  if (url) viewer.loadSkin(url, model ?? 'auto-detect')
  else     viewer.resetSkin()
})

watch(() => props.capeUrl, url => {
  if (!viewer) return
  if (url) viewer.loadCape(url)
  else     viewer.resetCape()
})

onUnmounted(() => {
  controls?.dispose()
  ro?.disconnect()
  viewer?.dispose()
  viewer = null
})

// Speed-burst emote triggered by pressing B on the home page
function triggerEmote() {
  if (!viewer) return
  viewer.animations.speed = 2.8
  setTimeout(() => { if (viewer) viewer.animations.speed = 0.7 }, 2200)
}

defineExpose({ triggerEmote })
</script>

<style scoped>
.hero-viewer {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
  user-select: none;
}

.viewer-canvas {
  display: block;
  width: 100%;
  height: 100%;
  cursor: grab;
}

.viewer-canvas:active {
  cursor: grabbing;
}
</style>
