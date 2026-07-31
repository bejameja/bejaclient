<template>
  <div ref="containerRef" class="static-viewer">
    <canvas v-if="isVisible" ref="canvasRef" class="viewer-canvas" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, onActivated, onDeactivated, watch, nextTick } from 'vue'
import { SkinViewer, createOrbitControls } from 'skinview3d'
import { useLazyVisible } from '../../composables/useLazyVisible'

const props = withDefaults(defineProps<{
  skinUrl?:          string | null
  capeUrl?:          string | null
  model?:            'default' | 'slim' | 'auto-detect'
  zoom?:             number
  initialRotationY?: number
}>(), {
  model:            'auto-detect',
  zoom:             0.75,
  initialRotationY: 0,
})

const containerRef = ref<HTMLDivElement | null>(null)
const canvasRef    = ref<HTMLCanvasElement | null>(null)
let viewer: SkinViewer | null = null
let ro:     ResizeObserver | null = null
let deactivated = false
let freezeRaf: number | null = null

// Card grids (Skins tab, etc.) can hold dozens of entries — creating a real
// WebGL context per card unconditionally silently blows Chromium's per-page
// context budget once enough cards exist, force-losing the *oldest* live
// context (often an unrelated viewer elsewhere, e.g. the Hub's HeroSkinViewer)
// with no recovery. Only running the viewer for on/near-screen cards keeps
// the live-context count bounded to what's actually visible. The canvas
// itself is v-if'd (not just v-show/dispose()'d) so the old context is
// actually eligible for GC — a WebGLRenderer.dispose() call frees its GPU
// resources but does NOT release the context slot on a canvas that stays
// in the DOM, and a browser won't hand out a second live context to the
// same canvas element anyway.
const isVisible = useLazyVisible(containerRef)

// The pose never changes after the first frame — there's no reason to keep
// the WebGL render loop running 60fps forever for a frame that never
// changes. Render a couple frames to let the pose settle, then pause.
function freezeSoon() {
  if (freezeRaf !== null) cancelAnimationFrame(freezeRaf)
  if (!viewer) return
  viewer.renderPaused = false
  freezeRaf = requestAnimationFrame(() => {
    freezeRaf = requestAnimationFrame(() => {
      freezeRaf = null
      if (viewer && !deactivated) viewer.renderPaused = true
    })
  })
}

// Apply the custom-idle rest pose once, then freeze
function applyIdlePose() {
  if (!viewer) return
  viewer.animations.add((player: any) => {
    player.skin.head.rotation.x        = -0.05
    player.skin.rightArm.rotation.z    = -0.0873
    player.skin.leftArm.rotation.z     =  0.0873
    player.skin.rightLeg.rotation.z    = -0.0873
    player.skin.leftLeg.rotation.z     =  0.0873
    viewer!.animations.speed = 0
  })
  viewer.animations.speed = 1
  freezeSoon()
}

let controls: ReturnType<typeof createOrbitControls> | null = null

function createViewer() {
  if (viewer || !canvasRef.value || !containerRef.value) return
  const canvas    = canvasRef.value
  const container = containerRef.value

  viewer = new SkinViewer({
    canvas,
    width:  container.clientWidth  || 200,
    height: container.clientHeight || 300,
    alpha:  true,
    zoom:   props.zoom,
    fov:    40,
  })

  viewer.globalLight.intensity = 0.62
  viewer.cameraLight.intensity = 0.5

  controls = createOrbitControls(viewer)
  controls.enableZoom   = false
  controls.enablePan    = false
  controls.enableRotate = false

  if (props.initialRotationY) {
    viewer.playerWrapper.rotation.y = props.initialRotationY
  }

  if (props.skinUrl) viewer.loadSkin(props.skinUrl, props.model)
  if (props.capeUrl) viewer.loadCape(props.capeUrl)

  applyIdlePose()
}

function destroyViewer() {
  if (freezeRaf !== null) { cancelAnimationFrame(freezeRaf); freezeRaf = null }
  controls?.dispose()
  controls = null
  viewer?.dispose()
  viewer = null
}

onMounted(() => {
  const container = containerRef.value!
  ro = new ResizeObserver(entries => {
    const { width, height } = entries[0].contentRect
    if (width > 0 && height > 0) {
      viewer?.setSize(width, height)
      freezeSoon()
    }
  })
  ro.observe(container)
})

// Card grids can hold far more entries than fit on screen at once — only the
// visible ones (per useLazyVisible above) get a real viewer/WebGL context;
// scrolling a card off-screen fully tears it down again (the v-if above drops
// the old canvas so a fresh one — and a fresh context — is made next time).
watch(isVisible, async visible => {
  if (visible) {
    await nextTick()
    createViewer()
    freezeSoon()
  } else {
    destroyViewer()
  }
}, { immediate: true })

// Paused via KeepAlive tab switches — no point rendering a frozen frame
// for a page the user isn't looking at.
onActivated(() => { deactivated = false; freezeSoon() })
onDeactivated(() => {
  deactivated = true
  if (freezeRaf !== null) { cancelAnimationFrame(freezeRaf); freezeRaf = null }
  if (viewer) viewer.renderPaused = true
})

watch([() => props.skinUrl, () => props.model], ([url, model]) => {
  if (!viewer) return
  if (url) viewer.loadSkin(url, model ?? 'auto-detect')
  else     viewer.resetSkin()
  applyIdlePose()
})

watch(() => props.capeUrl, url => {
  if (!viewer) return
  if (url) viewer.loadCape(url)
  else     viewer.resetCape()
  freezeSoon()
})

onUnmounted(() => {
  ro?.disconnect()
  destroyViewer()
})
</script>

<style scoped>
.static-viewer {
  width: 100%;
  height: 100%;
  overflow: hidden;
  user-select: none;
  pointer-events: none;
}
.viewer-canvas {
  display: block;
  width: 100%;
  height: 100%;
}
</style>
