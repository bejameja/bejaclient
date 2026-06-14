<template>
  <div class="forge-page">

    <!-- ── Header ──────────────────────────────────────────────────────────────── -->
    <div class="fp-header">
      <h1 class="fp-title">Forge</h1>
      <p class="fp-sub">Combine 3 duplicates to forge the next rarity tier</p>
    </div>

    <!-- ── Craft rows ──────────────────────────────────────────────────────────── -->
    <div class="fp-rows">
      <div
        v-for="recipe in RECIPES"
        :key="recipe.from"
        class="fp-row"
        :class="{ 'fp-row--ready': canCraft(recipe.from) }"
      >
        <!-- Input: 3× rarity -->
        <div class="fp-inputs">
          <div
            v-for="n in 3"
            :key="n"
            class="fp-slot"
            :class="{ 'fp-slot--filled': (inventory[recipe.from] ?? 0) >= n }"
            :style="slotStyle(recipe.from)"
          >
            <img
              v-if="RARITY_ICONS[recipe.from]"
              :src="RARITY_ICONS[recipe.from]"
              class="fp-slot-icon"
              :alt="RARITIES[recipe.from].label"
            />
            <svg v-else class="fp-slot-icon fp-slot-svg" viewBox="0 0 24 24" :fill="RARITIES[recipe.from].color" opacity="0.7">
              <polygon points="12,3 22,9 18,21 6,21 2,9" />
            </svg>
          </div>
          <span class="fp-count" :style="{ color: RARITIES[recipe.from].color }">
            {{ inventory[recipe.from] ?? 0 }} owned
          </span>
        </div>

        <!-- Arrow -->
        <div class="fp-arrow">
          <svg width="32" height="16" viewBox="0 0 32 16">
            <path d="M0 8 H26 M20 2 L28 8 L20 14" stroke="rgba(255,255,255,0.35)" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>

        <!-- Output: 1× next rarity -->
        <div class="fp-output">
          <div class="fp-slot fp-slot--output" :style="slotStyle(recipe.to)">
            <img
              v-if="RARITY_ICONS[recipe.to]"
              :src="RARITY_ICONS[recipe.to]"
              class="fp-slot-icon"
              :alt="RARITIES[recipe.to].label"
            />
            <svg v-else class="fp-slot-icon fp-slot-svg" viewBox="0 0 24 24" :fill="RARITIES[recipe.to].color" opacity="0.7">
              <polygon points="12,3 22,9 18,21 6,21 2,9" />
            </svg>
          </div>
          <span class="fp-out-label" :style="{ color: RARITIES[recipe.to].color }">
            {{ RARITIES[recipe.to].label }}
          </span>
        </div>

        <!-- Forge button -->
        <button
          class="fp-btn"
          :disabled="!canCraft(recipe.from) || forging === recipe.from"
          :style="canCraft(recipe.from) ? btnActiveStyle(recipe.from) : {}"
          @click="forge(recipe)"
        >
          <span v-if="forging === recipe.from" class="fp-spinner" />
          <template v-else>Forge</template>
        </button>
      </div>

      <div v-if="!inventoryLoaded" class="fp-loading">
        <span class="fp-spinner" />
      </div>
    </div>

    <!-- ── Result overlay ──────────────────────────────────────────────────────── -->
    <Teleport to="body">
      <Transition name="result">
        <div v-if="result" class="fp-result-overlay" @click.self="result = null">
          <div class="fp-result-card" :style="resultCardStyle">
            <p class="fp-result-title">Forged!</p>

            <div class="fp-result-viewer">
              <CosmeticModelViewer
                :model-url="result.cosmetic.model_url"
                :rarity="result.cosmetic.rarity"
                :show-label="false"
              />
            </div>

            <div class="fp-result-info">
              <span class="fp-result-rarity" :style="{ color: RARITIES[result.cosmetic.rarity].color }">
                {{ RARITIES[result.cosmetic.rarity].label }}
              </span>
              <span class="fp-result-name">{{ result.cosmetic.name || '???' }}</span>
            </div>

            <button class="fp-result-close" @click="result = null">Continue</button>
          </div>
        </div>
      </Transition>
    </Teleport>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import CosmeticModelViewer from '../components/cosmetics/CosmeticModelViewer.vue'
import type { Rarity, PlayerCosmetic, CrateResult } from '../types/cosmetics'
import { RARITIES } from '../types/cosmetics'
import { RARITY_ICONS } from '../assets/rarities/index'

// ── Recipes ───────────────────────────────────────────────────────────────────
const CRAFT_COST = 3
const RECIPES: { from: Rarity; to: Rarity }[] = [
  { from: 'leather',   to: 'iron'      },
  { from: 'iron',      to: 'gold'      },
  { from: 'gold',      to: 'diamond'   },
  { from: 'diamond',   to: 'netherite' },
]

// ── State ─────────────────────────────────────────────────────────────────────
const inventory      = ref<Partial<Record<Rarity, number>>>({})
const inventoryLoaded = ref(false)
const forging        = ref<Rarity | null>(null)
const result         = ref<CrateResult | null>(null)

// ── Computed ──────────────────────────────────────────────────────────────────
function canCraft(rarity: Rarity): boolean {
  return (inventory.value[rarity] ?? 0) >= CRAFT_COST
}

function slotStyle(rarity: Rarity) {
  const r = RARITIES[rarity]
  return {
    background:  r.bg,
    borderColor: r.color + '55',
  }
}

function btnActiveStyle(rarity: Rarity) {
  const r = RARITIES[rarity]
  return {
    borderColor: r.color + '90',
    boxShadow:   `0 0 14px ${r.glow}`,
    color:       r.color,
  }
}

const resultCardStyle = computed(() => {
  if (!result.value) return {}
  const r = RARITIES[result.value.cosmetic.rarity]
  return {
    boxShadow:   `0 0 60px ${r.glow}, 0 8px 32px rgba(0,0,0,0.8)`,
    borderColor: r.color + '80',
  }
})

// ── Actions ───────────────────────────────────────────────────────────────────
async function forge(recipe: { from: Rarity; to: Rarity }) {
  if (!canCraft(recipe.from) || forging.value) return
  forging.value = recipe.from

  try {
    const res = await (window as any).api.crafting.combine(recipe.from) as CrateResult
    inventory.value[recipe.from] = (inventory.value[recipe.from] ?? 0) - CRAFT_COST
    const existing = inventory.value[recipe.to] ?? 0
    inventory.value[recipe.to] = existing + 1
    result.value = res
  } catch {
    /* API not ready */
  } finally {
    forging.value = null
  }
}

// ── Load inventory ────────────────────────────────────────────────────────────
async function load() {
  try {
    const inv = await (window as any).api.crafting.inventory() as PlayerCosmetic[]
    const counts: Partial<Record<Rarity, number>> = {}
    for (const item of (inv ?? [])) {
      counts[item.rarity] = (counts[item.rarity] ?? 0) + 1
    }
    inventory.value = counts
  } catch {
    inventory.value = {}
  } finally {
    inventoryLoaded.value = true
  }
}

onMounted(load)
</script>

<style lang="scss" scoped>
@font-face {
  font-family: 'Mojangles';
  src: url('../assets/fonts/mojangles.ttf') format('truetype');
}

.forge-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 22px 28px;
  gap: 20px;
  overflow-y: auto;
  background: #09090b;
  scrollbar-width: thin;
  scrollbar-color: #222 transparent;
}

// ── Header ────────────────────────────────────────────────────────────────────
.fp-header { flex-shrink: 0; }

.fp-title {
  font-family: 'Mojangles', monospace;
  font-size: 26px;
  font-weight: normal;
  color: #f0f0f0;
  margin: 0 0 4px;
  letter-spacing: 0.02em;
}

.fp-sub {
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  color: #444;
  margin: 0;
  letter-spacing: 0.05em;
}

// ── Rows ─────────────────────────────────────────────────────────────────────
.fp-rows {
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex-shrink: 0;
}

.fp-row {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 14px 18px;
  background: #0e0e10;
  border: 1px solid #1a1a1e;
  transition: border-color 200ms, background 200ms;

  &--ready {
    background: #0f0f12;
    border-color: #2a2a30;
  }
}

// ── Slots ─────────────────────────────────────────────────────────────────────
.fp-inputs {
  display: flex;
  align-items: center;
  gap: 6px;
  position: relative;
}

.fp-slot {
  width: 54px;
  height: 54px;
  border: 1px solid transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.35;
  transition: opacity 200ms, box-shadow 200ms;

  &--filled { opacity: 1; }

  &--output {
    opacity: 0.7;
    width: 60px;
    height: 60px;
  }
}

.fp-row--ready .fp-slot--output { opacity: 1; }

.fp-slot-icon {
  width: 70%;
  height: 70%;
  object-fit: contain;
  image-rendering: pixelated;
  filter: drop-shadow(0 2px 5px rgba(0,0,0,0.7));
}

.fp-slot-svg { width: 55%; height: 55%; }

.fp-count {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  letter-spacing: 0.05em;
  position: absolute;
  bottom: -16px;
  left: 50%;
  transform: translateX(-50%);
  white-space: nowrap;
  opacity: 0.8;
}

// ── Arrow ─────────────────────────────────────────────────────────────────────
.fp-arrow {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  opacity: 0.4;
  .fp-row--ready & { opacity: 0.7; }
}

// ── Output ────────────────────────────────────────────────────────────────────
.fp-output {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 5px;
}

.fp-out-label {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  opacity: 0.8;
}

// ── Forge button ──────────────────────────────────────────────────────────────
.fp-btn {
  margin-left: auto;
  padding: 9px 24px;
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  background: #111;
  border: 1px solid #2a2a2a;
  color: #444;
  cursor: not-allowed;
  letter-spacing: 0.06em;
  display: flex;
  align-items: center;
  gap: 7px;
  transition: background 100ms, border-color 100ms, box-shadow 200ms, color 200ms;
  flex-shrink: 0;

  &:not(:disabled) {
    color: #ddd;
    cursor: pointer;
    &:hover { background: #181818; }
  }
  &:disabled { opacity: 0.4; }
}

// ── Loading ───────────────────────────────────────────────────────────────────
.fp-loading {
  display: flex;
  justify-content: center;
  padding: 24px;
}

.fp-spinner {
  width: 13px;
  height: 13px;
  border: 2px solid rgba(255,255,255,0.15);
  border-top-color: rgba(255,255,255,0.6);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  display: inline-block;
}
@keyframes spin { to { transform: rotate(360deg); } }

// ── Result overlay ────────────────────────────────────────────────────────────
.fp-result-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 5000;
  backdrop-filter: blur(6px);
}

.fp-result-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  background: #111214;
  border: 1px solid rgba(255,255,255,0.12);
  padding: 30px 38px;
  min-width: 280px;
}

.fp-result-title {
  font-family: 'Mojangles', monospace;
  font-size: 14px;
  color: #f0f0f0;
  margin: 0;
  letter-spacing: 0.06em;
}

.fp-result-viewer {
  width: 220px;
  height: 220px;
  position: relative;
}

.fp-result-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.fp-result-rarity {
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.fp-result-name {
  font-family: 'Mojangles', monospace;
  font-size: 16px;
  color: #f0f0f0;
  letter-spacing: 0.03em;
}

.fp-result-close {
  margin-top: 6px;
  padding: 8px 28px;
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  background: #1a1a1a;
  border: 1px solid #555;
  color: #ccc;
  cursor: pointer;
  letter-spacing: 0.05em;
  transition: background 80ms, border-color 80ms;
  &:hover { background: #222; border-color: #999; }
}

.result-enter-active { transition: opacity 220ms ease, transform 220ms ease; }
.result-leave-active { transition: opacity 130ms ease; }
.result-enter-from   { opacity: 0; transform: scale(0.95); }
.result-leave-to     { opacity: 0; }
</style>
