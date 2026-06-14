<template>
  <div class="crates-page">

    <!-- ── Header ──────────────────────────────────────────────────────────────── -->
    <div class="cp-header">
      <div class="cp-tabs">
        <button class="cp-tab" :class="{ active: tab === 'open' }" @click="tab = 'open'">Open Crates</button>
        <button class="cp-tab" :class="{ active: tab === 'forge' }" @click="tab = 'forge'">Forge</button>
      </div>
      <div class="cp-keys">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#FFB300" stroke-width="2">
          <circle cx="7" cy="17" r="3"/><path d="M10 17h4M14 14v6M17 14l3-3-3-3-3 3"/>
        </svg>
        <span class="cp-keys-count">{{ keys }} keys</span>
      </div>
    </div>

    <!-- ════════════════════════════════════════════════════════════════════════ -->
    <!-- ── OPEN tab ──────────────────────────────────────────────────────────── -->
    <!-- ════════════════════════════════════════════════════════════════════════ -->
    <template v-if="tab === 'open'">

      <!-- Crate selector -->
      <div class="cp-selector">
        <span v-if="!crateList.length" class="cp-no-crates">No crates available</span>
        <button
          v-for="c in crateList"
          :key="c.id"
          class="cp-crate-btn"
          :class="{ active: selectedCrate?.id === c.id }"
          @click="selectedCrate = c"
        >{{ c.name }}</button>
      </div>

      <!-- Spin strip -->
      <div class="cp-spin-wrap">
        <div class="cp-pointer" />
        <div ref="stripRef" class="cp-strip">
          <div
            v-for="(card, i) in spinCards"
            :key="i"
            class="cp-spin-card"
            :class="{ 'cp-spin-card--winner': spinDone && i === WINNER_IDX, 'cp-spin-card--active': spinning && !spinDone && i === activeCardIdx }"
            :style="spinCardStyle(card.rarity)"
          >
            <div class="cp-card-gem-wrap">
              <CosmeticModelViewer
                v-if="card.model_url"
                :model-url="card.model_url"
                :rarity="card.rarity"
                :auto-rotate="true"
                :show-label="false"
              />
              <template v-else>
                <img v-if="RARITY_ICONS[card.rarity]" :src="RARITY_ICONS[card.rarity]" class="cp-card-icon" :alt="RARITIES[card.rarity].label" />
                <svg v-else viewBox="0 0 24 24" :fill="RARITIES[card.rarity].color" opacity="0.8">
                  <polygon points="12,3 22,9 18,21 6,21 2,9" />
                  <polygon points="12,3 22,9 12,7" opacity="0.45" :fill="RARITIES[card.rarity].color" />
                </svg>
              </template>
            </div>
            <div class="cp-card-info">
              <span class="cp-card-name" :style="{ color: RARITIES[card.rarity].color }">{{ card.name ?? RARITIES[card.rarity].label }}</span>
              <img v-if="RARITY_ICONS[card.rarity]" :src="RARITY_ICONS[card.rarity]" class="cp-card-rarity-icon" />
            </div>
          </div>
        </div>
        <div class="cp-fade-left" />
        <div class="cp-fade-right" />
      </div>

      <!-- Open button -->
      <div class="cp-actions">
        <button class="cp-open-btn" :disabled="spinning || !selectedCrate" @click="openCrate">
          <span v-if="spinning && !spinDone" class="cp-spinner" />
          <template v-else>Open Crate</template>
        </button>
        <p v-if="!selectedCrate" class="cp-hint">Select a crate above to open</p>
        <p v-else-if="keys === 0" class="cp-hint" style="color:#c05050">No keys — purchase keys in the store</p>
      </div>

    </template>

    <!-- ════════════════════════════════════════════════════════════════════════ -->
    <!-- ── FORGE tab ─────────────────────────────────────────────────────────── -->
    <!-- ════════════════════════════════════════════════════════════════════════ -->
    <template v-else>

      <p class="fp-sub">Combine 3 duplicates to forge the next rarity tier</p>

      <div class="fp-rows">
        <div
          v-for="recipe in RECIPES"
          :key="recipe.from"
          class="fp-row"
          :class="{ 'fp-row--ready': canCraft(recipe.from) }"
        >
          <!-- 3 input slots -->
          <div class="fp-inputs">
            <div
              v-for="n in 3"
              :key="n"
              class="fp-slot"
              :class="{ 'fp-slot--filled': (inventory[recipe.from] ?? 0) >= n }"
              :style="slotStyle(recipe.from)"
            >
              <img v-if="RARITY_ICONS[recipe.from]" :src="RARITY_ICONS[recipe.from]" class="fp-slot-icon" :alt="RARITIES[recipe.from].label" />
              <svg v-else class="fp-slot-icon fp-slot-svg" viewBox="0 0 24 24" :fill="RARITIES[recipe.from].color" opacity="0.7">
                <polygon points="12,3 22,9 18,21 6,21 2,9" />
              </svg>
            </div>
            <span class="fp-count" :style="{ color: RARITIES[recipe.from].color }">{{ inventory[recipe.from] ?? 0 }} owned</span>
          </div>

          <!-- Arrow -->
          <div class="fp-arrow">
            <svg width="32" height="16" viewBox="0 0 32 16">
              <path d="M0 8 H26 M20 2 L28 8 L20 14" stroke="rgba(255,255,255,0.35)" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>

          <!-- Output slot -->
          <div class="fp-output">
            <div class="fp-slot fp-slot--output" :style="slotStyle(recipe.to)">
              <img v-if="RARITY_ICONS[recipe.to]" :src="RARITY_ICONS[recipe.to]" class="fp-slot-icon" :alt="RARITIES[recipe.to].label" />
              <svg v-else class="fp-slot-icon fp-slot-svg" viewBox="0 0 24 24" :fill="RARITIES[recipe.to].color" opacity="0.7">
                <polygon points="12,3 22,9 18,21 6,21 2,9" />
              </svg>
            </div>
            <span class="fp-out-label" :style="{ color: RARITIES[recipe.to].color }">{{ RARITIES[recipe.to].label }}</span>
          </div>

          <!-- Forge button -->
          <button
            class="fp-forge-btn"
            :disabled="!canCraft(recipe.from) || forging === recipe.from"
            :style="canCraft(recipe.from) ? forgeBtnStyle(recipe.from) : {}"
            @click="forge(recipe)"
          >
            <span v-if="forging === recipe.from" class="cp-spinner" />
            <template v-else>Forge</template>
          </button>
        </div>

        <div v-if="!inventoryLoaded" class="fp-loading"><span class="cp-spinner" /></div>
      </div>

    </template>

    <!-- ── Shared result overlay ───────────────────────────────────────────────── -->
    <Teleport to="body">
      <Transition name="result">
        <div v-if="showResult && activeResult" class="cp-result-overlay" @click.self="closeResult">
          <div class="cp-result-card" :style="resultCardStyle">
            <p v-if="tab === 'forge'" class="cp-result-title">Forged!</p>

            <div class="cp-result-viewer">
              <CosmeticModelViewer
                :model-url="activeResult.cosmetic.model_url"
                :rarity="activeResult.cosmetic.rarity"
                :show-label="false"
              />
            </div>

            <div class="cp-result-info">
              <span class="cp-result-rarity" :style="{ color: RARITIES[activeResult.cosmetic.rarity].color }">
                {{ RARITIES[activeResult.cosmetic.rarity].label }}
              </span>
              <span class="cp-result-name">{{ activeResult.cosmetic.name || '???' }}</span>
              <span v-if="activeResult.is_new" class="cp-result-new">NEW!</span>
            </div>

            <button class="cp-result-close" @click="closeResult">Continue</button>
          </div>
        </div>
      </Transition>
    </Teleport>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import CosmeticModelViewer from '../components/cosmetics/CosmeticModelViewer.vue'
import type { Rarity, CrateType, CrateResult, PlayerCosmetic } from '../types/cosmetics'
import { RARITIES } from '../types/cosmetics'
import { RARITY_ICONS } from '../assets/rarities/index'
import wingsUrl   from '../assets/cosmetics/wings.gltf?url'
import antlersUrl from '../assets/cosmetics/antlers.gltf?url'

const MOCK_MODELS: Record<string, string> = {
  wings:   wingsUrl,
  antlers: antlersUrl,
}

const MOCK_COSMETIC_POOL: Partial<Record<Rarity, Array<{ id: string; name: string; weight?: number }>>> = {
  leather:   [{ id: 'antlers', name: 'Antlers', weight: 1 }, { id: 'leather-acc-1', name: '???', weight: 5 }],
  iron:      [{ id: 'wings',           name: 'Wings', weight: 1 }, { id: 'iron-acc-1', name: '???', weight: 5 }],
  gold:      [{ id: 'gold-acc-1',      name: '???' }],
  diamond:   [{ id: 'diamond-acc-1',   name: '???' }],
  netherite: [{ id: 'netherite-acc-1', name: '???' }],
  enchanted: [{ id: 'enchanted-acc-1', name: '???' }],
}

function pickFromPool(pool: Array<{ id: string; weight?: number }>): { id: string } | undefined {
  if (!pool.length) return undefined
  const total = pool.reduce((s, e) => s + (e.weight ?? 1), 0)
  let r = Math.random() * total
  for (const entry of pool) { r -= (entry.weight ?? 1); if (r <= 0) return entry }
  return pool[pool.length - 1]
}

function injectModel(res: CrateResult): CrateResult {
  const url = res.cosmetic.model_url ?? MOCK_MODELS[res.cosmetic.id]
  if (url && !res.cosmetic.model_url) {
    return { ...res, cosmetic: { ...res.cosmetic, model_url: url } }
  }
  return res
}

function randomFillerCard(rarity: Rarity): { rarity: Rarity; model_url?: string; name: string } {
  const pick = pickFromPool(MOCK_COSMETIC_POOL[rarity] ?? []) as { id: string; name: string } | undefined
  return { rarity, model_url: pick ? MOCK_MODELS[pick.id] : undefined, name: pick?.name ?? '???' }
}

// ── Tabs ──────────────────────────────────────────────────────────────────────
const tab = ref<'open' | 'forge'>('open')

// ════════════════════════════════════════════════════════════════════════════
// CRATE OPEN
// ════════════════════════════════════════════════════════════════════════════
const CARD_W      = 140
const CARD_GAP    = 7
const CARD_STRIDE = CARD_W + CARD_GAP
const TOTAL_CARDS = 50
const WINNER_IDX  = 36
const SPIN_MS     = 5200

const STRIP_WEIGHTS: [Rarity, number][] = [
  ['leather',   50], ['iron', 26], ['gold', 13],
  ['diamond',    7], ['netherite', 3], ['enchanted', 1],
]

function weightedRarity(): Rarity {
  const total = STRIP_WEIGHTS.reduce((s, [, w]) => s + w, 0)
  let r = Math.random() * total
  for (const [rarity, w] of STRIP_WEIGHTS) { r -= w; if (r <= 0) return rarity }
  return 'leather'
}

const crateList     = ref<CrateType[]>([])
const selectedCrate = ref<CrateType | null>(null)
const keys          = ref(0)
const spinning      = ref(false)
const spinDone      = ref(false)
const crateResult   = ref<CrateResult | null>(null)
const stripRef      = ref<HTMLDivElement | null>(null)
const spinCards     = ref<{ rarity: Rarity; model_url?: string; name?: string }[]>([])
const activeCardIdx = ref<number>(-1)
let activeRaf = 0

function trackActiveCard() {
  if (!stripRef.value) return
  const container = stripRef.value.parentElement!
  const pointerX  = container.getBoundingClientRect().left + container.clientWidth / 2
  const stripX    = stripRef.value.getBoundingClientRect().left
  let bestIdx = -1, bestDist = Infinity
  for (let i = 0; i < TOTAL_CARDS; i++) {
    const dist = Math.abs(stripX + 16 + i * CARD_STRIDE + CARD_W / 2 - pointerX)
    if (dist < bestDist) { bestDist = dist; bestIdx = i }
  }
  activeCardIdx.value = bestIdx
  if (spinning.value && !spinDone.value) activeRaf = requestAnimationFrame(trackActiveCard)
}

function spinCardStyle(rarity: Rarity) {
  const r   = RARITIES[rarity]
  const hex = r.color.replace('#', '')
  const rr  = parseInt(hex.slice(0, 2), 16)
  const gg  = parseInt(hex.slice(2, 4), 16)
  const bb  = parseInt(hex.slice(4, 6), 16)
  return {
    '--cc-color': `rgba(${rr},${gg},${bb},0.65)`,
    '--cc-glow':  r.glow,
  }
}

async function openCrate() {
  if (spinning.value || !selectedCrate.value) return
  spinning.value = true
  spinDone.value = false
  crateResult.value = null

  let res: CrateResult
  try {
    res = await (window as any).api.crates.open(selectedCrate.value.id) as CrateResult
  } catch { spinning.value = false; return }

  res = injectModel(res)
  spinCards.value = Array.from({ length: TOTAL_CARDS }, (_, i) =>
    i === WINNER_IDX
      ? { rarity: res.cosmetic.rarity, model_url: res.cosmetic.model_url, name: res.cosmetic.name }
      : randomFillerCard(weightedRarity())
  )

  if (stripRef.value) {
    stripRef.value.style.transition = 'none'
    stripRef.value.style.transform  = 'translateX(0)'
  }
  await new Promise(r => requestAnimationFrame(r))
  await new Promise(r => requestAnimationFrame(r))

  if (stripRef.value) {
    const cw     = stripRef.value.parentElement?.clientWidth ?? 700
    const center = WINNER_IDX * CARD_STRIDE + CARD_W / 2
    const offset = center - cw / 2 + (Math.random() - 0.5) * 30
    stripRef.value.style.transition = `transform ${SPIN_MS}ms cubic-bezier(0.05, 0.4, 0.1, 1.0)`
    stripRef.value.style.transform  = `translateX(-${offset}px)`
    activeRaf = requestAnimationFrame(trackActiveCard)
  }

  await new Promise(r => setTimeout(r, SPIN_MS + 200))
  cancelAnimationFrame(activeRaf)
  activeCardIdx.value = -1
  crateResult.value = res
  spinDone.value    = true
  spinning.value    = false
  showResult.value  = true
  activeResult.value = res
}

function resetCrate() {
  cancelAnimationFrame(activeRaf)
  activeCardIdx.value = -1
  spinDone.value = false
  crateResult.value = null
  spinCards.value = Array.from({ length: TOTAL_CARDS }, () => randomFillerCard(weightedRarity()))
  if (stripRef.value) { stripRef.value.style.transition = 'none'; stripRef.value.style.transform = 'translateX(0)' }
}

// ════════════════════════════════════════════════════════════════════════════
// FORGE
// ════════════════════════════════════════════════════════════════════════════
const CRAFT_COST = 3
const RECIPES: { from: Rarity; to: Rarity }[] = [
  { from: 'leather', to: 'iron'      },
  { from: 'iron',    to: 'gold'      },
  { from: 'gold',    to: 'diamond'   },
  { from: 'diamond', to: 'netherite' },
]

const inventory       = ref<Partial<Record<Rarity, number>>>({})
const inventoryLoaded = ref(false)
const forging         = ref<Rarity | null>(null)

function canCraft(rarity: Rarity) { return (inventory.value[rarity] ?? 0) >= CRAFT_COST }

function slotStyle(rarity: Rarity) {
  const r = RARITIES[rarity]
  return { background: r.bg, borderColor: r.color + '55' }
}

function forgeBtnStyle(rarity: Rarity) {
  const r = RARITIES[rarity]
  return { borderColor: r.color + '90', boxShadow: `0 0 14px ${r.glow}`, color: r.color }
}

async function forge(recipe: { from: Rarity; to: Rarity }) {
  if (!canCraft(recipe.from) || forging.value) return
  forging.value = recipe.from
  try {
    const raw = await (window as any).api.crafting.combine(recipe.from) as CrateResult
    const res = injectModel(raw)
    inventory.value[recipe.from] = (inventory.value[recipe.from] ?? 0) - CRAFT_COST
    inventory.value[recipe.to]   = (inventory.value[recipe.to]   ?? 0) + 1
    activeResult.value = res
    showResult.value   = true
  } catch { /* API not ready */ }
  finally { forging.value = null }
}

async function loadInventory() {
  try {
    const inv = await (window as any).api.crafting.inventory() as PlayerCosmetic[]
    const counts: Partial<Record<Rarity, number>> = {}
    for (const item of (inv ?? [])) counts[item.rarity] = (counts[item.rarity] ?? 0) + 1
    inventory.value = counts
  } catch { inventory.value = {} }
  finally { inventoryLoaded.value = true }
}

watch(tab, t => { if (t === 'forge' && !inventoryLoaded.value) loadInventory() })

// ════════════════════════════════════════════════════════════════════════════
// SHARED RESULT
// ════════════════════════════════════════════════════════════════════════════
const showResult   = ref(false)
const activeResult = ref<CrateResult | null>(null)

const resultCardStyle = computed(() => {
  if (!activeResult.value) return {}
  const r = RARITIES[activeResult.value.cosmetic.rarity]
  return { boxShadow: `0 0 60px ${r.glow}, 0 8px 32px rgba(0,0,0,0.8)`, borderColor: r.color + '80' }
})

function closeResult() {
  showResult.value = false
  if (tab.value === 'open') resetCrate()
}

onUnmounted(() => { cancelAnimationFrame(activeRaf) })

// ── Init ──────────────────────────────────────────────────────────────────────
onMounted(async () => {
  try {
    const [crateRes, keyRes] = await Promise.all([
      (window as any).api.crates.list() as Promise<CrateType[]>,
      (window as any).api.crates.keys() as Promise<{ count: number }>,
    ])
    crateList.value = crateRes ?? []
    keys.value      = keyRes?.count ?? 0
    if (crateList.value.length) selectedCrate.value = crateList.value[0]
  } catch { /* API not ready */ }
  spinCards.value = Array.from({ length: TOTAL_CARDS }, () => randomFillerCard(weightedRarity()))
})
</script>

<style lang="scss" scoped>
@font-face {
  font-family: 'Mojangles';
  src: url('../assets/fonts/mojangles.ttf') format('truetype');
}

.crates-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px 22px;
  gap: 14px;
  overflow: hidden;
  background: #0a0b0d;
}

// ── Header / tabs ─────────────────────────────────────────────────────────────
.cp-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}

.cp-tabs {
  display: flex;
  gap: 4px;
}

.cp-tab {
  padding: 7px 18px;
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  background: #0e0e11;
  border: 1px solid #222;
  color: #555;
  cursor: pointer;
  letter-spacing: 0.05em;
  transition: background 80ms, color 80ms, border-color 80ms;

  &:hover { color: #999; border-color: #444; }
  &.active { background: #141417; color: #eee; border-color: #666; box-shadow: inset 0 -2px 0 #666; }
}

.cp-keys {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(255,179,0,0.08);
  border: 1px solid rgba(255,179,0,0.22);
  padding: 5px 11px;
}

.cp-keys-count {
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  color: #FFB300;
  letter-spacing: 0.05em;
}

// ── Crate selector ────────────────────────────────────────────────────────────
.cp-selector {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
  flex-wrap: wrap;
}

.cp-no-crates {
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  color: #444;
  letter-spacing: 0.04em;
}

.cp-crate-btn {
  padding: 7px 16px;
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  background: #111;
  border: 1px solid #2a2a2a;
  color: #777;
  cursor: pointer;
  letter-spacing: 0.04em;
  transition: background 80ms, border-color 80ms, color 80ms;
  &:hover { background: #181818; color: #bbb; border-color: #555; }
  &.active { background: #161616; color: #eee; border-color: #777; box-shadow: inset 0 -2px 0 #777; }
}

// ── Spin strip ────────────────────────────────────────────────────────────────
@property --cc-angle {
  syntax: '<angle>';
  initial-value: 45deg;
  inherits: false;
}

.cp-spin-wrap {
  position: relative;
  height: 225px;
  overflow: hidden;
  background: #0d0e10;
  border: 1px solid #1e1e22;
  flex-shrink: 0;
}

.cp-pointer {
  position: absolute;
  left: 50%;
  top: 0;
  bottom: 0;
  width: 2px;
  background: rgba(255,255,255,0.7);
  transform: translateX(-50%);
  z-index: 10;
  box-shadow: 0 0 8px rgba(255,255,255,0.4);
  pointer-events: none;
  &::before, &::after {
    content: '';
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    border: 6px solid transparent;
  }
  &::before { top: 0; border-top-color: rgba(255,255,255,0.8); border-bottom: none; }
  &::after  { bottom: 0; border-bottom-color: rgba(255,255,255,0.8); border-top: none; }
}

.cp-strip {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 0 16px;
  height: 100%;
  will-change: transform;
}

.cp-spin-card {
  width: 140px;
  height: 180px;
  flex-shrink: 0;
  border: 1px solid transparent;
  background-image:
    linear-gradient(#0f0f12, #0f0f12),
    conic-gradient(
      from var(--cc-angle),
      rgba(0,0,0,0) 0%,
      var(--cc-color) 18%,
      rgba(0,0,0,0) 36%,
      rgba(0,0,0,0) 100%
    );
  background-origin: border-box;
  background-clip: padding-box, border-box;
  transition:
    --cc-angle 300ms cubic-bezier(.2,0,0,1),
    transform   90ms  cubic-bezier(.2,0,0,1),
    box-shadow  90ms  cubic-bezier(.2,0,0,1);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 7px;
  cursor: default;

  &:hover {
    --cc-angle: 135deg;
    transform: perspective(600px) rotateY(-6deg) rotateX(3deg) translateZ(5px) scale(1.03);
    box-shadow: 6px 10px 28px rgba(0,0,0,0.75), 0 0 14px var(--cc-glow);
  }

  &--active {
    --cc-angle: 90deg;
    transform: perspective(450px) rotateY(-7deg) rotateX(5deg) scale(1.08);
    box-shadow: 0 0 32px var(--cc-glow), 0 14px 36px rgba(0,0,0,0.85);
    z-index: 2;
  }

  &--winner {
    --cc-angle: 135deg;
    transform: perspective(600px) rotateY(-4deg) rotateX(2deg) translateZ(3px) scale(1.04);
    box-shadow: 4px 8px 20px rgba(0,0,0,0.6), 0 0 18px var(--cc-glow);
  }
}

.cp-card-gem-wrap {
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  svg { width: 100%; height: 100%; }
}

.cp-card-icon {
  width: 100%;
  height: 100%;
  object-fit: contain;
  image-rendering: pixelated;
  filter: drop-shadow(0 2px 6px rgba(0,0,0,0.7));
}

.cp-card-info {
  display: flex;
  align-items: center;
  gap: 4px;
}

.cp-card-name {
  font-family: 'Mojangles', monospace;
  font-size: 7px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 72px;
}

.cp-card-rarity-icon {
  width: 14px;
  height: 14px;
  object-fit: contain;
  image-rendering: pixelated;
  flex-shrink: 0;
  filter: drop-shadow(0 1px 3px rgba(0,0,0,0.6));
}

.cp-fade-left, .cp-fade-right {
  position: absolute;
  top: 0; bottom: 0;
  width: 100px;
  pointer-events: none;
  z-index: 5;
}
.cp-fade-left  { left:  0; background: linear-gradient(to right, #0d0e10, transparent); }
.cp-fade-right { right: 0; background: linear-gradient(to left,  #0d0e10, transparent); }

// ── Open actions ──────────────────────────────────────────────────────────────
.cp-actions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.cp-open-btn {
  padding: 11px 40px;
  font-family: 'Mojangles', monospace;
  font-size: 12px;
  background: #1a1a1a;
  border: 1px solid #555;
  color: #eee;
  cursor: pointer;
  letter-spacing: 0.06em;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: background 100ms, border-color 100ms, box-shadow 100ms;
  &:hover:not(:disabled) { background: #222; border-color: #aaa; box-shadow: 0 0 16px rgba(255,255,255,0.08); }
  &:disabled { opacity: 0.35; cursor: not-allowed; }
}

.cp-hint {
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  color: #444;
  letter-spacing: 0.04em;
  margin: 0;
}

// ── Forge tab ─────────────────────────────────────────────────────────────────
.fp-sub {
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  color: #3a3a3a;
  margin: 0;
  letter-spacing: 0.05em;
  flex-shrink: 0;
}

.fp-rows {
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
  scrollbar-width: thin;
  scrollbar-color: #222 transparent;
}

.fp-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  background: #0e0e10;
  border: 1px solid #1a1a1e;
  flex-shrink: 0;
  transition: border-color 200ms;
  &--ready { border-color: #2a2a30; }
}

.fp-inputs {
  display: flex;
  align-items: center;
  gap: 5px;
  position: relative;
}

.fp-slot {
  width: 50px;
  height: 50px;
  border: 1px solid transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.3;
  transition: opacity 200ms;
  &--filled  { opacity: 1; }
  &--output  { opacity: 0.65; width: 56px; height: 56px; }
}
.fp-row--ready .fp-slot--output { opacity: 1; }

.fp-slot-icon {
  width: 68%;
  height: 68%;
  object-fit: contain;
  image-rendering: pixelated;
  filter: drop-shadow(0 1px 4px rgba(0,0,0,0.7));
}
.fp-slot-svg { width: 55%; height: 55%; }

.fp-count {
  font-family: 'Mojangles', monospace;
  font-size: 7px;
  letter-spacing: 0.05em;
  position: absolute;
  bottom: -14px;
  left: 50%;
  transform: translateX(-50%);
  white-space: nowrap;
  opacity: 0.8;
}

.fp-arrow {
  flex-shrink: 0;
  opacity: 0.35;
  .fp-row--ready & { opacity: 0.65; }
}

.fp-output {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.fp-out-label {
  font-family: 'Mojangles', monospace;
  font-size: 7px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  opacity: 0.8;
}

.fp-forge-btn {
  margin-left: auto;
  padding: 8px 20px;
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  background: #111;
  border: 1px solid #222;
  color: #333;
  cursor: not-allowed;
  letter-spacing: 0.06em;
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  transition: background 100ms, border-color 150ms, box-shadow 200ms, color 200ms;
  &:not(:disabled) { cursor: pointer; &:hover { background: #181818; } }
  &:disabled { opacity: 0.4; }
}

.fp-loading {
  display: flex;
  justify-content: center;
  padding: 20px;
}

// ── Shared spinner ────────────────────────────────────────────────────────────
.cp-spinner {
  width: 12px;
  height: 12px;
  border: 2px solid rgba(255,255,255,0.15);
  border-top-color: rgba(255,255,255,0.6);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  display: inline-block;
  flex-shrink: 0;
}

// ── Shared result overlay ─────────────────────────────────────────────────────
.cp-result-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 5000;
  backdrop-filter: blur(6px);
}

.cp-result-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  background: #111214;
  border: 1px solid rgba(255,255,255,0.12);
  padding: 28px 38px;
  min-width: 280px;
}

.cp-result-title {
  font-family: 'Mojangles', monospace;
  font-size: 13px;
  color: #f0f0f0;
  margin: 0;
  letter-spacing: 0.06em;
}

.cp-result-viewer { width: 220px; height: 220px; position: relative; }

.cp-result-info { display: flex; flex-direction: column; align-items: center; gap: 3px; }

.cp-result-rarity {
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  opacity: 0.9;
}

.cp-result-name {
  font-family: 'Mojangles', monospace;
  font-size: 17px;
  color: #f0f0f0;
  letter-spacing: 0.03em;
}

.cp-result-new {
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  color: #4AFFE0;
  letter-spacing: 0.1em;
  animation: pulse 1.2s ease-in-out infinite;
}

.cp-result-close {
  margin-top: 4px;
  padding: 8px 26px;
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  background: #1a1a1a;
  border: 1px solid #555;
  color: #ccc;
  cursor: pointer;
  letter-spacing: 0.05em;
  transition: background 80ms, border-color 80ms;
  &:hover { background: #222; border-color: #999; }
}

@keyframes spin  { to { transform: rotate(360deg); } }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }

.result-enter-active { transition: opacity 220ms ease, transform 220ms ease; }
.result-leave-active { transition: opacity 130ms ease; }
.result-enter-from   { opacity: 0; transform: scale(0.95); }
.result-leave-to     { opacity: 0; }
</style>
