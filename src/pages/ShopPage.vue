<template>
  <div class="shop-page">

    <WipPage
      v-if="MAINTENANCE"
      label="Shop under maintenance"
      sub="We're working at full speed to bring it back. Sorry for the inconvenience!"
    />

    <template v-else>
    <!-- Cape Shop -->
    <section class="shop-section">
      <h2 class="shop-section-title">Cape Shop</h2>
      <p class="shop-section-sub">Shop-exclusive capes, purchasable with Beja Coins — separate from Client Pass unlocks.</p>

      <div v-if="shopStore.loading" class="empty-tab">
        <span class="btn-spinner" style="width:18px;height:18px;border-width:2px" />
      </div>

      <div v-else class="skin-grid">
        <EditableRegion
          v-for="item in capeItems"
          :key="item.id"
          id="shop.itemCard"
          label="Shop-Karte"
          :features="['radius', 'outline']"
        >
        <div
          class="skin-card shop-card"
          :class="{ 'skin-card--active': isOwned(item.id), 'skin-card--legendary': item.rarity === 'legendary' }"
        >
          <span class="cape-builtin-badge" :style="rarityBadgeStyle(item.rarity)">{{ item.rarity.toUpperCase() }}</span>

          <button class="cape-preview-btn" title="Preview on your skin" @click.stop="openPreview(item)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <path d="M1.5 12S5 5 12 5s10.5 7 10.5 7-3.5 7-10.5 7S1.5 12 1.5 12z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </button>

          <div class="cape-front-wrap">
            <div class="cape-3d">
              <div class="cape-face cape-face--back" :style="capeBackStyle(item.id)" />
              <div class="cape-face cape-face--side" :style="capeSideStyle(item.id)" />
              <div class="cape-face cape-face--front" :style="capeTextureStyle(item.id)" />
            </div>
          </div>

          <div class="skin-label-row">
            <span class="skin-username">{{ item.name }}</span>
            <span class="skin-model-tag">{{ item.price }} gems</span>
          </div>
          <div class="shop-card-footer">
            <template v-if="isOwned(item.id)">
              <button
                v-if="item.capeUrl"
                class="shop-buy-btn"
                :class="{ 'shop-buy-btn--confirm': isEquipped(item) }"
                :disabled="equippingId === item.id"
                @click="isEquipped(item) ? unequipCape() : equipCape(item)"
              >
                <span v-if="equippingId === item.id" class="btn-spinner" />
                <template v-else-if="isEquipped(item)">Equipped ✓</template>
                <template v-else>Equip</template>
              </button>
              <span v-else class="shop-owned-tag">Owned</span>
            </template>
            <template v-else-if="confirmingId === item.id">
              <button class="shop-buy-btn shop-buy-btn--confirm" :disabled="shopStore.purchasingId === item.id" @click="buy(item)">
                <span v-if="shopStore.purchasingId === item.id" class="btn-spinner" />
                <template v-else>Confirm — {{ item.price }}?</template>
              </button>
              <button class="shop-cancel-btn" @click="confirmingId = null">Cancel</button>
            </template>
            <template v-else>
              <button
                class="shop-buy-btn"
                :disabled="walletStore.balance < item.price"
                @click="confirmingId = item.id"
              >
                {{ walletStore.balance < item.price ? 'Not enough gems' : `Buy · ${item.price}` }}
              </button>
            </template>
          </div>
        </div>
        </EditableRegion>
      </div>

      <p v-if="shopErrorText" class="shop-error">{{ shopErrorText }}</p>
    </section>

    <!-- Emotes -->
    <section class="shop-section">
      <h2 class="shop-section-title">Emotes</h2>
      <p class="shop-section-sub">Cosmetic chat emotes — flex a little between matches.</p>

      <div v-if="!shopStore.loading" class="emote-grid">
        <div
          v-for="item in emoteItems"
          :key="item.id"
          class="emote-card"
          :style="{ '--emote-accent': rarityColor(item.rarity) }"
          :class="{ 'emote-card--owned': isOwned(item.id) }"
        >
          <div class="emote-icon-wrap">
            <svg viewBox="0 0 24 24">
              <template v-if="item.id === 'shop_emote_floss'">
                <!-- Stick figure mid-floss: arms swung to one side, hips countering the other way -->
                <circle cx="12.6" cy="4.6" r="2" fill="#ffb454" />
                <g stroke="#ffb454" stroke-width="1.8" stroke-linecap="round" fill="none">
                  <path d="M12.4 6.8 11.7 13.2" />
                  <path d="M12.1 8.4 17.4 6.2" />
                  <path d="M12.1 8.4 17.9 9.4" />
                  <path d="M11.7 13.2 9.4 19" />
                  <path d="M11.7 13.2 13.6 19.2" />
                </g>
              </template>
              <template v-else-if="item.id === 'shop_emote_griddy'">
                <!-- Stick figure mid-griddy: both arms pumping forward, back leg kicked up -->
                <circle cx="10" cy="4.6" r="2" fill="#b46eff" />
                <g stroke="#b46eff" stroke-width="1.8" stroke-linecap="round" fill="none">
                  <path d="M10.3 6.8 11.3 13" />
                  <path d="M10.7 8.6 16.5 7.4" />
                  <path d="M10.9 10.2 16.5 9.8" />
                  <path d="M11.3 13 10 19.2" />
                  <path d="M11.3 13 6.4 16.2" />
                </g>
              </template>
              <template v-else-if="item.id === 'shop_emote_rat_dance'">
                <ellipse cx="11" cy="14.8" rx="6.5" ry="4.3" fill="#b9bcc7" />
                <path d="M16.2 12.6 21 14.8l-4.8 2z" fill="#b9bcc7" />
                <circle cx="15" cy="10.2" r="1.9" fill="#b9bcc7" />
                <circle cx="15" cy="10.2" r=".9" fill="#f2a6c0" />
                <circle cx="17.3" cy="14.2" r=".7" fill="#26262e" />
                <path d="M4.7 15.6c-1.9 0-2.8 1.7-1.7 3.1" stroke="#b9bcc7" stroke-width="1.3" fill="none" stroke-linecap="round" />
              </template>
            </svg>
          </div>

          <span class="emote-name">{{ item.name }}</span>

          <div class="emote-footer">
            <template v-if="isOwned(item.id)">
              <span class="emote-owned-tag">Owned</span>
            </template>
            <template v-else-if="confirmingId === item.id">
              <button class="emote-buy-btn emote-buy-btn--confirm" :disabled="shopStore.purchasingId === item.id" @click="buy(item)">
                <span v-if="shopStore.purchasingId === item.id" class="btn-spinner" style="width:10px;height:10px;border-width:2px" />
                <template v-else>{{ item.price }}?</template>
              </button>
              <button class="emote-cancel-btn" @click="confirmingId = null">×</button>
            </template>
            <template v-else>
              <button class="emote-buy-btn" :disabled="walletStore.balance < item.price" @click="confirmingId = item.id">
                {{ item.price }}
              </button>
            </template>
          </div>
        </div>
      </div>
    </section>

    <CapePreviewModal
      v-model="previewOpen"
      :cape-url="previewItem ? CAPE_TEXTURES[previewItem.id] ?? null : null"
      :cape-name="previewItem?.name ?? ''"
    />
    </template>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useWalletStore } from '../store/walletStore'
import { useShopStore } from '../store/shopStore'
import { useAccountStore } from '../store/accountStore'
import { useLockerStore } from '../store/lockerStore'
import type { ShopItem } from '../types'
import CapePreviewModal from '../components/shop/CapePreviewModal.vue'
import WipPage from '../components/common/WipPage.vue'
import EditableRegion from '../components/common/EditableRegion.vue'

// Temporary kill-switch while the shop backend is being reworked — flip back
// to false to restore the real Cape Shop / Emotes sections.
const MAINTENANCE = true

const walletStore  = useWalletStore()
const shopStore    = useShopStore()
const accountStore = useAccountStore()
const lockerStore  = useLockerStore()

const confirmingId = ref<string | null>(null)

const previewOpen = ref(false)
const previewItem = ref<ShopItem | null>(null)
function openPreview(item: ShopItem) {
  previewItem.value = item
  previewOpen.value = true
}

const capeItems  = computed(() => shopStore.items.filter(i => i.type === 'cape'))
const emoteItems = computed(() => shopStore.items.filter(i => i.type === 'emote'))

// ── Equip / unequip a purchased cape ───────────────────────────────────────────
// Mirrors LockerPage.vue's equip flow: push the real hosted cape URL (not the
// bundled preview texture — the Minecraft mod fetches it over HTTP) through
// cosmetics:update so it renders in-game, and keep the local account cache +
// 3D preview in sync.
const equippingId = ref<string | null>(null)

function isEquipped(item: ShopItem): boolean {
  return !!item.capeUrl && accountStore.selectedAccount?.capeUrl === item.capeUrl
}

function saveLocalCapeUrl(url: string | null) {
  if (url) localStorage.setItem('beja_local_cape_url', url)
  else     localStorage.removeItem('beja_local_cape_url')
}

async function equipCape(item: ShopItem) {
  if (!item.capeUrl || equippingId.value) return
  equippingId.value = item.id
  try {
    saveLocalCapeUrl(item.capeUrl)
    lockerStore.selectSkin({ skinUrl: lockerStore.skinUrl, capeUrl: item.capeUrl, model: lockerStore.model })
    const acc = accountStore.selectedAccount
    if (acc) acc.capeUrl = item.capeUrl
    await window.api.cosmetics.update({ cape_url: item.capeUrl })
  } catch (e) {
    console.error('[Shop] equip failed — local preview updated but the server was not, so it will revert on next launch:', e)
    shopErrorText.value = 'Equip failed to save — try again.'
    setTimeout(() => { shopErrorText.value = '' }, 3200)
  } finally {
    equippingId.value = null
  }
}

async function unequipCape() {
  if (equippingId.value) return
  try {
    saveLocalCapeUrl(null)
    lockerStore.selectSkin({ skinUrl: lockerStore.skinUrl, capeUrl: null, model: lockerStore.model })
    const acc = accountStore.selectedAccount
    if (acc) acc.capeUrl = null
    await window.api.cosmetics.update({ cape_url: null })
  } catch (e) {
    console.error('[Shop] unequip failed:', e)
  }
}

const RARITY_COLORS: Record<string, string> = {
  common:    '#9aa0a6',
  uncommon:  '#4caf50',
  rare:      '#3ea8ff',
  epic:      '#b46eff',
  legendary: '#ffb300',
}

function rarityColor(rarity: string): string {
  return RARITY_COLORS[rarity] ?? RARITY_COLORS.common
}

function rarityBadgeStyle(rarity: string) {
  const c = rarityColor(rarity)
  return {
    background: `${c}22`,
    borderColor: `${c}88`,
    color: c,
  }
}

// ── Real cape textures (512x256, generated by scripts/generate-shop-capes.js) ─
// FRONT face lives at (8,8) 80x128 within the 512x256 texture — crop + scale
// that region to fill the card art, same UV layout the in-game cape uses.
import texBlackout from '../assets/capes/shop_cape_blackout.png'
import texPink     from '../assets/capes/shop_cape_pink.png'

const CAPE_TEXTURES: Record<string, string> = {
  shop_cape_blackout: texBlackout,
  shop_cape_pink: texPink,
}

const CAPE_SCALE = 1.125 // display size = one face (80x128) * scale = 90x144
// Crops one named UV region (FRONT/BACK/L strip) out of the same 512x256
// texture — the 3D card is built from these real regions (not a flat fake),
// so tilting it actually reveals the cape's real back and side pixels.
function capeCropStyle(id: string, x: number, y: number) {
  const tex = CAPE_TEXTURES[id]
  if (!tex) return {}
  return {
    backgroundImage: `url(${tex})`,
    backgroundSize: `${512 * CAPE_SCALE}px ${256 * CAPE_SCALE}px`,
    backgroundPosition: `-${x * CAPE_SCALE}px -${y * CAPE_SCALE}px`,
  }
}
function capeTextureStyle(id: string) { return capeCropStyle(id, 8, 8) }  // FRONT
function capeBackStyle(id: string)    { return capeCropStyle(id, 96, 8) } // BACK
function capeSideStyle(id: string)    { return capeCropStyle(id, 0, 8) } // L strip (cape thickness)

function isOwned(itemId: string): boolean {
  return shopStore.owned.includes(itemId)
}

const ERROR_TEXT: Record<string, string> = {
  insufficient_funds: "You don't have enough Beja Coins for that yet.",
  already_owned: 'You already own this.',
  purchase_failed: 'Purchase failed — try again.',
  no_token: 'Sign in to make purchases.',
  network_error: "Couldn't reach the shop — try again.",
}
const shopErrorText = ref('')

async function buy(item: ShopItem) {
  const res = await shopStore.purchase(item.id)
  confirmingId.value = null
  if (!res.purchased) {
    shopErrorText.value = ERROR_TEXT[res.reason ?? ''] ?? ERROR_TEXT.purchase_failed
    setTimeout(() => { shopErrorText.value = '' }, 3200)
  }
}

onMounted(() => {
  if (MAINTENANCE) return
  walletStore.load()
  shopStore.load()
})
</script>

<style lang="scss" scoped>
.shop-page {
  display: flex;
  flex-direction: column;
  gap: 28px;
  height: 100%;
  overflow-y: auto;
  padding: 4px 2px 24px;
}

// ── Sections ──────────────────────────────────────────────────────────────────
.shop-section-title {
  font-family: 'Mojangles', monospace;
  font-size: 18px;
  color: $text-primary;
  margin-bottom: 4px;
}

.shop-section-sub {
  font-size: 12px;
  color: $text-secondary;
  margin-bottom: 18px;
}

// ── Reused Capes-tab card shell ───────────────────────────────────────────────
.skin-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 32px;
}

@property --sc-angle {
  syntax: '<angle>';
  initial-value: 45deg;
  inherits: false;
}

.skin-card {
  width: 260px;
  height: 340px;
  border: 1px solid transparent;
  border-radius: var(--edr-radius, 0px);
  outline: var(--edr-outline, none);
  outline-offset: 3px;
  background-image:
    linear-gradient(#111, #111),
    conic-gradient(
      from var(--sc-angle),
      rgba(255,255,255,.04) 0%,
      rgba(255,255,255,.55) 18%,
      rgba(255,255,255,.04) 36%,
      rgba(255,255,255,.04) 100%
    );
  background-origin: border-box;
  background-clip: padding-box, border-box;
  transition: --sc-angle 600ms cubic-bezier(.2,0,0,1), transform 400ms cubic-bezier(.2,0,0,1), box-shadow 400ms cubic-bezier(.2,0,0,1);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-end;
  position: relative;
  flex-shrink: 0;
  overflow: hidden;

  &:hover {
    --sc-angle: 135deg;
    transform: perspective(900px) rotateY(-7deg) rotateX(4deg) translateZ(6px);
    box-shadow: 8px 16px 40px rgba(0,0,0,0.7);
  }

  &--active {
    --sc-angle: 135deg;
    background-image:
      linear-gradient(#151515, #151515),
      conic-gradient(
        from var(--sc-angle),
        rgba(255,255,255,.08) 0%,
        rgba(255,255,255,.75) 18%,
        rgba(255,255,255,.08) 36%,
        rgba(255,255,255,.08) 100%
      );
  }

  &--legendary {
    box-shadow: 0 0 30px rgba(255, 179, 0, 0.16);
  }
}

.cape-front-wrap {
  flex: 1;
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  perspective: 700px;
  position: relative;
}

// Real 3D box built from the cape's own FRONT/BACK/L-strip texture regions —
// tilting it actually reveals real cape pixels on the side and back, not a
// flat card faked with a drop-shadow.
.cape-3d {
  position: relative;
  width: 90px;
  height: 144px;
  transform-style: preserve-3d;
  transform: rotateY(-16deg);
  transition: transform 500ms cubic-bezier(.2,0,0,1);
  filter: drop-shadow(6px 16px 26px rgba(0, 0, 0, 0.65));
}

.skin-card:hover .cape-3d { transform: rotateY(-30deg) rotateX(3deg) scale(1.05); }

.cape-face {
  position: absolute;
  top: 0;
  left: 0;
  background-repeat: no-repeat;
  image-rendering: pixelated;
  backface-visibility: hidden;
}

.cape-face--front {
  width: 90px;
  height: 144px;
  z-index: 2;
}

.cape-face--back {
  width: 90px;
  height: 144px;
  transform: translateZ(-9px);
  filter: brightness(0.55);
}

.cape-face--side {
  width: 9px;
  height: 144px;
  transform-origin: left center;
  transform: rotateY(-90deg);
  filter: brightness(0.7);
}

.skin-label-row {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 5px 7px;
  background: rgba(0,0,0,0.5);
  flex-shrink: 0;
}

.skin-username {
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  color: #aaa;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  letter-spacing: 0.02em;
}

.skin-model-tag {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  color: #555;
  flex-shrink: 0;
  margin-left: 4px;
}

.cape-builtin-badge {
  position: absolute;
  top: 6px;
  left: 6px;
  font-family: 'Mojangles', monospace;
  font-size: 7px;
  letter-spacing: 0.1em;
  border: 1px solid;
  padding: 2px 6px;
  pointer-events: none;
  z-index: 2;
}

.cape-preview-btn {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.55);
  border: none;
  color: #ccc;
  cursor: pointer;
  z-index: 2;
  transition: background 100ms, color 100ms;

  svg { width: 14px; height: 14px; }

  &:hover { background: rgba(0, 0, 0, 0.8); color: #fff; }
}

.empty-tab {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 0;
}

// ── Shop-specific footer ──────────────────────────────────────────────────────
.shop-card-footer {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px;
  background: #0a0a0a;
  flex-shrink: 0;
}

.shop-buy-btn {
  flex: 1;
  padding: 7px 0;
  background: #1a1a1a;
  border: 1px solid transparent;
  color: #ddd;
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  cursor: pointer;
  transition: background 100ms, border-color 100ms;

  &:hover:not(:disabled) { background: #222; }
  &:disabled { cursor: not-allowed; opacity: 0.4; }

  &--confirm {
    border-color: rgba(255, 179, 0, 0.5);
    color: #ffb300;
    &:hover:not(:disabled) { background: rgba(255, 179, 0, 0.1); }
  }
}

.shop-cancel-btn {
  padding: 7px 12px;
  background: none;
  border: none;
  color: #777;
  font-size: 11px;
  cursor: pointer;
  transition: color 100ms;
  &:hover { color: #aaa; }
}

.shop-owned-tag {
  flex: 1;
  text-align: center;
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  color: #4caf50;
  padding: 7px 0;
}

.shop-error {
  margin-top: 12px;
  font-size: 12px;
  color: #e05050;
}

// ── Emotes ────────────────────────────────────────────────────────────────────
.emote-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
}

.emote-card {
  width: 118px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 14px 10px 10px;
  background: rgba(10, 10, 11, 0.72);
  border: 1px solid $border;
  box-shadow: $shadow-xs;
  transition: border-color 80ms, background 80ms, box-shadow 80ms;

  &:hover { background: rgba(20, 20, 22, 0.85); border-color: $border-strong; box-shadow: $shadow-sm; }

  &--owned {
    border-color: color-mix(in srgb, var(--emote-accent) 55%, transparent);
  }
}

.emote-icon-wrap {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;

  svg { width: 34px; height: 34px; }
}

.emote-name {
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  color: $text-primary;
  text-align: center;
}

.emote-footer {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
}

.emote-buy-btn {
  flex: 1;
  padding: 5px 0;
  background: #1a1a1a;
  border: 1px solid transparent;
  color: #ddd;
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  cursor: pointer;
  transition: background 100ms, border-color 100ms;

  &:hover:not(:disabled) { background: #222; }
  &:disabled { cursor: not-allowed; opacity: 0.4; }

  &--confirm {
    border-color: rgba(255, 179, 0, 0.5);
    color: #ffb300;
    &:hover:not(:disabled) { background: rgba(255, 179, 0, 0.1); }
  }
}

.emote-cancel-btn {
  padding: 5px 8px;
  background: none;
  border: none;
  color: #777;
  font-size: 11px;
  cursor: pointer;
  transition: color 100ms;
  &:hover { color: #aaa; }
}

.emote-owned-tag {
  flex: 1;
  text-align: center;
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  color: #4caf50;
  padding: 5px 0;
}

// ── Spinner (mirrors LockerPage's) ────────────────────────────────────────────
.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255,255,255,0.25);
  border-top-color: #fff;
  border-radius: 50%;
  animation: shop-spin 0.7s linear infinite;
  display: inline-block;
}

@keyframes shop-spin {
  to { transform: rotate(360deg); }
}
</style>
