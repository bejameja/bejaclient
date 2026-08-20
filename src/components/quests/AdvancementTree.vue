<template>
  <div class="adv-shell" :class="{ fullscreen: isFullscreen }">

    <div class="adv-toolbar">
      <button class="adv-icon-btn" :title="isFullscreen ? 'Exit fullscreen (Esc)' : 'Fullscreen'" @click="toggleFullscreen">
        <svg v-if="!isFullscreen" viewBox="0 0 24 24"><path d="M4 9V4h5M15 4h5v5M20 15v5h-5M9 20H4v-5" stroke="currentColor" stroke-width="1.8" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>
        <svg v-else viewBox="0 0 24 24"><path d="M9 4v5H4M15 4v5h5M20 15h-5v5M9 20v-5H4" stroke="currentColor" stroke-width="1.8" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>
      </button>
    </div>

    <div
      class="adv-viewport"
      ref="viewportRef"
      :class="{ dragging }"
      @wheel.prevent="onWheel"
      @mousedown="onDragStart"
    >
      <div class="adv-canvas" :style="canvasStyle">

        <!-- Connector lines — drawn first, sit behind the nodes -->
        <svg class="adv-lines" :width="canvasW" :height="canvasH">
          <line
            v-for="e in edges"
            :key="e.from.id + '-' + e.to.id"
            :x1="e.from.x" :y1="e.from.y"
            :x2="e.to.x"   :y2="e.to.y"
            :class="{ lit: e.lit }"
          />
          <!-- Traveling energy flow on top of lit lines, pointing at what's next -->
          <line
            v-for="e in litEdges"
            :key="e.from.id + '-' + e.to.id + '-flow'"
            :x1="e.from.x" :y1="e.from.y"
            :x2="e.to.x"   :y2="e.to.y"
            class="line-flow"
          />
        </svg>

        <!-- Nodes -->
        <div
          v-for="n in nodes"
          :key="n.id"
          class="adv-node"
          :class="[n.stateClass, { root: n.isRoot, dimmed: n.dimmed }]"
          :style="{ left: n.x + 'px', top: n.y + 'px' }"
          @mouseenter="hoveredId = n.id"
          @mouseleave="hoveredId = hoveredId === n.id ? null : hoveredId"
          @click="onNodeClick(n)"
        >
          <div
            class="node-frame"
            :class="['shape-' + n.shape, { obtained: isObtained(n) }]"
            :style="{ backgroundImage: 'url(' + frameSrc(n) + ')' }"
          >
            <div class="node-icon">
              <!-- root -->
              <svg v-if="n.icon === 'chest'" viewBox="0 0 24 24"><path d="M3 9h18v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V9Z" stroke="currentColor" stroke-width="1.6" fill="none" stroke-linejoin="round"/><path d="M3 9V6a1 1 0 0 1 1-1h16a1 1 0 0 1 1 1v3" stroke="currentColor" stroke-width="1.6" fill="none" stroke-linejoin="round"/><rect x="10.5" y="11.5" width="3" height="3" rx="0.5" stroke="currentColor" stroke-width="1.4" fill="none"/></svg>
              <!-- real vanilla item textures for the 5 tracked quests -->
              <img v-else-if="ITEM_ICONS[n.icon]" :src="ITEM_ICONS[n.icon]" alt="" />
              <!-- locked / not yet discovered -->
              <img v-else-if="n.icon === 'lock'" :src="lockIcon" alt="" />
              <!-- fallback -->
              <svg v-else viewBox="0 0 24 24"><path d="M12 2 3 14h7l-1 8 10-12h-7l1-8z" fill="currentColor"/></svg>
            </div>

            <span v-if="n.stateClass === 'state-claimable'" class="node-badge">!</span>
            <span v-else-if="n.stateClass === 'state-claimed'" class="node-badge node-badge--done">
              <svg width="9" height="9" viewBox="0 0 10 10"><polyline points="1.5,5 4,7.5 8.5,2" stroke="currentColor" stroke-width="1.8" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </span>

            <!-- Claim burst — sparks + floating reward text -->
            <div v-if="burstingId === n.id" class="claim-burst">
              <span v-for="i in 12" :key="i" class="spark" :style="sparkStyle(i)" />
              <div class="claim-float">
                <span class="claim-float-xp">+{{ n.quest?.xp }} XP</span>
                <span class="claim-float-coins">+{{ n.quest?.coins }}</span>
              </div>
            </div>
          </div>

          <div v-if="!n.isRoot && !n.locked" class="node-progress-track">
            <div class="node-progress-fill" :style="{ width: pct(n.quest!) + '%' }" />
          </div>

          <!-- Tooltip -->
          <Transition name="adv-tip">
            <div v-if="hoveredId === n.id && !dragging" class="node-tooltip" :class="{ 'flip-up': n.y < 215 }">
              <div class="tip-title">{{ n.isRoot ? 'Weekly Quests' : (n.locked ? '???' : n.quest!.name) }}</div>
              <template v-if="!n.isRoot && !n.locked">
                <div class="tip-progress">{{ n.quest!.progress }} / {{ n.quest!.goal }}</div>
                <div class="tip-rewards">
                  <span class="tip-chip tip-chip--xp">
                    <svg viewBox="0 0 24 24" fill="currentColor"><path d="M13 2 3 14h7l-1 8 10-12h-7l1-8z"/></svg>
                    {{ n.quest!.xp }} XP
                  </span>
                  <span class="tip-chip tip-chip--coins">
                    <img :src="gemIcon" alt="" />
                    {{ n.quest!.coins }}
                  </span>
                </div>
                <div class="tip-hint">
                  <span v-if="n.stateClass === 'state-claimed'">Claimed</span>
                  <span v-else-if="n.stateClass === 'state-claimable'">Click to claim</span>
                  <span v-else>In progress</span>
                </div>
              </template>
              <div v-else-if="n.locked" class="tip-hint">Locked</div>
              <div v-else class="tip-hint">Resets weekly</div>
            </div>
          </Transition>
        </div>

      </div>

      <!-- Ambient dust — camera-space, drifts independently of pan/zoom -->
      <div class="adv-dust" aria-hidden="true">
        <span v-for="i in DUST_COUNT" :key="i" class="dust-mote" :style="dustStyle(i)" />
      </div>
    </div>

    <div v-if="isFullscreen" class="adv-zoom-controls">
      <button class="adv-icon-btn" title="Zoom in" @click="zoomBy(1.25)">+</button>
      <button class="adv-icon-btn" title="Fit to screen" @click="fitView">⟳</button>
      <button class="adv-icon-btn" title="Zoom out" @click="zoomBy(0.8)">−</button>
    </div>

    <!-- Minimap — only useful once the tree extends past the viewport, i.e. fullscreen -->
    <div v-if="isFullscreen" class="adv-minimap" @click="onMinimapClick">
      <div
        v-for="n in nodes"
        :key="'mm-' + n.id"
        class="mm-dot"
        :class="n.stateClass"
        :style="minimapNodeStyle(n)"
      />
      <div
        class="mm-viewport"
        :style="{ left: minimapViewport.x + 'px', top: minimapViewport.y + 'px', width: minimapViewport.w + 'px', height: minimapViewport.h + 'px' }"
      />
    </div>

  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted, nextTick } from 'vue'
import type { Quest } from '../../types'
import { playSuccess } from '../../composables/useSounds'
import gemIcon from '../../assets/violet-gem.webp'
import frameTaskUnobtained from '../../assets/icons/frame-task-unobtained.png'
import frameTaskObtained from '../../assets/icons/frame-task-obtained.png'
import frameGoalUnobtained from '../../assets/icons/frame-goal-unobtained.png'
import frameGoalObtained from '../../assets/icons/frame-goal-obtained.png'
import frameChallengeUnobtained from '../../assets/icons/frame-challenge-unobtained.png'
import frameChallengeObtained from '../../assets/icons/frame-challenge-obtained.png'
import iconFireworkRocket from '../../assets/icons/items/firework_rocket.png'
import iconClock from '../../assets/icons/items/clock.png'
import iconElytra from '../../assets/icons/items/elytra.png'
import iconNameTag from '../../assets/icons/items/name_tag.png'
import iconCake from '../../assets/icons/items/cake.png'
import stoneTexture from '../../assets/icons/advancements-stone.png'
import lockIcon from '../../assets/icons/icon-locked.png'

// Bound into the scoped <style> block below via v-bind() so the tiled stone
// background can reference an imported asset URL.
const stoneBg = `url(${stoneTexture})`

// Real vanilla item textures (PrismarineJS/minecraft-assets, extracted 1:1 from
// the game) standing in for each quest's SVG icon — kept separate from the
// decorative "lock"/"star"/"chest" glyphs, which stay hand-drawn SVGs.
const ITEM_ICONS: Record<string, string> = {
  firework_rocket: iconFireworkRocket,
  clock: iconClock,
  elytra: iconElytra,
  name_tag: iconNameTag,
  cake: iconCake,
}

const props = defineProps<{
  quests: Quest[]
  claimingId: string | null
  searchQuery?: string
  statusFilter?: 'all' | 'claimable' | 'progress' | 'claimed' | 'locked'
}>()

const emit = defineEmits<{ claim: [quest: Quest] }>()

const hoveredId = ref<string | null>(null)

// ── Layout ────────────────────────────────────────────────────────────────────
// Client-side presentational tree: known quest ids are arranged into branches
// resembling vanilla's advancement layout. Any quest the backend adds that we
// don't recognise falls back to its own root-level branch so the tree never
// breaks — it just grows a new row. Beyond the real, backend-tracked quests,
// a large field of locked/"???" branches (vanilla's hidden-advancement style)
// fills the tree out — purely decorative until real quests are designed for them.
const COL_W = 168
const ROW_H = 132
const PAD_X = 90
const PAD_Y = 84

type Shape = 'task' | 'goal' | 'challenge'
interface LayoutDef { id: string; parent: string | null; col: number; row: number; icon: string; shape: Shape; locked?: boolean }

const KNOWN_LAYOUT: LayoutDef[] = [
  { id: 'launch_3',    parent: 'root',     col: 1, row: 0, icon: 'firework_rocket', shape: 'task' },
  { id: 'playtime_60', parent: 'launch_3', col: 2, row: 0, icon: 'clock',           shape: 'task' },
  { id: 'cosmetic_1',  parent: 'root',     col: 1, row: 1, icon: 'elytra',          shape: 'goal' },
  { id: 'friend_1',    parent: 'root',     col: 1, row: 2, icon: 'name_tag',        shape: 'task' },
  { id: 'party_1',     parent: 'friend_1', col: 2, row: 2, icon: 'cake',            shape: 'goal' },
]

// Procedurally builds a zigzagging locked chain, occasionally forking a side
// branch (a node parenting off two steps back rather than the immediate
// previous one) so branches read as an organic tree rather than a straight line.
function buildLockedChain(
  prefix: string,
  rootParent: string,
  startCol: number,
  startRow: number,
  count: number,
  colStep: number,
  rowPattern: number[],
  forkAt: number[] = [],
): LayoutDef[] {
  const defs: LayoutDef[] = []
  let col = startCol
  let row = startRow
  let prevId = rootParent
  let prevPrevId: string | null = null

  for (let i = 0; i < count; i++) {
    col += colStep
    row += rowPattern[i % rowPattern.length]
    const id = `${prefix}_${i}`
    const parent = forkAt.includes(i) && prevPrevId ? prevPrevId : prevId
    const shape: Shape = i === count - 1 ? 'challenge' : (i % 3 === 2 ? 'goal' : 'task')
    defs.push({ id, parent, col, row, icon: 'lock', shape, locked: true })
    prevPrevId = prevId
    prevId = id
  }
  return defs
}

// 61 decorative nodes across 6 branches + 5 real + 1 root = 67 total.
const DECORATIVE_LAYOUT: LayoutDef[] = [
  ...buildLockedChain('explorer',  'root',     2, -4, 11, 1, [0, -1, 0, 1], [4, 8]),
  ...buildLockedChain('builder',   'root',     2,  5, 10, 1, [0, 1, 0, -1], [5]),
  ...buildLockedChain('social',    'party_1',  3,  2, 10, 1, [1, 0, -1, 0], [3, 7]),
  ...buildLockedChain('collector', 'root',     2,  9, 10, 1, [0, -1, 1, 0], [6]),
  ...buildLockedChain('survivor',  'root',     2, -9, 10, 1, [-1, 0, 1, 0], [4]),
  ...buildLockedChain('legend',    'collector_9', 12, 9, 10, 1, [0, 1, 0, -1], [5]),
]

interface TreeNode {
  id: string
  isRoot: boolean
  quest: Quest | null
  parent: string | null
  col: number
  row: number
  x: number
  y: number
  icon: string
  shape: Shape
  locked: boolean
  stateClass: string
  dimmed: boolean
}

function stateFor(q: Quest): string {
  if (q.claimed) return 'state-claimed'
  if (q.progress >= q.goal) return 'state-claimable'
  if (q.progress > 0) return 'state-progress'
  return 'state-todo'
}

// Vanilla only has two frame textures per shape — completed (gold "obtained")
// vs not (grey "unobtained"). "Claimable" reads as obtained too since the
// goal itself has been met; only the reward pickup is still pending.
const FRAME_IMAGES: Record<Shape, { obtained: string; unobtained: string }> = {
  task:      { obtained: frameTaskObtained,      unobtained: frameTaskUnobtained },
  goal:      { obtained: frameGoalObtained,      unobtained: frameGoalUnobtained },
  challenge: { obtained: frameChallengeObtained, unobtained: frameChallengeUnobtained },
}

function matchesSearch(q: Quest | null): boolean {
  const query = props.searchQuery?.trim().toLowerCase()
  if (!query) return true
  return !!q && q.name.toLowerCase().includes(query)
}

function matchesStatus(stateClass: string): boolean {
  const f = props.statusFilter ?? 'all'
  return f === 'all' || stateClass === `state-${f}`
}

function isObtained(n: TreeNode): boolean {
  return n.stateClass === 'state-claimable' || n.stateClass === 'state-claimed'
}

function frameSrc(n: TreeNode): string {
  const set = FRAME_IMAGES[n.shape]
  return isObtained(n) ? set.obtained : set.unobtained
}

const nodes = computed<TreeNode[]>(() => {
  const byId = new Map(props.quests.map(q => [q.id, q]))
  // Always include the known layout nodes, even if their quest data hasn't
  // loaded yet — decorative branches (e.g. 'social') parent onto real quest
  // ids like party_1, so those anchor nodes must always exist in the tree.
  const defs = [...KNOWN_LAYOUT]

  // Unknown quests (not in our hand-authored layout) become their own
  // root-level branches, stacked below the known rows.
  let extraRow = Math.max(2, ...defs.map(d => d.row)) + 1
  for (const q of props.quests) {
    if (!KNOWN_LAYOUT.some(d => d.id === q.id)) {
      defs.push({ id: q.id, parent: 'root', col: 1, row: extraRow++, icon: 'star', shape: 'task' })
    }
  }

  defs.push(...DECORATIVE_LAYOUT)

  const minRow = Math.min(1, ...defs.map(d => d.row))

  const list: TreeNode[] = [{
    id: 'root', isRoot: true, quest: null, parent: null, col: 0, row: 1,
    x: PAD_X, y: PAD_Y + ROW_H * (1 - minRow), icon: 'chest', shape: 'task', locked: false,
    stateClass: 'state-claimed', dimmed: false,
  }]

  for (const d of defs) {
    const q = byId.get(d.id) ?? null
    const stateClass = (d.locked || !q) ? 'state-locked' : stateFor(q)
    list.push({
      id: d.id,
      isRoot: false,
      quest: q,
      parent: d.parent,
      col: d.col,
      row: d.row,
      x: PAD_X + COL_W * d.col,
      y: PAD_Y + ROW_H * (d.row - minRow),
      icon: d.icon,
      shape: d.shape,
      // A known node whose quest data hasn't arrived yet reads as locked/"???"
      // rather than crashing on a null quest.
      locked: !!d.locked || !q,
      stateClass,
      dimmed: !(matchesSearch(q) && matchesStatus(stateClass)),
    })
  }

  return list
})

const edges = computed(() => {
  const byId = new Map(nodes.value.map(n => [n.id, n]))
  return nodes.value
    .filter(n => n.parent)
    .map(n => {
      const from = byId.get(n.parent!)!
      const lit = n.isRoot || n.stateClass === 'state-claimable' || n.stateClass === 'state-claimed' || n.stateClass === 'state-progress'
      return { from: { id: from.id, x: from.x + 36, y: from.y + 36 }, to: { id: n.id, x: n.x + 36, y: n.y + 36 }, lit }
    })
})

// Lit edges get a second, animated "energy flow" line drawn on top — a
// traveling dash that reads as progress/light moving toward the next quest.
const litEdges = computed(() => edges.value.filter(e => e.lit))

const canvasW = computed(() => Math.max(...nodes.value.map(n => n.x)) + 168)
const canvasH = computed(() => Math.max(...nodes.value.map(n => n.y)) + 140)

function pct(q: Quest): number {
  return Math.min(100, Math.round((q.progress / q.goal) * 100))
}

// ── Claim burst ──────────────────────────────────────────────────────────────
// Fires the instant a claimable node is clicked rather than waiting on the
// async claim round-trip — the goal is already met, so the payoff should
// feel immediate rather than lagging behind a network call.
const burstingId = ref<string | null>(null)
let burstTimer: ReturnType<typeof setTimeout> | null = null

function spawnClaimBurst(id: string) {
  burstingId.value = id
  if (burstTimer) clearTimeout(burstTimer)
  burstTimer = setTimeout(() => {
    if (burstingId.value === id) burstingId.value = null
  }, 900)
}

function sparkStyle(i: number) {
  const angle = (i / 12) * 360 + ((i * 37) % 40 - 20)
  const dist = 24 + ((i * 13) % 20)
  return {
    '--spark-angle': angle + 'deg',
    '--spark-dist': dist + 'px',
    animationDelay: (i * 16) + 'ms',
  }
}

function onNodeClick(n: TreeNode) {
  if (n.isRoot || !n.quest) return
  if (n.stateClass === 'state-claimable' && props.claimingId !== n.quest.id) {
    spawnClaimBurst(n.id)
    playSuccess()
    emit('claim', n.quest)
  }
}

// ── Fullscreen + pan/zoom ──────────────────────────────────────────────────────
const viewportRef = ref<HTMLElement | null>(null)
const isFullscreen = ref(false)
const scale = ref(1)
const panX = ref(0)
const panY = ref(0)
const dragging = ref(false)

const canvasStyle = computed(() => ({
  width: canvasW.value + 'px',
  height: canvasH.value + 'px',
  transform: `translate(${panX.value}px, ${panY.value}px) scale(${scale.value})`,
  transformOrigin: '0 0',
}))

function clamp(v: number, lo: number, hi: number) {
  return Math.min(hi, Math.max(lo, v))
}

// Keeps the stone-tiled canvas from ever being dragged/zoomed past its own
// edges — no revealing empty viewport beyond where the background ends.
function clampPan(px: number, py: number, s: number) {
  const vp = viewportRef.value
  if (!vp) return { x: px, y: py }
  const vw = vp.clientWidth
  const vh = vp.clientHeight
  const cw = canvasW.value * s
  const ch = canvasH.value * s
  const minX = Math.min(0, vw - cw)
  const maxX = Math.max(0, vw - cw)
  const minY = Math.min(0, vh - ch)
  const maxY = Math.max(0, vh - ch)
  return { x: clamp(px, minX, maxX), y: clamp(py, minY, maxY) }
}

// The smallest scale at which the canvas still fully covers the viewport —
// zooming out past this would reveal empty space beyond the background.
function minScale(): number {
  const vp = viewportRef.value
  if (!vp || !vp.clientWidth || !vp.clientHeight) return 0.25
  return Math.max(vp.clientWidth / canvasW.value, vp.clientHeight / canvasH.value)
}

function zoomBy(factor: number, centerX?: number, centerY?: number) {
  const vp = viewportRef.value
  const px = centerX ?? (vp ? vp.clientWidth / 2 : 0)
  const py = centerY ?? (vp ? vp.clientHeight / 2 : 0)
  const newScale = clamp(scale.value * factor, minScale(), 2.5)
  const nx = px - (px - panX.value) * (newScale / scale.value)
  const ny = py - (py - panY.value) * (newScale / scale.value)
  scale.value = newScale
  const clamped = clampPan(nx, ny, newScale)
  panX.value = clamped.x
  panY.value = clamped.y
}

function onWheel(e: WheelEvent) {
  const rect = viewportRef.value!.getBoundingClientRect()
  zoomBy(e.deltaY < 0 ? 1.1 : 0.9, e.clientX - rect.left, e.clientY - rect.top)
}

let dragStart = { x: 0, y: 0, panX: 0, panY: 0 }
let dragRaf = 0
let pendingEvent: MouseEvent | null = null

function onDragStart(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('.adv-node')) return
  dragging.value = true
  dragStart = { x: e.clientX, y: e.clientY, panX: panX.value, panY: panY.value }
  window.addEventListener('mousemove', onDragMove)
  window.addEventListener('mouseup', onDragEnd)
}

// Batched via rAF so 67 nodes' worth of re-render only happens once per
// frame, instead of once per raw mousemove event (which fires far more
// often than the screen can repaint and was the main source of drag jank).
function applyPendingDrag() {
  dragRaf = 0
  const e = pendingEvent
  if (!e || !dragging.value) return
  const nx = dragStart.panX + (e.clientX - dragStart.x)
  const ny = dragStart.panY + (e.clientY - dragStart.y)
  const clamped = clampPan(nx, ny, scale.value)
  panX.value = clamped.x
  panY.value = clamped.y
}

function onDragMove(e: MouseEvent) {
  if (!dragging.value) return
  pendingEvent = e
  if (!dragRaf) dragRaf = requestAnimationFrame(applyPendingDrag)
}
function onDragEnd() {
  dragging.value = false
  if (dragRaf) { cancelAnimationFrame(dragRaf); dragRaf = 0 }
  pendingEvent = null
  window.removeEventListener('mousemove', onDragMove)
  window.removeEventListener('mouseup', onDragEnd)
}

// ── Ambient dust ──────────────────────────────────────────────────────────────
// Lives in viewport (camera) space, not canvas (world) space, so it drifts
// independently of pan/zoom — a fixed count is enough regardless of tree size.
const DUST_COUNT = 16
function dustStyle(i: number) {
  const left = (i * 53) % 100
  const size = 2 + (i % 3)
  return {
    left: left + '%',
    width: size + 'px',
    height: size + 'px',
    animationDelay: ((i * 0.7) % 6) + 's',
    animationDuration: (7 + (i % 5)) + 's',
  }
}

// ── Minimap (fullscreen only) ───────────────────────────────────────────────
const MINIMAP_W = 170
const MINIMAP_H = 110

const minimapScale = computed(() =>
  Math.min(MINIMAP_W / canvasW.value, MINIMAP_H / canvasH.value))

const minimapViewport = computed(() => {
  const vp = viewportRef.value
  const s = minimapScale.value
  if (!vp || !scale.value) return { x: 0, y: 0, w: 0, h: 0 }
  return {
    x: (-panX.value / scale.value) * s,
    y: (-panY.value / scale.value) * s,
    w: (vp.clientWidth / scale.value) * s,
    h: (vp.clientHeight / scale.value) * s,
  }
})

function minimapNodeStyle(n: TreeNode) {
  const s = minimapScale.value
  return { left: (n.x * s) + 'px', top: (n.y * s) + 'px' }
}

function onMinimapClick(e: MouseEvent) {
  const vp = viewportRef.value
  if (!vp) return
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  const s = minimapScale.value
  const wx = (e.clientX - rect.left) / s
  const wy = (e.clientY - rect.top) / s
  const nx = vp.clientWidth / 2 - wx * scale.value
  const ny = vp.clientHeight / 2 - wy * scale.value
  const clamped = clampPan(nx, ny, scale.value)
  panX.value = clamped.x
  panY.value = clamped.y
}

function fitView() {
  nextTick(() => {
    const vp = viewportRef.value
    if (!vp) return
    const vw = vp.clientWidth
    const vh = vp.clientHeight
    if (!vw || !vh) return
    const fitScale = Math.min(vw / canvasW.value, vh / canvasH.value) * 0.92
    // Never shrink nodes past a legible size just to cram all 67 in view —
    // if the full tree doesn't fit at that size, anchor on the real/tracked
    // quests near the root instead and let panning reveal the rest.
    const s = clamp(fitScale, 1, 1.6)
    scale.value = s

    let nx: number, ny: number
    if (fitScale >= 1) {
      nx = (vw - canvasW.value * s) / 2
      ny = (vh - canvasH.value * s) / 2
    } else {
      const root = nodes.value.find(n => n.isRoot)
      const cx = root ? root.x + 38 : 0
      const cy = root ? root.y + 38 : 0
      nx = vw * 0.14 - cx * s
      ny = vh / 2 - cy * s
    }
    const clamped = clampPan(nx, ny, s)
    panX.value = clamped.x
    panY.value = clamped.y
  })
}

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && isFullscreen.value) isFullscreen.value = false
}

watch(isFullscreen, (val) => {
  document.body.style.overflow = val ? 'hidden' : ''
  fitView()
})

onMounted(() => {
  fitView()
  window.addEventListener('resize', fitView)
  window.addEventListener('keydown', onKeydown)
})
onUnmounted(() => {
  window.removeEventListener('resize', fitView)
  window.removeEventListener('keydown', onKeydown)
  window.removeEventListener('mousemove', onDragMove)
  window.removeEventListener('mouseup', onDragEnd)
  if (dragRaf) cancelAnimationFrame(dragRaf)
  if (burstTimer) clearTimeout(burstTimer)
  document.body.style.overflow = ''
})
</script>

<style lang="scss" scoped>
.adv-shell {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background-color: #17140f;
  border: 3px solid #000;
  box-shadow:
    inset 3px 3px 0 rgba(255,255,255,0.06),
    inset -3px -3px 0 rgba(0,0,0,0.7);
  padding: 4px;
  transition: background 200ms ease;

  &.fullscreen {
    position: fixed;
    inset: 0;
    z-index: 2000;
    border-width: 0;
    padding: 0;
  }
}

.adv-toolbar {
  position: absolute;
  top: 10px;
  right: 10px;
  z-index: 10;
  display: flex;
  gap: 6px;
}

.adv-zoom-controls {
  position: absolute;
  bottom: 14px;
  right: 14px;
  z-index: 10;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.adv-icon-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #1c1c1c;
  border: 2px solid #000;
  color: #ccc;
  cursor: pointer;
  font-size: 15px;
  box-shadow:
    inset 1px 1px 0 rgba(255,255,255,0.12),
    inset -1px -1px 0 rgba(0,0,0,0.65);

  svg { width: 16px; height: 16px; }

  &:hover { background: #262626; color: #fff; }
  &:active { background: #141414; }
}

.adv-viewport {
  position: relative;
  overflow: hidden;
  width: 100%;
  flex: 1;
  min-height: 0;
  cursor: grab;
  // Vignette lives on the viewport (not the canvas) so it stays fixed to the
  // screen edges while the stone tiling underneath pans/zooms with the tree.
  background: radial-gradient(ellipse at 50% 0%, transparent, rgba(0,0,0,0.45) 70%);

  &.dragging { cursor: grabbing; }
}

.adv-canvas {
  position: absolute;
  top: 0;
  left: 0;
  will-change: transform;
  background-color: #17140f;
  background-image: v-bind(stoneBg);
  background-repeat: repeat;
  background-size: 32px 32px;
  image-rendering: pixelated;
}

// ── Connector lines ─────────────────────────────────────────────────────────
.adv-lines {
  position: absolute;
  inset: 0;
  pointer-events: none;

  line {
    stroke: rgba(255, 255, 255, 0.14);
    stroke-width: 2;
    transition: stroke 400ms ease;

    &.lit { stroke: rgba(255, 179, 0, 0.55); }
  }

  .line-flow {
    fill: none;
    stroke: rgba(255, 228, 150, 0.85);
    stroke-width: 2;
    stroke-linecap: round;
    stroke-dasharray: 3 24;
    animation: line-flow-dash 1s linear infinite;
  }
}

@keyframes line-flow-dash {
  to { stroke-dashoffset: -27; }
}

// ── Node ──────────────────────────────────────────────────────────────────────
.adv-node {
  position: absolute;
  width: 76px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  z-index: 1;
  user-select: none;
  transition: opacity 220ms ease;

  &:not(.state-locked):hover .node-frame { transform: scale(1.08); }

  &.dimmed {
    opacity: 0.22;
    filter: grayscale(0.6);
  }
}

.node-frame {
  position: relative;
  width: 72px;
  height: 72px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-repeat: no-repeat;
  background-position: center;
  background-size: 100% 100%;
  image-rendering: pixelated;
  transition: transform 140ms cubic-bezier(0.34, 1.56, 0.64, 1), filter 200ms ease;
  cursor: default;

  // real vanilla goal-frame art is a taller, rounded shape — not a square crop
  &.shape-goal {
    width: 76px;
    height: 90px;
  }
}

.node-icon {
  position: relative;
  width: 38px;
  height: 38px;
  color: #6b6b6b;
  filter: grayscale(1);
  opacity: 0.6;
  transition: color 200ms ease, opacity 200ms ease, filter 200ms ease;

  svg { width: 100%; height: 100%; }
  img { width: 100%; height: 100%; object-fit: contain; image-rendering: pixelated; }
}

// ── States ────────────────────────────────────────────────────────────────────
// Frame art itself only encodes obtained/unobtained (vanilla has no third
// texture) — "in progress" is conveyed by the icon brightening, not the frame.
.state-progress .node-icon { color: #d9d9d9; filter: none; opacity: 1; }

.state-claimable {
  .node-frame {
    cursor: pointer;
    animation: adv-pulse 1.6s ease-in-out infinite;
  }
  .node-icon { filter: none; opacity: 1; }
}

@keyframes adv-pulse {
  0%, 100% { filter: drop-shadow(0 0 0 rgba(255,215,64,0)); }
  50%       { filter: drop-shadow(0 0 6px rgba(255,215,64,0.7)); }
}

.state-claimed .node-icon { filter: none; opacity: 1; }

// vanilla-style hidden/undiscovered advancement — dim, inert, padlock icon
.state-locked {
  .node-frame { opacity: 0.55; }
  .node-icon { color: #4a4a4a; opacity: 0.8; }
}

.adv-node.root .node-frame {
  width: 78px;
  height: 78px;
}

// ── Badges ────────────────────────────────────────────────────────────────────
.node-badge {
  position: absolute;
  top: -8px;
  right: -8px;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  background: #ffb300;
  color: #241c00;
  border: 2px solid #0a0a0b;
  z-index: 1;

  &--done { background: #4caf50; color: #0a0a0b; }
}

// ── Claim burst ───────────────────────────────────────────────────────────────
.claim-burst {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 6;
}

.spark {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 4px;
  height: 4px;
  background: #ffdd66;
  border-radius: 50%;
  box-shadow: 0 0 4px 1px rgba(255, 221, 102, 0.85);
  transform: translate(-50%, -50%) rotate(var(--spark-angle)) translateX(0) scale(1);
  animation: spark-fly 650ms ease-out forwards;
}

@keyframes spark-fly {
  0%   { opacity: 1; transform: translate(-50%, -50%) rotate(var(--spark-angle)) translateX(0) scale(1); }
  100% { opacity: 0; transform: translate(-50%, -50%) rotate(var(--spark-angle)) translateX(var(--spark-dist)) scale(0.3); }
}

.claim-float {
  position: absolute;
  top: -2px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  animation: claim-float-up 900ms ease-out forwards;
}

@keyframes claim-float-up {
  0%   { opacity: 0; transform: translate(-50%, 0); }
  18%  { opacity: 1; }
  100% { opacity: 0; transform: translate(-50%, -38px); }
}

.claim-float-xp {
  font-size: 12px;
  font-weight: 700;
  color: #7ee787;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.9);
}

.claim-float-coins {
  font-size: 11px;
  font-weight: 700;
  color: #c084fc;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.9);
}

// ── Progress bar under node ───────────────────────────────────────────────────
.node-progress-track {
  width: 60px;
  height: 4px;
  background: rgba(255,255,255,0.08);
  border-radius: 4px;
  overflow: hidden;
}

.node-progress-fill {
  height: 100%;
  background: #ffb300;
  transition: width 0.4s ease;
}

// ── Tooltip ───────────────────────────────────────────────────────────────────
.node-tooltip {
  position: absolute;
  top: 100px;
  left: 50%;
  transform: translateX(-50%);
  width: 190px;
  background: #141414;
  border: 1px solid rgba(255,255,255,0.14);
  box-shadow: 0 8px 20px rgba(0,0,0,0.5);
  padding: 8px 10px;
  z-index: 20;
  pointer-events: none;

  &.flip-up {
    top: auto;
    bottom: 100px;
  }
}

.adv-tip-enter-active, .adv-tip-leave-active { transition: opacity 120ms ease, transform 120ms ease; }
.adv-tip-enter-from, .adv-tip-leave-to { opacity: 0; transform: translateX(-50%) translateY(4px); }

.tip-title {
  font-size: 12px;
  color: $text-primary;
  margin-bottom: 4px;
}

.tip-progress {
  font-size: 10px;
  color: $text-muted;
  margin-bottom: 6px;
}

.tip-rewards {
  display: flex;
  gap: 5px;
  margin-bottom: 6px;
}

.tip-chip {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: 8.5px;
  padding: 2px 5px;
  border: 1px solid;
  white-space: nowrap;

  svg, img { width: 9px; height: 9px; }

  &--xp { color: #9aa0a6; background: rgba(255,255,255,0.04); border-color: rgba(255,255,255,0.14); }
  &--coins { color: #c084fc; background: rgba(168, 85, 247, 0.08); border-color: rgba(192, 132, 252, 0.35); }
}

.tip-hint {
  font-size: 10px;
  color: #ffb300;
  letter-spacing: 0.02em;
}

// ── Ambient dust ──────────────────────────────────────────────────────────────
.adv-dust {
  position: absolute;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
  z-index: 3;
}

.dust-mote {
  position: absolute;
  bottom: -10px;
  background: rgba(255, 255, 255, 0.55);
  border-radius: 50%;
  filter: blur(0.3px);
  animation-name: dust-drift;
  animation-timing-function: ease-in-out;
  animation-iteration-count: infinite;
}

@keyframes dust-drift {
  0%   { transform: translateY(0) translateX(0); opacity: 0; }
  10%  { opacity: 0.5; }
  50%  { transform: translateY(-46vh) translateX(12px); }
  90%  { opacity: 0.15; }
  100% { transform: translateY(-92vh) translateX(-8px); opacity: 0; }
}

// ── Minimap ───────────────────────────────────────────────────────────────────
.adv-minimap {
  position: absolute;
  bottom: 14px;
  left: 14px;
  width: 170px;
  height: 110px;
  background: rgba(10, 10, 10, 0.78);
  border: 2px solid #000;
  box-shadow:
    inset 1px 1px 0 rgba(255,255,255,0.08),
    inset -1px -1px 0 rgba(0,0,0,0.6);
  z-index: 10;
  cursor: pointer;
  overflow: hidden;
}

.mm-dot {
  position: absolute;
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: #4a4a4a;
  transform: translate(-50%, -50%);

  &.state-claimed, &.state-claimable { background: #ffb300; box-shadow: 0 0 3px 1px rgba(255,179,0,0.7); }
  &.state-progress { background: #d9d9d9; }
}

.mm-viewport {
  position: absolute;
  border: 1px solid rgba(255, 255, 255, 0.75);
  background: rgba(255, 255, 255, 0.07);
  pointer-events: none;
}
</style>
