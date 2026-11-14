<template>
  <div ref="rootEl" class="campfire" :class="{ 'campfire--hover': hovering }" @contextmenu.prevent="onContextMenu" @pointermove="onPointerMove" @pointerleave="onPointerLeave" @click="onClick">
    <canvas ref="canvasEl" class="campfire-canvas" />

    <div class="campfire-overlay">
      <div
        v-for="m in members"
        :key="m.uuid"
        :ref="(el) => setTagEl(m.uuid, el as HTMLElement | null)"
        class="cf-tag"
        :class="{ 'cf-tag--self': m.uuid === localUuid }"
              >
        <svg v-if="m.isLeader" class="cf-crown" viewBox="0 0 9 7" shape-rendering="crispEdges" aria-hidden="true">
          <path d="M0 1h1v3h1V2h1v2h1V0h1v4h1V2h1v2h1V1h1v6H0z" fill="#3f3a00" transform="translate(0.4 0.4)" />
          <path d="M0 1h1v3h1V2h1v2h1V0h1v4h1V2h1v2h1V1h1v5H0z" fill="#ffd83a" />
          <path d="M0 5h9v1H0z" fill="#e0a410" />
        </svg>
        <span v-if="m.level" class="cf-level" :title="'Level ' + m.level">
          <svg class="cf-star" viewBox="0 0 7 7" shape-rendering="crispEdges" aria-hidden="true">
            <path d="M3 0h1v1h-1zM2 1h1v1h-1zM3 1h1v1h-1zM4 1h1v1h-1zM0 2h1v1h-1zM1 2h1v1h-1zM2 2h1v1h-1zM3 2h1v1h-1zM4 2h1v1h-1zM5 2h1v1h-1zM6 2h1v1h-1zM1 3h1v1h-1zM2 3h1v1h-1zM3 3h1v1h-1zM4 3h1v1h-1zM5 3h1v1h-1zM2 4h1v1h-1zM3 4h1v1h-1zM4 4h1v1h-1zM1 5h1v1h-1zM2 5h1v1h-1zM4 5h1v1h-1zM5 5h1v1h-1zM1 6h1v1h-1zM5 6h1v1h-1z" fill="#3f3a00" transform="translate(0.5 0.5)" />
            <path d="M3 0h1v1h-1zM2 1h1v1h-1zM3 1h1v1h-1zM4 1h1v1h-1zM0 2h1v1h-1zM1 2h1v1h-1zM2 2h1v1h-1zM3 2h1v1h-1zM4 2h1v1h-1zM5 2h1v1h-1zM6 2h1v1h-1zM1 3h1v1h-1zM2 3h1v1h-1zM3 3h1v1h-1zM4 3h1v1h-1zM5 3h1v1h-1zM2 4h1v1h-1zM3 4h1v1h-1zM4 4h1v1h-1zM1 5h1v1h-1zM2 5h1v1h-1zM4 5h1v1h-1zM5 5h1v1h-1zM1 6h1v1h-1zM5 6h1v1h-1z" fill="#ffd83a" />
          </svg>
          <b>{{ m.level }}</b>
        </span>
        <span>{{ m.username }}</span>
      </div>

      <div v-show="unreadCount > 0" :ref="(el) => setMailboxBadgeEl(el as HTMLElement | null)" class="cf-mail-badge">{{ unreadCount > 9 ? '9+' : unreadCount }}</div>

      <div v-if="bubble" :ref="(el) => setBubbleEl(el as HTMLElement | null)" class="cf-bubble">
        <span v-if="bubble.text">{{ bubble.text }}</span>
        <template v-else-if="bubble.give && bubble.get">
          <svg class="cf-icon" viewBox="0 0 8 8" shape-rendering="crispEdges">
            <rect v-for="(r, i) in iconRects(bubble.give)" :key="i" :x="r.x" :y="r.y" width="1" height="1" :fill="r.c" />
          </svg>
          <span class="cf-arrow">&rarr;</span>
          <svg class="cf-icon" viewBox="0 0 8 8" shape-rendering="crispEdges">
            <rect v-for="(r, i) in iconRects(bubble.get)" :key="i" :x="r.x" :y="r.y" width="1" height="1" :fill="r.c" />
          </svg>
        </template>
      </div>

      <button
        v-for="i in freeSeats"
        :key="'seat' + i"
        :ref="(el) => setSeatEl(i, el as HTMLElement | null)"
        class="cf-seat-invite"
        title="Invite a friend"
        @click="emit('invite')"
      >
        <svg class="cf-plus" viewBox="0 0 8 8" shape-rendering="crispEdges" aria-hidden="true">
          <path d="M1 4h6M4 1v6" stroke="#2a2a2a" stroke-width="1.5" transform="translate(0.5 0.5)" fill="none" />
          <path d="M1 3.5h6M3.5 1v6" stroke="#ffffff" stroke-width="1.5" fill="none" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, onActivated, onDeactivated, watch } from 'vue'
import {
  WebGLRenderer, Scene, PerspectiveCamera, Color, Fog, HemisphereLight, DirectionalLight, PointLight,
  Mesh, BoxGeometry, MeshBasicMaterial, Box3, Vector3, Raycaster, Object3D,
  PCFSoftShadowMap, NearestFilter, DoubleSide, sRGBEncoding, CanvasTexture,
} from 'three'
import { GLTFLoader, type GLTF } from 'three/examples/jsm/loaders/GLTFLoader'
import sceneUrl from '../../assets/campfire/Campfire_Scene.gltf?url'
import { CampfireActor, type VisitGate } from './CampfireActor'
import { CampfireDog } from './CampfireDog'
import { CampfireVillager, type Bubble } from './CampfireVillager'
import { iconRects } from './campfireIcons'
import { createSky, MOONLIGHT_DIR, SKY_COLOR } from './campfireSky'
import { createPost } from './campfirePost'
import { addSwayAttribute, applyWind, windKindFor, windTime } from './campfireWind'
import { FIRE, SEATS, FOLIAGE_RE, SEAT_TOP, THEME, type SceneMember } from './campfireConfig'
import { applySnowCover, createSnowfall, type Snowfall } from './campfireSnow'
import { buildTrees, buildFarGround } from './campfireTrees'
import { CampfireMailbox } from './CampfireMailbox'
import { useNotificationsStore } from '../../store/notificationsStore'

const props = defineProps<{
  /** Everyone who should currently be sitting at the fire (local player included). */
  members: SceneMember[]
  localUuid: string | null
}>()
const emit = defineEmits<{
  (e: 'invite'): void
  (e: 'self-contextmenu', ev: MouseEvent): void
}>()

const rootEl   = ref<HTMLDivElement | null>(null)
const hovering = ref(false)
const bubble = ref<Bubble | null>(null)
let bubbleEl: HTMLElement | null = null
function setBubbleEl(el: HTMLElement | null) { bubbleEl = el }
const canvasEl = ref<HTMLCanvasElement | null>(null)

// Seat bookkeeping is reactive so the "+" buttons follow it.
const occupied = ref<number[]>([])
const freeSeats = computed(() => [1, 2, 3, 4].filter(i => !occupied.value.includes(i)))

const tagEls  = new Map<string, HTMLElement>()
const seatEls = new Map<number, HTMLElement>()
function setTagEl(uuid: string, el: HTMLElement | null) { if (el) tagEls.set(uuid, el); else tagEls.delete(uuid) }
function setSeatEl(i: number, el: HTMLElement | null)   { if (el) seatEls.set(i, el);   else seatEls.delete(i) }

// ── three setup ───────────────────────────────────────────────────────────────
const SKY = SKY_COLOR
const FAKE_NEWS: [string, string][] = [
  ['BejaClient 1.2 is out', 'New launcher hub, mailbox and a faster start. Update in the launcher.'],
  ['Server event this weekend', 'Hoplite hosts a double-XP weekend. Jump in with your party.'],
  ['New cape unlocked', 'The Campfire cape is now in your locker. Equip it under Capes.'],
  ['Rewindr beta', 'Record and replay your sessions. Early access starts today.'],
  ['Maintenance tonight', 'Short downtime at 03:00 UTC while we move the servers.'],
]
const MAILBOX_SCALE = 1.2
const MAILBOX_NDC_DX = -0.19   // how far left of the fire (screen space, -1..1) the mailbox stands
const MAILBOX_NDC_DY = 0.17   // and how far below it
const HFOV_DEG = 50   // narrower view: the characters fill more of the card
const CAMERA_PITCH = 0.06   // tilt down a touch so the fire isn't cropped by the wide, short hub card
let renderer: WebGLRenderer | null = null
let scene: Scene | null = null
let camera: PerspectiveCamera | null = null
let fireLight: PointLight | null = null
let sky: ReturnType<typeof createSky> | null = null
let post: ReturnType<typeof createPost> | null = null
let moonLight: DirectionalLight | null = null
let dog: CampfireDog | null = null
let villager: CampfireVillager | null = null
let mailbox: CampfireMailbox | null = null
let snow: Snowfall | null = null
let mailboxBadgeEl: HTMLElement | null = null
function setMailboxBadgeEl(el: HTMLElement | null) { mailboxBadgeEl = el }
let mailboxHover = false
const notifStore = useNotificationsStore()
const unreadCount = computed(() => notifStore.unreadCount)
let bubbleKey = ''
// Frame-time guard: on a slow GPU, drop bloom and moon shadows instead of stuttering.
let perfFrames = 0
let perfSum = 0
let perfPrev = 0
let perfLevel = 0
let rafId: number | null = null
let ro: ResizeObserver | null = null
let paused = false
let worldReady = false
let simTime = 0
let lastFrame = 0
let width = 1, height = 1

const groundMeshes: Mesh[] = []
const actors: CampfireActor[] = []

// Only one character visits the fire at a time, and only while everyone else sits still and the dog is not moving.
let visitor: object | null = null   // an actor on a visit, or the villager
const visitGate: VisitGate = {
  request(a) {
    if (visitor && visitor !== a) return false
    if (villager?.wantsToStart) return false   // the villager is waiting for a calm moment
    for (const o of actors) if (o !== a && o.state !== 'idle') return false
    if (dog && dog.state !== 'absent' && dog.state !== 'rest') return false
    visitor = a
    return true
  },
  release(a) { if (visitor === a) visitor = null },
}
const seatOwner: (string | null)[] = SEATS.map(() => null)
const skinKeys = new Map<string, string>()
const ray = new Raycaster()
const DOWN = new Vector3(0, -1, 0)
const tmp = new Vector3()

// The glTF is 4 MB with embedded textures - parse it once and reuse across remounts.
let gltfPromise: Promise<GLTF> | null = null
function loadWorld(): Promise<GLTF> {
  if (!gltfPromise) gltfPromise = new GLTFLoader().loadAsync(sceneUrl)
  return gltfPromise
}

function groundY(x: number, z: number): number {
  ray.set(new Vector3(x, 50, z), DOWN)
  ray.far = 40
  const hit = ray.intersectObjects(groundMeshes, false)[0]
  return hit ? hit.point.y : 35
}

let treeDispose: (() => void) | null = null

function buildWorld(gltf: GLTF) {
  const markers: Record<string, Mesh> = {}
  let logMat: any = null
  let leafMat: any = null
  gltf.scene.traverse((o: Object3D) => {
    const mesh = o as Mesh
    if (o.name === 'camera' || o.name === 'camera_target') {
      if (mesh.isMesh) { mesh.visible = false; markers[o.name] = mesh }
      return
    }
    if (!mesh.isMesh) return
    if (/^Campfire/i.test(o.parent?.name || '') || /^Campfire/i.test(o.name || '')) hookCampfireBlock(mesh)
    const name = o.name || o.parent?.name || ''
    const mat: any = mesh.material
    mat.metalness = 0
    mat.roughness = 1
    const isWater = /water/i.test(name)
    const isFoliage = FOLIAGE_RE.test(name) && !/block/i.test(name)
    if (isWater) {
      mat.transparent = true; mat.opacity = 0.7; mat.depthWrite = false
    } else {
      mat.alphaTest = 0.5; mat.transparent = false
      if (isFoliage) mat.side = DoubleSide
    }
    if (mat.map) { mat.map.magFilter = NearestFilter; mat.map.minFilter = NearestFilter }
    const fullName = `${o.name} ${o.parent?.name ?? ""}`
    if (!logMat && /oak_log/i.test(fullName) && !/stripped/i.test(fullName)) logMat = mat.clone()
    if (!leafMat && /oak_leaves/i.test(fullName)) leafMat = mat.clone()
    const windKind = isFoliage ? windKindFor(name) : null   // solid blocks, water and lichen stay still
    if (windKind) {
      addSwayAttribute(mesh.geometry, windKind)
      applyWind(mat, windKind)
    }
    mesh.castShadow = !isWater
    mesh.receiveShadow = true
    if (!isWater && !isFoliage) {
      groundMeshes.push(mesh)
      const isFireBlock = /^Campfire/i.test(o.parent?.name || '') || /^Campfire/i.test(o.name || '')
      if (THEME === 'christmas' && !isFireBlock) applySnowCover(mat)
    }
  })
  scene!.add(gltf.scene)
  gltf.scene.updateMatrixWorld(true)

  const c = markers.camera, t = markers.camera_target
  if (c && t) {
    camera!.position.copy(new Box3().setFromObject(c).getCenter(new Vector3()))
    camera!.lookAt(new Box3().setFromObject(t).getCenter(new Vector3()))
    camera!.rotateX(-CAMERA_PITCH)
  } else {
    camera!.position.set(-27.4, 37, 3.4)
    camera!.lookAt(-13.5, 37, -4.9)
    camera!.rotateX(-CAMERA_PITCH)
  }
  if (logMat && leafMat) {
    const t0 = performance.now()
    // trees along the hub camera's view over the plateau (a cone from the camera, 24..60 blocks out)
    const cam = camera!.position
    const hd = new Vector3(-13.5 - cam.x, 0, -4.9 - cam.z).normalize()
    const positions: [number, number][] = []
    for (let d = 44; d <= 80; d += 7) {
      for (let lat = -0.62 * d; lat <= 0.62 * d; lat += 7) {
        if (Math.abs(lat / d + 0.22) < 0.12) continue   // keep the moon free
        positions.push([cam.x + hd.x * d - hd.z * lat + (Math.sin(d * 7.1 + lat * 3.3) * 2), cam.z + hd.z * d + hd.x * lat + (Math.cos(d * 5.3 + lat * 2.1) * 2)])
      }
    }
    const trees = buildTrees({
      logMaterial: logMat, leafMaterial: leafMat,
      groundTop: (x, z) => {
        ray.set(new Vector3(x, 60, z), DOWN)
        ray.far = 60
        const hit = ray.intersectObjects(groundMeshes, false)[0]
        return Math.max(hit ? hit.point.y : 0, 35)   // the far plain below carries the trees where the model has none
      },
      positions,
      keepClear: { x: FIRE.x, z: FIRE.z, r: 14 },
    })
    const far = buildFarGround({ x: cam.x + hd.x * 55, z: cam.z + hd.z * 55 }, { w: 130, l: 130 }, 34.8, 0)
    scene!.add(far.mesh)
    scene!.add(trees.group)
    treeDispose = () => { trees.dispose(); far.dispose() }
    console.info('[Campfire] trees built in', Math.round(performance.now() - t0), 'ms')
  }
}

// The fire is the glTF's own campfire block: a crossed-quad flame sprite (index count 12) above a coal/log block
// (index count 54). The sprite texture is redrawn procedurally ~12x/s (same pixel palette as the original art) and the
// coals pulse, so the real model burns instead of loose particles floating around it.
const FLAME_PAL: Array<[number, [number, number, number]]> = [
  [0.86, [251, 233, 160]], [0.66, [243, 201, 74]], [0.47, [227, 154, 26]], [0.30, [194, 86, 12]], [0.20, [165, 64, 10]],
]
let flameCanvas: HTMLCanvasElement | null = null
let flameTex: CanvasTexture | null = null
let flameMat: MeshBasicMaterial | null = null
let coalMat: any = null
let flameClock = 0
const hash = (x: number, y: number, f: number) => { const n = Math.sin(x * 127.1 + y * 311.7 + f * 74.7) * 43758.5453; return n - Math.floor(n) }

function hookCampfireBlock(mesh: Mesh) {
  const idx = mesh.geometry.index?.count ?? 0
  if (idx === 12) {
    flameCanvas = document.createElement('canvas'); flameCanvas.width = 16; flameCanvas.height = 16
    flameTex = new CanvasTexture(flameCanvas)
    flameTex.magFilter = NearestFilter; flameTex.minFilter = NearestFilter; flameTex.encoding = sRGBEncoding
    flameTex.flipY = false   // glTF UVs expect unflipped textures (CanvasTexture flips by default)
    paintFlame(0)
    // HDR (>1) basic material: unlit, and the bloom pass picks it up like the old particles did
    flameMat = new MeshBasicMaterial({ map: flameTex, alphaTest: 0.5, side: DoubleSide, fog: false, color: new Color(1.5, 1.5, 1.5) })
    mesh.material = flameMat
    mesh.castShadow = false
  } else if (idx === 54) {
    coalMat = mesh.material
  }
}

function paintFlame(t: number) {
  if (!flameCanvas || !flameTex) return
  const ctx = flameCanvas.getContext('2d')!
  const img = ctx.createImageData(16, 16)
  const f = Math.floor(t * 10)
  for (let y = 0; y < 16; y++) {
    const v = y / 15                                  // 0 = tip, 1 = base
    for (let x = 0; x < 16; x++) {
      const sway = Math.sin(t * 3.1 + v * 2.4) * (1 - v) * 1.4
      const ad = Math.abs(x + 0.5 - 8 - sway)
      // each column licks up to its own random height; the centre reaches highest
      const top = 0.5 + ad * 2.1 + (hash(x, 0, f) - 0.5) * 5.5 + Math.sin(t * 6 + x * 1.3) * 0.8
      if (y < top) continue
      const hw = 1.2 + Math.pow(v, 0.8) * 5.8
      if (ad > hw) continue
      const heat = (1 - ad / hw) * 0.55 + v * 0.5 + (hash(x, y, f) - 0.5) * 0.3
      let rgb: [number, number, number] | null = null
      for (const [th, c] of FLAME_PAL) if (heat > th) { rgb = c; break }
      if (!rgb) continue
      const o = (y * 16 + x) * 4
      img.data[o] = rgb[0]; img.data[o + 1] = rgb[1]; img.data[o + 2] = rgb[2]; img.data[o + 3] = 255
    }
  }
  ctx.putImageData(img, 0, 0)
  flameTex.needsUpdate = true
}

function updateFire(dt: number, t: number) {
  if (fireLight) fireLight.intensity = 30 + Math.sin(t * 13) * 3 + Math.sin(t * 23.7) * 2.2 + Math.sin(t * 5.3) * 3 + Math.sin(t * 41) * 1.4
  flameClock += dt
  if (flameClock >= 1 / 12) { flameClock = 0; paintFlame(t) }
  if (coalMat?.color) coalMat.color.setScalar(1.0 + Math.sin(t * 4.1) * 0.12 + Math.sin(t * 11.3) * 0.08)
}

// ── party → actors ────────────────────────────────────────────────────────────
function skinOf(m: SceneMember) { return { skinUrl: m.skinUrl, capeUrl: m.capeUrl, model: m.model } }
const skinKey = (m: SceneMember) => `${m.skinUrl}|${m.capeUrl}|${m.model}`

function activeActor(uuid: string) { return actors.find(a => a.uuid === uuid && !a.leaving) }

function pickSeat(uuid: string): number {
  if (uuid === props.localUuid) return seatOwner[0] === null ? 0 : -1
  for (let i = 1; i < SEATS.length; i++) if (seatOwner[i] === null) return i
  return -1
}

function syncActors() {
  if (!worldReady || !scene) return
  const wanted = new Map(props.members.map(m => [m.uuid, m]))

  for (const a of actors) if (!wanted.has(a.uuid)) a.leave()

  for (const m of props.members) {
    const existing = activeActor(m.uuid)
    if (existing) {
      const key = skinKey(m)
      if (skinKeys.get(m.uuid) !== key) { skinKeys.set(m.uuid, key); void existing.setSkin(skinOf(m)) }
      continue
    }
    const seatIdx = pickSeat(m.uuid)
    if (seatIdx < 0) continue          // no free seat yet - retried when one frees up
    const actor = new CampfireActor(m.uuid, seatIdx, SEATS[seatIdx], groundY)
    actor.peers = () => [
      ...actors.filter(a => a !== actor && a.state === 'idle').map(a => a.rig.position),
      ...(villager && villager.state === 'talk' ? [villager.group.position] : []),
    ]
    actor.visitGate = visitGate
    seatOwner[seatIdx] = m.uuid
    actors.push(actor)
    scene.add(actor.rig)
    skinKeys.set(m.uuid, skinKey(m))
    void actor.setSkin(skinOf(m))
  }
  occupied.value = seatOwner.map((o, i) => (o ? i : -1)).filter(i => i >= 0)
}

function reapActors(): boolean {
  let freed = false
  for (let i = actors.length - 1; i >= 0; i--) {
    const a = actors[i]
    if (a.state !== 'gone') continue
    if (hoverActor === a) setHover(false)
    visitGate.release(a)
    scene?.remove(a.rig)
    a.dispose()
    seatOwner[a.seatIndex] = null
    actors.splice(i, 1)
    freed = true
  }
  return freed
}

watch(() => props.members, syncActors, { deep: true })
watch(() => props.localUuid, syncActors)
watch(unreadCount, n => mailbox?.setUnread(n))

// ── overlay projection ────────────────────────────────────────────────────────
function project(world: Vector3, el: HTMLElement | undefined) {
  if (!el || !camera) return
  tmp.copy(world).project(camera)
  if (tmp.z > 1) { el.style.display = 'none'; return }
  el.style.display = ''
  el.style.transform = `translate(${((tmp.x * 0.5 + 0.5) * width).toFixed(1)}px, ${((-tmp.y * 0.5 + 0.5) * height).toFixed(1)}px)`
}

const headPos = new Vector3()
const seatPos = new Vector3()
function updateOverlay() {
  for (const a of actors) {
    const el = tagEls.get(a.uuid)
    if (!el) continue
    if (a.state === 'gone' || a.leaving) { el.style.display = 'none'; continue }
    project(a.headWorld(headPos), el)
  }
  for (const [i, el] of seatEls) project(seatPos.copy(SEATS[i].pos).setY(SEAT_TOP + 0.9), el)
  if (mailbox && mailbox.group.visible && mailboxBadgeEl) {
    if (unreadCount.value > 0) project(mailbox.badgeWorld(seatPos), mailboxBadgeEl)   // project() shows the element, so only call it when there is a count
    else mailboxBadgeEl.style.display = 'none'
  }
  if (villager && villager.active && bubbleEl) {
    const p = villager.group.position
    project(seatPos.set(p.x, p.y + 2.5, p.z), bubbleEl)
  }
}

// ── lifecycle ─────────────────────────────────────────────────────────────────
function step(dt: number) {
  simTime += dt
  windTime.value = simTime
  for (const a of actors) a.update(dt)
  if (reapActors()) syncActors()
  const local = actors.find(a => a.uuid === props.localUuid)
  dog?.update(dt, !!local && !local.leaving && local.state !== 'walk', visitor === null)
  stepVillager(dt, local)
  updateFire(dt, simTime)
  mailbox?.update(dt)
  snow?.update(dt, simTime)
  if (sky && camera) sky.update(simTime, camera.position)
  updateOverlay()
}

/** The villager may only come when everything is calm: the local player sits, nobody else is moving, the dog is not walking. */
function canStartEvent(): boolean {
  const local = actors.find(a => a.uuid === props.localUuid)
  if (!local || local.state !== 'idle') return false
  if (visitor !== null) return false
  if (actors.some(a => a.state !== 'idle')) return false
  if (dog && dog.state !== 'absent' && dog.state !== 'rest') return false
  return true
}

function stepVillager(dt: number, local: CampfireActor | undefined) {
  if (!villager) return
  if (villager.wantsToStart && villager.state === 'absent' && canStartEvent()) {
    visitor = villager
    villager.begin()
  }
  villager.update(dt, !!local && local.state === 'idle', local ? local.rig.position : null)
  if (visitor === villager && villager.state === 'absent') visitor = null
  local?.setChat(villager.playerCue, villager.group.position)
  const key = villager.bubble?.key ?? ''
  if (key !== bubbleKey) {
    bubbleKey = key
    bubble.value = villager.bubble ? { ...villager.bubble } : null
  }
}

/** Test button: let the villager come now (or as soon as the scene is calm). */
function triggerVillager(): 'started' | 'queued' | 'busy' | 'unavailable' {
  if (!villager || !worldReady) return 'unavailable'
  if (villager.active || villager.wantsToStart) return 'busy'
  villager.requestNow()
  return canStartEvent() ? 'started' : 'queued'
}

/** Hub test actions, numbered like SCENE_ACTION_LABELS (Ctrl+1..9 in dev builds). Returns a short message for the toast. */
function runAction(n: number): string {
  if (!worldReady) return 'The scene is not ready yet.'
  switch (n) {
    case 1: {
      const r = triggerVillager()
      return r === 'started' ? 'The villager is coming.' : r === 'queued' ? 'He will come once the fire is calm.' : r === 'busy' ? 'The villager is already on the way.' : 'The scene is not ready yet.'
    }
    case 2: return dog?.callNow() ? 'The dog will come once you sit at the fire and it is calm.' : 'The dog is already here.'
    case 3: return dog?.sendAway() ? 'The dog is leaving.' : 'The dog is not lying at the fire right now.'
    case 4: {
      const local = actors.find(a => a.uuid === props.localUuid)
      return local?.requestVisit() ? 'You will stand up shortly.' : 'You are not sitting right now.'
    }
    case 5: {
      const friends = actors.filter(a => a.uuid !== props.localUuid && a.state === 'idle')
      const pick = friends[Math.floor(Math.random() * friends.length)]
      return pick?.requestVisit() ? 'A friend will stand up shortly.' : 'No friend is sitting at the fire right now.'
    }
    case 6:
      sky?.shoot()
      return 'Shooting star!'
    case 7:
      for (const [title, body] of FAKE_NEWS) notifStore.addSystem(title, body)
      return 'Five letters arrived.'
    default: return ''
  }
}

function renderScene() {
  if (!renderer || !scene || !camera) return
  if (post) post.render(simTime)
  else renderer.render(scene, camera)
}

function frame(now: number) {
  rafId = requestAnimationFrame(frame)
  if (paused || !renderer || !scene || !camera) return
  const dt = Math.min(0.05, (now - lastFrame) / 1000)
  lastFrame = now
  step(dt)
  renderScene()
  watchFramerate(now)
}

// Render at least 1.5x the CSS size (downscaled by the browser = smoother edges), at most 2x.
function renderRatio(): number {
  return Math.min(Math.max(window.devicePixelRatio || 1, 1), 1.5)   // RAM: the composer targets are half-float + MSAA, their size grows with the square of this
}

function watchFramerate(now: number) {
  const delta = now - perfPrev
  perfPrev = now
  if (perfLevel >= 2 || delta <= 0 || delta > 250) return   // ignore the gaps left by pauses
  perfFrames++
  if (perfFrames > 30) perfSum += delta
  if (perfFrames < 150) return
  const avg = perfSum / (perfFrames - 30)
  perfFrames = 0
  perfSum = 0
  if (avg > 26) {   // under ~38 fps
    perfLevel++
    if (perfLevel === 1) {
      post?.setBloom(false)
      if (moonLight) moonLight.castShadow = false
      const native = Math.min(window.devicePixelRatio || 1, 2)   // also give up the supersampling
      renderer?.setPixelRatio(native)
      post?.setPixelRatio(native)
      resize()
    } else {
      post?.dispose()
      post = null
    }
    console.info('[Campfire] slow frames (' + avg.toFixed(1) + ' ms), reducing effects to level ' + perfLevel)
  } else {
    perfLevel = 2   // fine as it is, stop measuring
  }
}

/** Deterministic fast-forward (used by dev/test harnesses): run `seconds` of simulation, then render once. */
function advance(seconds: number) {
  if (!renderer || !scene || !camera) return
  for (let t = 0; t < seconds; t += 1 / 30) step(1 / 30)
  renderScene()
}

function resize() {
  const el = rootEl.value
  if (!el || !renderer || !camera) return
  width = Math.max(1, el.clientWidth)
  height = Math.max(1, el.clientHeight)
  renderer.setSize(width, height, false)
  post?.setSize(width, height)
  camera.aspect = width / height
  // Keep the horizontal field of view constant: the hub card is much wider than 16:9, and a fixed
  // vertical FOV would leave the characters tiny in the middle of a sea of sky.
  const vfov = 2 * Math.atan(Math.tan((HFOV_DEG * Math.PI) / 360) / camera.aspect) * (180 / Math.PI)
  camera.fov = Math.min(65, Math.max(18, vfov))
  camera.updateProjectionMatrix()
}

function onContextLost(e: Event) { e.preventDefault() }

// Hovering your own character draws an outline around it (like the old hub's hero viewer).
let hoverActor: CampfireActor | null = null
function setHover(on: boolean, actor: CampfireActor | null = null) {
  const next = on ? actor : null
  if (next === hoverActor) return
  hoverActor = next
  hovering.value = !!next || mailboxHover
  post?.setOutline(next ? [next.rig] : mailboxHover && mailbox ? mailbox.pickables : [])
}

function overMailbox(e: MouseEvent): boolean {
  if (!mailbox || !mailbox.group.visible || !camera || !canvasEl.value) return false
  const r = canvasEl.value.getBoundingClientRect()
  ray.setFromCamera({ x: ((e.clientX - r.left) / r.width) * 2 - 1, y: -((e.clientY - r.top) / r.height) * 2 + 1 } as any, camera)
  ray.far = 200
  return ray.intersectObjects(mailbox.pickables.filter(o => o.visible && o.parent?.visible !== false), false).length > 0
}

function setMailboxHover(on: boolean) {
  if (on === mailboxHover) return
  mailboxHover = on
  hovering.value = on || !!hoverActor
  post?.setOutline(on && mailbox ? mailbox.pickables : hoverActor ? [hoverActor.rig] : [])
}

function onPointerLeave() { setHover(false); setMailboxHover(false) }

function onClick(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('button')) return
  if (overMailbox(e)) notifStore.openDrawer()
}

function onPointerMove(e: PointerEvent) {
  const onBox = overMailbox(e)
  setMailboxHover(onBox)
  if (onBox) { if (hoverActor) setHover(false); return }
  const self = actors.find(a => a.uuid === props.localUuid && !a.leaving)
  if (!self || !camera || !canvasEl.value) { setHover(false); return }
  const r = canvasEl.value.getBoundingClientRect()
  ray.setFromCamera({ x: ((e.clientX - r.left) / r.width) * 2 - 1, y: -((e.clientY - r.top) / r.height) * 2 + 1 } as any, camera)
  ray.far = 200
  setHover(ray.intersectObject(self.rig, true).length > 0, self)
}

function onContextMenu(e: MouseEvent) {
  const self = actors.find(a => a.uuid === props.localUuid && !a.leaving)
  if (!self || !camera || !canvasEl.value) return
  const r = canvasEl.value.getBoundingClientRect()
  ray.setFromCamera({ x: ((e.clientX - r.left) / r.width) * 2 - 1, y: -((e.clientY - r.top) / r.height) * 2 + 1 } as any, camera)
  ray.far = 200
  if (ray.intersectObject(self.rig, true).length) emit('self-contextmenu', e)
}

onMounted(async () => {
  const canvas = canvasEl.value!
  renderer = new WebGLRenderer({ canvas, antialias: false })
  renderer.setPixelRatio(renderRatio())   // supersampled: the pixel-art edges stay clean even on a 1x display
  renderer.outputEncoding = sRGBEncoding
  renderer.physicallyCorrectLights = true
  renderer.shadowMap.enabled = true
  renderer.shadowMap.type = PCFSoftShadowMap
  canvas.addEventListener('webglcontextlost', onContextLost, false)

  scene = new Scene()
  scene.background = new Color(SKY)
  scene.fog = new Fog(SKY, 30, 105)
  camera = new PerspectiveCamera(55, 1, 0.1, 200)

  scene.add(new HemisphereLight(0x8290c0, 0x4a4038, 0.72))
  // moonlight: cool, from behind the scene, with its own shadows so the logs and figures get blue shadows
  // that contrast with the warm fire light
  const moon = new DirectionalLight(0x95a8e4, 0.42)
  moonLight = moon
  moon.position.copy(FIRE).addScaledVector(MOONLIGHT_DIR, 70)
  moon.target.position.copy(FIRE)
  moon.castShadow = true
  moon.shadow.mapSize.set(2048, 2048)
  Object.assign(moon.shadow.camera, { left: -26, right: 26, top: 26, bottom: -26, near: 1, far: 160 })
  moon.shadow.camera.updateProjectionMatrix()
  moon.shadow.bias = -0.0004
  moon.shadow.normalBias = 0.04
  scene.add(moon, moon.target)

  const horizon = new Color(SKY)
  sky = createSky(new Vector3(horizon.r, horizon.g, horizon.b))
  scene.add(sky.mesh)

  fireLight = new PointLight(0xff7424, 32, 20, 2)
  fireLight.position.copy(FIRE).add(new Vector3(0, 0.9, 0))
  fireLight.castShadow = true
  fireLight.shadow.mapSize.set(1024, 1024)
  fireLight.shadow.bias = -0.002
  scene.add(fireLight)

  ro = new ResizeObserver(resize)
  ro.observe(rootEl.value!)
  resize()
  try {
    post = createPost(renderer, scene, camera, width, height)
  } catch (e) {
    console.warn('[Campfire] post-processing unavailable, rendering directly', e)
    post = null
  }

  try {
    buildWorld(await loadWorld())
  } catch (e) {
    console.error('[Campfire] failed to load scene', e)
    return
  }
  dog = new CampfireDog(groundY)
  scene.add(dog.group)
  villager = new CampfireVillager(groundY)
  scene.add(villager.group)
  // Stand the mailbox where the camera can see it: a ray through a screen point left of the fire, onto the y=35 plateau.
  camera!.updateMatrixWorld(true)
  const fireNdc = FIRE.clone().project(camera!)
  const aim = new Vector3(fireNdc.x + MAILBOX_NDC_DX, fireNdc.y + MAILBOX_NDC_DY, 0.5).unproject(camera!).sub(camera!.position).normalize()
  const mbPos = camera!.position.clone().addScaledVector(aim, (35 - camera!.position.y) / aim.y)
  const camFwd = camera!.getWorldDirection(new Vector3())
  mailbox = new CampfireMailbox(mbPos, Math.atan2(-camFwd.x, -camFwd.z) + 0.35, MAILBOX_SCALE)
  scene.add(mailbox.group)
  void mailbox.load(groundY).then(() => mailbox?.setUnread(notifStore.unreadCount))
  if (THEME === 'christmas') {
    snow = createSnowfall({ x0: -24, x1: -4, z0: -18, z1: 12, yBottom: 34.6, yTop: 47 })
    scene.add(snow.points)
  }
  worldReady = true
  syncActors()
  lastFrame = performance.now()
  rafId = requestAnimationFrame(frame)
})

// <KeepAlive> keeps this mounted while another tab is open - stop rendering then.
onActivated(() => { paused = false; lastFrame = performance.now() })
onDeactivated(() => { paused = true })

onUnmounted(() => {
  if (rafId !== null) cancelAnimationFrame(rafId)
  canvasEl.value?.removeEventListener('webglcontextlost', onContextLost)
  ro?.disconnect()
  for (const a of actors) { scene?.remove(a.rig); a.dispose() }
  actors.length = 0
  flameTex?.dispose(); flameMat?.dispose(); flameTex = null; flameMat = null; flameCanvas = null; coalMat = null
  groundMeshes.length = 0
  treeDispose?.()
  treeDispose = null
  fireLight?.shadow.map?.dispose()
  post?.dispose()
  post = null
  sky?.dispose()
  sky = null
  moonLight = null
  if (dog) { scene?.remove(dog.group); dog.dispose(); dog = null }
  if (villager) { scene?.remove(villager.group); villager.dispose(); villager = null }
  if (mailbox) { mailbox.dispose(); mailbox = null }
  if (snow) { snow.dispose(); snow = null }
  renderer?.dispose()
  renderer = null
})

defineExpose({
  runAction,
  triggerVillager,
  advance,
  setPaused: (p: boolean) => { paused = p },
  isReady: () => worldReady,
})
</script>

<style scoped lang="scss">
.campfire {
  position: absolute;
  inset: 0;
  overflow: hidden;
  background: #121a30;
}

.campfire--hover { cursor: pointer; }

// Villager speech / trade bubble, positioned each frame above the villager's head.
.cf-bubble {
  position: absolute;
  left: 0;
  top: 0;
  translate: -50% calc(-100% - 8px);
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 9px;
  background: rgba(10, 12, 18, 0.88);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: #fff;
  font-size: 13px;
  font-weight: 700;
  white-space: nowrap;
  will-change: transform;

  &::after {   // little pointer towards the villager
    content: '';
    position: absolute;
    left: 50%;
    bottom: -6px;
    translate: -50% 0;
    border: 6px solid transparent;
    border-top-color: rgba(10, 12, 18, 0.88);
    border-bottom: 0;
  }
}

.cf-icon {
  display: block;
  width: 22px;
  height: 22px;
  image-rendering: pixelated;
}

.cf-arrow { color: rgba(255, 255, 255, 0.7); }

.campfire-canvas {
  display: block;
  width: 100%;
  height: 100%;
}

.campfire-overlay {
  position: absolute;
  inset: 0;
  pointer-events: none;
  overflow: hidden;
}

// Nametags and invite buttons are positioned each frame (transform) from the 3D projection.
.cf-tag,
.cf-mail-badge {
  position: absolute;
  left: 0;
  top: 0;
  translate: -50% -100%;
  min-width: 16px;
  padding: 1px 4px 0;
  text-align: center;
  background: #000;
  color: #ff3b3b;
  font-family: 'Minecrafter', 'Plus Jakarta Sans', sans-serif;
  font-size: 12px;
  line-height: 1.3;
  text-shadow: 1px 1px 0 #3f0000;
  pointer-events: none;
}

.cf-seat-invite {
  position: absolute;
  left: 0;
  top: 0;
  will-change: transform;
}

.cf-tag {
  display: flex;
  align-items: center;
  gap: 5px;
  margin: -6px 0 0 0;
  padding: 2px 6px 1px;
  // in-game nametag: translucent black plate, white pixel-font text with the dark drop shadow
  background: rgba(0, 0, 0, 0.42);
  color: #fff;
  font-family: 'Minecrafter', 'Plus Jakarta Sans', sans-serif;
  font-size: 12px;
  line-height: 1.25;
  letter-spacing: 0.04em;
  text-shadow: 1px 1px 0 #3f3f3f;
  white-space: nowrap;
  // centre horizontally on the projected point without clobbering the JS transform
  translate: -50% -100%;

  // separate black level label (same look as the nametag plate): yellow pixel star + level number
  .cf-level {
    display: flex;
    align-items: center;
    gap: 4px;
    margin: -2px 4px -1px -6px;
    padding: 2px 6px 1px;
    background: rgba(0, 0, 0, 0.62);
    font-weight: normal;

    b {
      font-weight: normal;
      color: #ffd83a;
      text-shadow: 1px 1px 0 #3f3a00;
    }
  }

  .cf-star {
    width: 11px;
    height: 11px;
    flex: none;
    margin-top: -1px;
    image-rendering: pixelated;
  }

  .cf-crown {
    width: 13px;
    height: 10px;
    flex: none;
    margin-top: -1px;
  }
}

.cf-seat-invite {
  width: 34px;
  height: 34px;
  border-radius: 0;   // the global button reset rounds corners; the hub scene overlays stay square
  translate: -50% -50%;
  pointer-events: auto;
  cursor: pointer;
  display: grid;
  place-items: center;
  padding: 0;
  border: 0;
  // inventory-slot look in black: dark well, black top/left edge, grey bottom/right edge
  background: rgba(8, 8, 10, 0.85);
  box-shadow: inset 2px 2px 0 #000, inset -2px -2px 0 rgba(80, 80, 84, 0.9);
  transition: background 120ms steps(2);

  .cf-plus {
    width: 18px;
    height: 18px;
    animation: cf-plus-bob 1.8s steps(2, jump-none) infinite;   // a two-frame hover, like a dropped item
  }

  &:hover {
    background: rgba(30, 30, 36, 0.92);
    box-shadow: inset 2px 2px 0 #000, inset -2px -2px 0 rgba(150, 150, 156, 0.95), 0 0 0 2px rgba(255, 255, 255, 0.85);
  }
}

@keyframes cf-plus-bob {
  from { transform: translateY(-1px); }
  to { transform: translateY(1px); }
}
</style>
