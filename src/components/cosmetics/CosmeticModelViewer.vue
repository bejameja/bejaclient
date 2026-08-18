<template>
  <div ref="containerRef" class="cmv" :class="{ 'cmv--enchanted': info?.animated }">
    <canvas v-if="isVisible" v-show="modelLoaded" ref="canvasRef" class="cmv__canvas" />

    <div v-if="!modelLoaded" class="cmv__placeholder" :style="placeholderStyle">
      <img v-if="rarity && RARITY_ICONS[rarity]" class="cmv__icon" :src="RARITY_ICONS[rarity]" :alt="info?.label" />
      <svg v-else class="cmv__gem" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <polygon points="12,3 22,9 18,21 6,21 2,9" :fill="info?.color ?? '#555555'" opacity="0.85" />
        <polygon points="12,3 22,9 12,7" :fill="info?.color ?? '#555555'" opacity="0.5" />
        <polygon points="2,9 12,7 6,21" :fill="info?.color ?? '#555555'" opacity="0.6" />
      </svg>
      <span v-if="showLabel && rarity" class="cmv__label" :style="{ color: info?.color }">
        {{ info?.label }}
      </span>
    </div>

    <div v-if="info?.animated" class="cmv__enchant" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, onActivated, onDeactivated, nextTick } from 'vue'
import type { Rarity } from '../../types/cosmetics'
import { RARITIES } from '../../types/cosmetics'
import { RARITY_ICONS } from '../../assets/rarities/index'
import { useLazyVisible } from '../../composables/useLazyVisible'

/* eslint-disable @typescript-eslint/no-explicit-any */
let _THREE: any = null
let _GLTFLoader: any = null

async function getThree() {
  if (!_THREE) {
    _THREE      = await import('three' as any)
    const m     = await import('three/examples/jsm/loaders/GLTFLoader.js' as any)
    _GLTFLoader = m.GLTFLoader
  }
  return { THREE: _THREE, GLTFLoader: _GLTFLoader }
}

const props = withDefaults(defineProps<{
  modelUrl?: string
  rarity?: Rarity
  autoRotate?: boolean
  showLabel?: boolean
}>(), {
  autoRotate: true,
  showLabel: false,
})

const containerRef = ref<HTMLDivElement | null>(null)
const canvasRef    = ref<HTMLCanvasElement | null>(null)
const modelLoaded  = ref(false)

const info = computed(() => props.rarity ? RARITIES[props.rarity] : null)

const placeholderStyle = computed(() => ({
  background: info.value?.bg ?? 'rgba(80,80,80,0.12)',
}))

// ── Three.js state ────────────────────────────────────────────────────────────
let scene:    any = null
let camera:   any = null
let renderer: any = null
let model:    any = null
let frameId = 0
let looping = false

// The Cosmetics tab renders one of these per owned item — an unbounded grid.
// Creating a real WebGLRenderer per card unconditionally silently exceeds
// Chromium's per-page WebGL context budget once a collection gets large
// enough, force-losing the *oldest* live context elsewhere (e.g. the Hub's
// HeroSkinViewer, which then renders as a blank/white box with no recovery).
// Only building the renderer for on/near-screen cards keeps the live count
// bounded to what's actually visible, however big the collection is. The
// canvas itself is v-if'd (not just the renderer disposed) so the old
// context is actually eligible for GC once a card scrolls away — disposing
// a THREE.WebGLRenderer frees its GPU resources but not the context slot on
// a canvas that stays in the DOM, and a browser won't hand a canvas a second
// live context anyway.
const isVisible = useLazyVisible(containerRef)



// ── Scene init ────────────────────────────────────────────────────────────────
async function initScene(w: number, h: number) {
  if (!canvasRef.value) return
  const { THREE } = await getThree()

  scene  = new THREE.Scene()
  camera = new THREE.PerspectiveCamera(60, w / h, 0.1, 100)
  camera.position.set(0, 0, 4)

  renderer = new THREE.WebGLRenderer({ canvas: canvasRef.value, alpha: true, antialias: true })
  renderer.setSize(w, h)
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
  renderer.outputColorSpace = THREE.SRGBColorSpace

  const ambient = new THREE.AmbientLight(0xffffff, 0.7)
  const key     = new THREE.DirectionalLight(0xffffff, 1.3)
  key.position.set(2, 3, 4)
  scene.add(ambient, key)

  if (props.rarity && RARITIES[props.rarity]) {
    const col = parseInt(RARITIES[props.rarity].color.replace('#', ''), 16)
    const rim = new THREE.PointLight(col, 1.4, 14)
    rim.position.set(-2, 1, -3)
    scene.add(rim)
  }

  startLoop()
}

// ── Load cosmetic model ───────────────────────────────────────────────────────
async function loadModel(url: string) {
  if (!scene) return
  if (model) { scene.remove(model); model = null }
  modelLoaded.value = false

  try {
    const { THREE, GLTFLoader } = await getThree()
    const loader = new GLTFLoader()
    const gltf: any = await new Promise((res, rej) => loader.load(url, res, undefined, rej))
    const obj = gltf.scene

    // Compute bounding box
    obj.updateMatrixWorld(true)
    const box = new THREE.Box3().setFromObject(obj)

    if (!box.isEmpty()) {
      const size   = box.getSize(new THREE.Vector3())
      const scale  = 4.0 / (Math.max(size.x, size.y, size.z) || 1)
      obj.scale.setScalar(scale)
      const center = box.getCenter(new THREE.Vector3())
      obj.position.sub(center.multiplyScalar(scale))
    }

    scene.add(obj)
    model = obj
    modelLoaded.value = true
  } catch {
    modelLoaded.value = false
  }
}

// ── Render loop ───────────────────────────────────────────────────────────────
function startLoop() {
  if (looping) return
  looping = true
  function tick() {
    frameId = requestAnimationFrame(tick)
    if (model && props.autoRotate) model.rotation.y += 0.008
    if (renderer && scene && camera) renderer.render(scene, camera)
  }
  tick()
}

function stopLoop() {
  looping = false
  cancelAnimationFrame(frameId)
}

// ── Resize ────────────────────────────────────────────────────────────────────
function handleResize() {
  if (!containerRef.value || !renderer || !camera) return
  const w = containerRef.value.clientWidth
  const h = containerRef.value.clientHeight
  renderer.setSize(w, h)
  camera.aspect = w / h
  camera.updateProjectionMatrix()
}

let ro: ResizeObserver | null = null

async function setupIfNeeded() {
  if (renderer || !containerRef.value) return
  const w = containerRef.value.clientWidth  || 200
  const h = containerRef.value.clientHeight || 200
  await initScene(w, h)
  if (props.modelUrl) loadModel(props.modelUrl)
}

function teardownScene() {
  stopLoop()
  if (model) {
    scene?.remove(model)
    model.traverse((obj: any) => {
      obj.geometry?.dispose?.()
      if (Array.isArray(obj.material)) obj.material.forEach((m: any) => m.dispose?.())
      else obj.material?.dispose?.()
    })
  }
  renderer?.dispose()
  renderer = null
  scene    = null
  camera   = null
  model    = null
  modelLoaded.value = false
}

onMounted(() => {
  if (!containerRef.value) return
  ro = new ResizeObserver(handleResize)
  ro.observe(containerRef.value)
})

// Only build the actual THREE scene/renderer while the card is on/near
// screen — torn down again once it scrolls away. See the isVisible comment
// above for why an unconditional per-card renderer is the real bug.
watch(isVisible, async visible => {
  if (visible) {
    await nextTick()
    await setupIfNeeded()
  } else {
    teardownScene()
  }
}, { immediate: true })

watch(() => props.modelUrl, url => {
  if (!renderer) return // no live scene right now; setupIfNeeded() picks up the current modelUrl once visible
  if (!url) { modelLoaded.value = false; return }
  loadModel(url)
})

// Paused via KeepAlive tab switches — a raw three.js render loop left
// running behind an inactive tab is a straight GPU/CPU leak per tab visited.
onActivated(() => { if (scene && camera && renderer) startLoop() })
onDeactivated(stopLoop)

onUnmounted(() => {
  ro?.disconnect()
  teardownScene()
})
</script>

<style lang="scss" scoped>
.cmv {
  position: relative;
  overflow: hidden;
  width: 100%;
  height: 100%;
}

.cmv__canvas {
  width: 100%;
  height: 100%;
  display: block;
}

.cmv__placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.cmv__icon {
  width: 44%;
  height: 44%;
  min-width: 28px;
  min-height: 28px;
  image-rendering: pixelated;
  object-fit: contain;
  filter: drop-shadow(0 2px 8px rgba(0,0,0,0.6));
}

.cmv__gem {
  width: 38%;
  height: 38%;
  min-width: 24px;
  min-height: 24px;
  filter: drop-shadow(0 0 6px currentColor);
}

.cmv__label {
  font-size: 9px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  opacity: 0.9;
}

.cmv__enchant {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: linear-gradient(
    135deg,
    transparent 0%,
    rgba(255, 136, 255, 0.07) 25%,
    transparent 50%,
    rgba(136, 136, 255, 0.07) 75%,
    transparent 100%
  );
  background-size: 200% 200%;
  animation: enchant 3s linear infinite;
}

@keyframes enchant {
  0%   { background-position: 0% 0%; }
  100% { background-position: 200% 200%; }
}
</style>
