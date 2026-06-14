<template>
  <div class="pass-page">

    <!-- ── Header ──────────────────────────────────────────────────────────────── -->
    <div class="pass-header">
      <h1 class="pass-title">Client Pass</h1>
      <span class="pass-season">Season 1</span>
    </div>

    <!-- ── Tier section ────────────────────────────────────────────────────────── -->
    <div class="tier-section">

      <div v-if="loading" class="tier-loading">
        <span class="spinner" />
      </div>

      <template v-else>
        <div class="tier-row">
          <div
            v-for="tier in currentTiers"
            :key="tier.tier"
            class="tier-card"
            :class="{ unlocked: isUnlocked(tier) }"
          >
            <div
              class="card-body"
              :style="{
                background:  getCardBg(tier),
                borderColor: getCardBorder(tier),
                boxShadow:   isUnlocked(tier) ? getCardGlow(tier) : 'none',
              }"
            >
              <!-- Rarity icon (MC texture or gem SVG fallback) -->
              <img
                v-if="RARITY_ICONS[tier.cosmetic.rarity as Rarity]"
                class="card-gem card-gem--img"
                :src="RARITY_ICONS[tier.cosmetic.rarity as Rarity]"
                :alt="tier.cosmetic.rarity"
              />
              <svg
                v-else
                class="card-gem"
                viewBox="0 0 24 24"
                :fill="getRarityInfo(tier.cosmetic.rarity).color"
                opacity="0.55"
              >
                <polygon points="12,3 22,9 18,21 6,21 2,9" />
                <polygon points="12,3 22,9 12,7" opacity="0.4" :fill="getRarityInfo(tier.cosmetic.rarity).color" />
              </svg>
              <svg v-if="isUnlocked(tier)" class="card-check" width="16" height="16" viewBox="0 0 10 10">
                <polyline points="1.5,5 4,7.5 8.5,2" stroke="rgba(255,255,255,0.9)" stroke-width="1.8" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <div class="card-label">
              <span class="card-rarity" :style="{ color: getRarityInfo(tier.cosmetic.rarity).color }">
                {{ getRarityInfo(tier.cosmetic.rarity).label }}
              </span>
              <span class="card-xp">{{ formatXp(tier.xp_required) }}</span>
            </div>
          </div>
        </div>

        <p class="page-label">Page {{ currentPage }}/{{ totalPages }}</p>

        <div class="pagination">
          <button class="arrow-btn" :disabled="currentPage === 1" @click="prevPage">
            <img :src="arrowImg" class="arrow arrow-flip" alt="" />
          </button>
          <div class="dots">
            <span
              v-for="n in totalPages"
              :key="n"
              class="dot"
              :class="{ active: n === currentPage }"
              @click="currentPage = n"
            />
          </div>
          <button class="arrow-btn" :disabled="currentPage === totalPages" @click="nextPage">
            <img :src="arrowImg" class="arrow" alt="" />
          </button>
        </div>
      </template>

    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated } from 'vue'
import arrowImg from '../assets/arrow-circle.png'
import type { Rarity } from '../types/cosmetics'
import { RARITIES } from '../types/cosmetics'
import { RARITY_ICONS } from '../assets/rarities/index'

interface PassTier {
  tier: number
  xp_required: number
  cosmetic: { id: string; name: string; type: string; rarity: string; model_url?: string }
}

interface PassProgress {
  xp: number
  current_tier: number
  unlocked_cosmetics: string[]
  daily_available: boolean
}

const TIERS_PER_PAGE = 8

const tiers       = ref<PassTier[]>([])
const progress    = ref<PassProgress | null>(null)
const loading     = ref(true)
const currentPage = ref(1)

const totalPages = computed(() => Math.ceil(tiers.value.length / TIERS_PER_PAGE) || 5)

const currentTiers = computed(() => {
  const start = (currentPage.value - 1) * TIERS_PER_PAGE
  return tiers.value.slice(start, start + TIERS_PER_PAGE)
})

function getRarityInfo(rarity: string) {
  return RARITIES[rarity as Rarity] ?? RARITIES.leather
}

function getCardBg(tier: PassTier): string {
  return getRarityInfo(tier.cosmetic.rarity).bg
}

function getCardBorder(tier: PassTier): string {
  return getRarityInfo(tier.cosmetic.rarity).color + '70'
}

function getCardGlow(tier: PassTier): string {
  const info = getRarityInfo(tier.cosmetic.rarity)
  return `0 0 10px ${info.glow}`
}

function isUnlocked(tier: PassTier): boolean {
  return progress.value ? tier.xp_required <= progress.value.xp : false
}

function formatXp(xp: number): string {
  return xp.toLocaleString() + ' XP'
}

function prevPage() { if (currentPage.value > 1) currentPage.value-- }
function nextPage() { if (currentPage.value < totalPages.value) currentPage.value++ }

async function load() {
  loading.value = true
  try {
    const [passRes, progressRes] = await Promise.all([
      window.api.pass.get() as Promise<{ tiers: PassTier[] }>,
      window.api.pass.progress() as Promise<PassProgress | null>,
    ])
    tiers.value    = (passRes as any)?.tiers ?? []
    progress.value = progressRes ?? null
    if (progress.value && progress.value.current_tier > 0) {
      currentPage.value = Math.ceil(progress.value.current_tier / TIERS_PER_PAGE)
    }
  } finally {
    loading.value = false
  }
}

onMounted(load)
onActivated(load)
</script>

<style lang="scss" scoped>
.pass-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
  background-image: url('../assets/pass-bg.png');
  background-size: cover;
  background-position: center;
}

// ── Header ────────────────────────────────────────────────────────────────────
.pass-header {
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
}

.pass-title {
  font-family: 'Mojangles', monospace;
  font-size: 28px;
  font-weight: normal;
  color: #fff;
  margin: 0;
  letter-spacing: 0.02em;
}

.pass-season {
  font-family: 'Mojangles', monospace;
  font-size: 14px;
  color: #fff;
  opacity: 0.75;
  letter-spacing: 0.04em;
}

// ── Tier section ──────────────────────────────────────────────────────────────
.tier-section {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0;
  padding-bottom: 22px;
  flex-shrink: 0;
}

.tier-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 120px;
}

// ── Tier row ──────────────────────────────────────────────────────────────────
.tier-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin-bottom: 10px;

  // visual gap between group of 4 and group of 4
  .tier-card:nth-child(4) {
    margin-right: 28px;
  }
}

// ── Tier card ─────────────────────────────────────────────────────────────────
.tier-card {
  display: flex;
  flex-direction: column;
  width: 70px;
  flex-shrink: 0;
}

.card-body {
  width: 70px;
  height: 72px;
  border: 1px solid transparent;
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
  padding: 5px;
  position: relative;
  flex-shrink: 0;
  transition: box-shadow 300ms;
}

.card-gem {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -56%);
  width: 30px;
  height: 30px;
  pointer-events: none;

  &--img {
    image-rendering: pixelated;
    object-fit: contain;
    filter: drop-shadow(0 1px 4px rgba(0,0,0,0.7));
  }
}

.card-check {
  display: block;
  position: relative;
  z-index: 1;
}

.card-label {
  width: 70px;
  background: rgba(20, 20, 22, 0.85);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3px 2px;
  flex-shrink: 0;
  gap: 1px;
}

.card-rarity {
  font-family: 'Mojangles', monospace;
  font-size: 6px;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  white-space: nowrap;
  opacity: 0.9;
}

.card-xp {
  font-family: 'Mojangles', monospace;
  font-size: 6px;
  color: rgba(255,255,255,0.5);
  letter-spacing: 0.05em;
  white-space: nowrap;
}

// ── Page label ────────────────────────────────────────────────────────────────
.page-label {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  color: rgba(255, 255, 255, 0.6);
  letter-spacing: 0.08em;
  margin: 0 0 8px;
}

// ── Pagination ────────────────────────────────────────────────────────────────
.pagination {
  display: flex;
  align-items: center;
  gap: 10px;
}

.arrow-btn {
  background: transparent;
  border: none;
  padding: 0;
  cursor: pointer;
  opacity: 1;
  transition: opacity 150ms;
  display: flex;
  align-items: center;

  &:disabled { opacity: 0.25; cursor: default; }
  &:hover:not(:disabled) { opacity: 0.75; }
}

.arrow {
  width: 20px;
  height: 20px;
  display: block;

  &.arrow-flip { transform: scaleX(-1); }
}

.dots {
  display: flex;
  align-items: center;
  gap: 6px;
}

.dot {
  width: 10px;
  height: 10px;
  background: #8d8989;
  cursor: pointer;
  flex-shrink: 0;
  transition: background 150ms;

  &.active { background: #d9d9d9; }
  &:hover:not(.active) { background: #aaa; }
}

// ── Spinner ───────────────────────────────────────────────────────────────────
.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid rgba(255,255,255,0.15);
  border-top-color: rgba(255,255,255,0.6);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
