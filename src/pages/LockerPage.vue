<template>
  <div class="locker-page">

    <!-- Tabs -->
    <div class="tab-row">
      <button
        v-for="t in tabs"
        :key="t"
        :ref="(el) => setTabBtnRef(t, el as HTMLElement | null)"
        class="tab-btn"
        :class="{ active: activeTab === t }"
        @click="activeTab = t"
      >{{ t }}</button>
      <span class="tab-indicator" :style="indicatorStyle" />
    </div>

    <!-- Search -->
    <div class="search-bar">
      <input
        v-model="search"
        class="search-input"
        :placeholder="searchPlaceholder"
      />
      <img :src="searchIcon" class="search-icon" alt="" />
    </div>

    <Transition name="tab-fade" mode="out-in">
    <!-- ── Skins tab ──────────────────────────────────────────────────── -->
    <div v-if="activeTab === 'Skins'" key="skins" class="skin-grid">

      <!-- Add new skin card -->
      <button class="skin-card skin-card--add" @click="addModalOpen = true">
        <!-- ghost bg skin -->
        <div v-if="account?.skinUrl" class="add-card-ghost">
          <StaticSkinViewer
            :skin-url="account.skinUrl"
            :cape-url="account.capeUrl"
            :model="account.skinModel"
            :zoom="0.75"
            :initial-rotation-y="0.65"
          />
        </div>
        <div class="add-card-content">
          <img :src="addIcon" class="add-icon-img" alt="" />
          <span class="add-title">Add new skin</span>
          <span class="add-sub">Import or search by username</span>
        </div>
      </button>

      <!-- Saved skin cards -->
      <div
        v-for="skin in filteredSkins"
        :key="skin.uuid"
        class="skin-card"
        :class="{ 'skin-card--active': skin.uuid === selectedUuid }"
        @click="equipSkin(skin)"
      >
        <button class="skin-del-btn" @click.stop="removeSkin(skin.uuid)" title="Remove">
          <img :src="removeIcon" alt="" />
        </button>

        <!-- 3D viewer for all cards with a skin URL -->
        <template v-if="resolvedSkinUrl(skin)">
          <div class="skin-viewer-wrap">
            <StaticSkinViewer
              :skin-url="resolvedSkinUrl(skin)"
              :cape-url="resolvedCapeUrl(skin)"
              :model="skin.uuid === account?.uuid ? (account.skinModel ?? 'auto-detect') : (skin.model ?? 'auto-detect')"
              :zoom="0.75"
              :initial-rotation-y="0.65"
            />
          </div>
        </template>
        <!-- Fallback: 2D body image -->
        <template v-else>
          <div class="skin-body-wrap">
            <img
              class="skin-body-img"
              :src="`https://mc-heads.net/player/${skin.uuid}/120`"
              :alt="skin.username"
            />
          </div>
        </template>

        <div class="skin-label-row">
          <span class="skin-username">{{ skin.username }}</span>
          <span class="skin-model-tag">{{ skin.model }}</span>
        </div>
      </div>
    </div>

    <!-- ── Capes tab ──────────────────────────────────────────────────── -->
    <div v-else-if="activeTab === 'Capes'" key="capes" class="skin-grid">

      <!-- No cape card -->
      <div
        class="skin-card"
        :class="{ 'skin-card--active': selectedCapeUuid === null }"
        @click="equipNoCape"
      >
        <div class="no-cape-wrap">
          <span class="no-cape-label">None</span>
        </div>
      </div>

      <div v-if="capesLoading" class="empty-tab">
        <span class="btn-spinner" style="width:18px;height:18px;border-width:2px" />
      </div>

      <div
        v-for="cape in filteredCapes"
        :key="cape.id"
        class="skin-card"
        :class="{ 'skin-card--active': cape.id === selectedCapeUuid }"
        @click="equipCape(cape)"
      >
        <span v-if="cape.builtin" class="cape-builtin-badge">BEJA</span>
        <span v-else-if="cape.shop" class="cape-builtin-badge cape-shop-badge">SHOP</span>
        <div class="cape-front-wrap">
          <div class="cape-front" :style="capeFrontStyle(cape.url)" />
        </div>
        <div class="skin-label-row">
          <span class="skin-username">{{ cape.alias }}</span>
          <span class="skin-model-tag">cape</span>
        </div>
      </div>

      <div v-if="!capesLoading && capesError" class="empty-tab" style="width:100%">
        <span class="empty-tab-text" style="color:#c05050">{{ capesError }}</span>
      </div>
      <div v-else-if="!capesLoading && !filteredCapes.length" class="empty-tab" style="width:100%">
        <span class="empty-tab-text">No capes on this account</span>
      </div>
    </div>

    <!-- ── Cosmetics tab ──────────────────────────────────────────────── -->
    <div v-else-if="activeTab === 'Cosmetics'" key="cosmetics" class="cosmetics-grid">
      <div v-if="cosmeticsLoading" class="empty-tab">
        <span class="btn-spinner" style="width:18px;height:18px;border-width:2px" />
      </div>

      <div v-else-if="!ownedCosmetics.length" class="empty-tab" style="width:100%">
        <span class="empty-tab-text">No cosmetics in your collection</span>
      </div>

      <EditableRegion
        v-for="cos in filteredCosmetics"
        :key="cos.id"
        id="locker.cosmeticCard"
        label="Cosmetic-Karte"
        :features="['radius', 'outline', 'color', 'fontFamily']"
      >
      <div
        class="cosmetic-card"
        :class="{ 'cosmetic-card--equipped': cos.equipped }"
        :style="cosmeticCardStyle(cos.rarity)"
        @click="toggleEquip(cos)"
      >
        <div class="cosmetic-viewer">
          <CosmeticModelViewer
            :model-url="cos.model_url"
            :rarity="cos.rarity"
          />
        </div>
        <div class="cosmetic-label-row">
          <span class="cosmetic-name">{{ cos.name || '???' }}</span>
          <span class="cosmetic-rarity" :style="{ color: RARITIES[cos.rarity]?.color }">
            {{ RARITIES[cos.rarity]?.label }}
          </span>
        </div>
      </div>
      </EditableRegion>
    </div>

    <!-- ── Community tab ──────────────────────────────────────────────── -->
    <div v-else-if="activeTab === 'Community'" key="community" class="skin-grid">

      <!-- Upload new cape card -->
      <button class="skin-card skin-card--add" @click="communityUploadModalOpen = true">
        <div class="add-card-content">
          <img :src="addIcon" class="add-icon-img" alt="" />
          <span class="add-title">Upload a cape</span>
          <span class="add-sub">Share a design with everyone</span>
        </div>
      </button>

      <div v-if="communityLoading && !communityCapes.length" class="empty-tab">
        <span class="btn-spinner" style="width:18px;height:18px;border-width:2px" />
      </div>

      <div
        v-for="cape in filteredCommunityCapes"
        :key="cape.id"
        class="skin-card"
        :class="{ 'skin-card--active': cape.url === lockerStore.capeUrl }"
      >
        <button class="skin-del-btn" title="Report" @click.stop="reportCommunityCape(cape)">
          <img :src="removeIcon" alt="" />
        </button>
        <div class="cape-front-wrap">
          <div class="cape-front" :style="capeFrontStyle(cape.url)" />
        </div>
        <div class="skin-label-row">
          <span class="skin-username">{{ cape.name }}</span>
          <span class="skin-model-tag">by {{ cape.uploader_username }}</span>
        </div>
        <div class="shop-card-footer">
          <button
            class="shop-buy-btn"
            :class="{ 'shop-buy-btn--confirm': cape.url === lockerStore.capeUrl }"
            :disabled="equippingCommunityId === cape.id"
            @click="equipCommunityCape(cape)"
          >
            <span v-if="equippingCommunityId === cape.id" class="btn-spinner" />
            <template v-else-if="cape.url === lockerStore.capeUrl">Equipped ✓</template>
            <template v-else>Equip</template>
          </button>
        </div>
      </div>

      <div v-if="!communityLoading && communityError" class="empty-tab" style="width:100%">
        <span class="empty-tab-text" style="color:#c05050">{{ communityError }}</span>
      </div>
      <div v-else-if="!communityLoading && !communityCapes.length" class="empty-tab" style="width:100%">
        <span class="empty-tab-text">No community capes yet — be the first to upload one</span>
      </div>

      <button
        v-if="!communityLoading && communityCapes.length < communityTotal"
        class="shop-cancel-btn"
        style="width:100%; margin-top: 8px;"
        @click="loadCommunityCapes(false)"
      >Load more</button>
    </div>
    </Transition>

    <!-- ── Add skin modal ─────────────────────────────────────────────── -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="addModalOpen" class="modal-overlay" @click.self="closeModal">
          <div class="add-modal">
            <p class="modal-title">Add new skin</p>

            <template v-if="!uploadPreviewUrl">
              <div class="modal-input-wrap" :class="{ focused: modalInputFocused }">
                <input
                  ref="modalInputRef"
                  v-model="usernameInput"
                  class="modal-input"
                  placeholder="Enter username..."
                  @focus="modalInputFocused = true"
                  @blur="modalInputFocused = false"
                  @keyup.enter="lookupSkin"
                />
              </div>

              <div class="modal-divider"><span>or</span></div>

              <input ref="uploadInputRef" type="file" accept="image/png" style="display:none" @change="onSkinFileSelected" />
              <button class="upload-skin-btn" @click="pickSkinFile">
                <img :src="addIcon" alt="" />
                Upload a skin PNG
              </button>
              <span class="upload-skin-hint">64×64 or legacy 64×32</span>

              <Transition name="fade">
                <p v-if="lookupError" class="modal-error">{{ lookupError }}</p>
              </Transition>
              <Transition name="fade">
                <p v-if="uploadError" class="modal-error">{{ uploadError }}</p>
              </Transition>

              <div class="modal-footer">
                <button class="modal-btn modal-btn--cancel" @click="closeModal">Cancel</button>
                <button class="modal-btn modal-btn--confirm" :disabled="!usernameInput.trim() || lookingUp" @click="lookupSkin">
                  <span v-if="lookingUp" class="btn-spinner" />
                  <template v-else>Search</template>
                </button>
              </div>
            </template>

            <template v-else>
              <div class="upload-preview-row">
                <img :src="uploadPreviewUrl" class="upload-preview-img" alt="" />
                <div class="upload-preview-info">
                  <input v-model="uploadName" class="modal-input upload-name-input" placeholder="Skin name" />
                  <div class="model-toggle">
                    <button type="button" :class="{ active: uploadModel === 'default' }" @click="uploadModel = 'default'">Classic</button>
                    <button type="button" :class="{ active: uploadModel === 'slim' }" @click="uploadModel = 'slim'">Slim</button>
                  </div>
                </div>
              </div>

              <div class="modal-footer">
                <button class="modal-btn modal-btn--cancel" @click="uploadPreviewUrl = null">Back</button>
                <button class="modal-btn modal-btn--confirm" @click="confirmUpload">Add skin</button>
              </div>
            </template>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- ── Upload community cape modal ──────────────────────────────────── -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="communityUploadModalOpen" class="modal-overlay" @click.self="closeCommunityUploadModal">
          <div class="add-modal">
            <p class="modal-title">Upload a cape</p>

            <input ref="communityUploadInputRef" type="file" accept="image/png,image/jpeg" style="display:none" @change="onCommunityCapeFileSelected" />

            <template v-if="!communityUploadPreviewUrl">
              <button class="upload-skin-btn" @click="pickCommunityCapeFile">
                <img :src="addIcon" alt="" />
                Choose a PNG or JPG
              </button>
              <span class="upload-skin-hint">Max 5MB &middot; reviewed for content before going public</span>

              <Transition name="fade">
                <p v-if="communityUploadError" class="modal-error">{{ communityUploadError }}</p>
              </Transition>

              <div class="modal-footer">
                <button class="modal-btn modal-btn--cancel" @click="closeCommunityUploadModal">Cancel</button>
              </div>
            </template>

            <template v-else>
              <div class="upload-preview-row">
                <img :src="communityUploadPreviewUrl" class="upload-preview-img" alt="" />
                <div class="upload-preview-info">
                  <input v-model="communityUploadName" class="modal-input upload-name-input" placeholder="Cape name" maxlength="32" />
                </div>
              </div>

              <Transition name="fade">
                <p v-if="communityUploadError" class="modal-error">{{ communityUploadError }}</p>
              </Transition>

              <div class="modal-footer">
                <button class="modal-btn modal-btn--cancel" @click="communityUploadPreviewUrl = null">Back</button>
                <button class="modal-btn modal-btn--confirm" :disabled="communityUploading" @click="confirmCommunityUpload">
                  <span v-if="communityUploading" class="btn-spinner" />
                  <template v-else>Upload</template>
                </button>
              </div>
            </template>
          </div>
        </div>
      </Transition>
    </Teleport>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { useAccountStore } from '../store/accountStore'

import addIcon    from '../assets/icons8-add-64.png'
import removeIcon from '../assets/icons8-remove-24.png'
import searchIcon from '../assets/icons8-search-50.png'
import StaticSkinViewer from '../components/skin/StaticSkinViewer.vue'
import CosmeticModelViewer from '../components/cosmetics/CosmeticModelViewer.vue'
import EditableRegion from '../components/common/EditableRegion.vue'
import { useLockerStore } from '../store/lockerStore'
import { useLobbyStore } from '../store/lobbyStore'
import { useShopStore } from '../store/shopStore'
import type { Rarity, PlayerCosmetic } from '../types/cosmetics'
import type { CommunityCape } from '../types'
import { RARITIES } from '../types/cosmetics'
import { useSlidingTabIndicator } from '../composables/useSlidingTabIndicator'

const lockerStore = useLockerStore()
const lobbyStore  = useLobbyStore()
const shopStore   = useShopStore()

const accountStore = useAccountStore()
const account = computed(() => accountStore.selectedAccount)

// ── Cosmetics ─────────────────────────────────────────────────────────────────
const ownedCosmetics  = ref<PlayerCosmetic[]>([])
const cosmeticsLoading = ref(false)

const filteredCosmetics = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return ownedCosmetics.value
  return ownedCosmetics.value.filter(c =>
    c.name.toLowerCase().includes(q) || c.rarity.toLowerCase().includes(q)
  )
})

function cosmeticCardStyle(rarity: Rarity) {
  const r = RARITIES[rarity]
  if (!r) return {}
  return {
    background:   r.bg,
    borderColor:  r.color + '50',
    boxShadow:    `0 0 12px ${r.glow}`,
  }
}

async function toggleEquip(cos: PlayerCosmetic) {
  cos.equipped = !cos.equipped
  const equipped = ownedCosmetics.value.filter(c => c.equipped).map(c => c.id)
  try { await window.api.cosmetics.update({ equipped }) } catch { /* non-fatal */ }
}

async function loadCosmetics() {
  if (!account.value?.uuid) return
  cosmeticsLoading.value = true
  try {
    const inv = await (window as any).api.cosmetics.inventory(account.value.uuid) as PlayerCosmetic[] | null
    ownedCosmetics.value = inv ?? []
  } catch {
    ownedCosmetics.value = []
  } finally {
    cosmeticsLoading.value = false
  }
}

const tabs        = ['Skins', 'Capes', 'Cosmetics', 'Community'] as const
type Tab          = typeof tabs[number]
const activeTab   = ref<Tab>('Skins')
const { setTabBtnRef, indicatorStyle } = useSlidingTabIndicator(activeTab)
const search      = ref('')

const searchPlaceholder = computed(() => `Search ${activeTab.value.toLowerCase()}...`)

interface SkinEntry {
  uuid: string
  username: string
  model: 'default' | 'slim'
  skinUrl?: string | null
  capeUrl?: string | null
}

const skins       = ref<SkinEntry[]>([])
const selectedUuid = ref<string | null>(null)

const filteredSkins = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return skins.value
  return skins.value.filter(s => s.username.toLowerCase().includes(q))
})

interface CapeEntry {
  id: string
  url: string
  alias: string
  state: 'ACTIVE' | 'INACTIVE'
  builtin?: boolean
  // A real Mojang cape entitlement — id can be passed to the official
  // capes/active endpoint. Custom/BejaClient capes have no such id.
  official?: boolean
  // Bought from the Cape Shop (routes/shop.js) — same underlying cape_url
  // mechanism as any other BejaClient cape, just sourced from shop_purchases.
  shop?: boolean
}

const BEJA_ORIGINAL_ID  = 'beja-original'
const BEJA_ORIGINAL_URL = 'http://127.0.0.1:25588/beja-default.png'

const ownedCapes    = ref<CapeEntry[]>([])
const capesLoading  = ref(false)
const selectedCapeUuid = ref<string | null>(null)

const filteredCapes = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return ownedCapes.value
  return ownedCapes.value.filter(c => c.alias.toLowerCase().includes(q))
})

const capesError = ref<string | null>(null)

// ── Community capes (browse/upload/report — routes/capes.js) ──────────────────
const communityCapes       = ref<CommunityCape[]>([])
const communityTotal       = ref(0)
const communityLoading     = ref(false)
const communityError       = ref<string | null>(null)
const equippingCommunityId = ref<number | null>(null)
const reportedCommunityIds = new Set<number>()

const filteredCommunityCapes = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return communityCapes.value
  return communityCapes.value.filter(c =>
    c.name.toLowerCase().includes(q) || c.uploader_username.toLowerCase().includes(q)
  )
})

async function loadCommunityCapes(reset = true) {
  if (communityLoading.value) return
  communityLoading.value = true
  communityError.value   = null
  try {
    const offset = reset ? 0 : communityCapes.value.length
    const res = await window.api.capes.list(offset)
    communityTotal.value = res.total ?? 0
    communityCapes.value = reset ? (res.capes ?? []) : [...communityCapes.value, ...(res.capes ?? [])]
  } catch {
    communityError.value = 'Could not load community capes.'
  } finally {
    communityLoading.value = false
  }
}

async function equipCommunityCape(cape: CommunityCape) {
  if (equippingCommunityId.value) return
  equippingCommunityId.value = cape.id
  try {
    saveLocalCapeUrl(cape.url)
    lockerStore.selectSkin({ skinUrl: lockerStore.skinUrl, capeUrl: cape.url, model: lockerStore.model })
    syncLocalCosmetics({ capeUrl: cape.url })
    await window.api.cosmetics.update({ cape_url: cape.url })
  } catch (e) {
    console.error('[Community] equip failed:', e)
  } finally {
    equippingCommunityId.value = null
  }
}

async function reportCommunityCape(cape: CommunityCape) {
  if (reportedCommunityIds.has(cape.id)) return
  reportedCommunityIds.add(cape.id)
  try { await window.api.capes.report(cape.id) } catch { /* non-fatal */ }
}

// ── Upload community cape modal ────────────────────────────────────────────────
const communityUploadModalOpen  = ref(false)
const communityUploadInputRef   = ref<HTMLInputElement | null>(null)
const communityUploadPreviewUrl = ref<string | null>(null)
const communityUploadBase64     = ref('')
const communityUploadFilename   = ref('')
const communityUploadName       = ref('')
const communityUploading        = ref(false)
const communityUploadError      = ref('')

function pickCommunityCapeFile() {
  communityUploadInputRef.value?.click()
}

async function onCommunityCapeFileSelected(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  communityUploadError.value = ''

  if (file.type !== 'image/png' && file.type !== 'image/jpeg') {
    communityUploadError.value = 'Please choose a PNG or JPG file'
    return
  }
  if (file.size > 5 * 1024 * 1024) {
    communityUploadError.value = 'File too large (max 5MB)'
    return
  }

  try {
    const dataUrl = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader()
      reader.onerror = () => reject(reader.error)
      reader.onload   = () => resolve(reader.result as string)
      reader.readAsDataURL(file)
    })
    communityUploadPreviewUrl.value = dataUrl
    communityUploadBase64.value     = dataUrl.slice(dataUrl.indexOf(',') + 1)
    communityUploadFilename.value   = file.name
    communityUploadName.value       = file.name.replace(/\.(png|jpe?g)$/i, '').trim() || 'Custom Cape'
  } catch {
    communityUploadError.value = 'Could not read that file'
  }
}

function closeCommunityUploadModal() {
  communityUploadModalOpen.value  = false
  communityUploadPreviewUrl.value = null
  communityUploadBase64.value     = ''
  communityUploadFilename.value   = ''
  communityUploadName.value       = ''
  communityUploadError.value      = ''
}

async function confirmCommunityUpload() {
  if (!communityUploadBase64.value || communityUploading.value) return
  communityUploading.value   = true
  communityUploadError.value = ''
  try {
    const res = await window.api.capes.upload(
      communityUploadBase64.value,
      communityUploadFilename.value,
      communityUploadName.value.trim() || 'Custom Cape'
    )
    if (!res?.ok) {
      communityUploadError.value = res?.error === 'content_rejected'
        ? 'That image was rejected by content moderation.'
        : 'Upload failed — try again.'
      return
    }
    closeCommunityUploadModal()
    await loadCommunityCapes(true)
  } catch {
    communityUploadError.value = 'Upload failed — try again.'
  } finally {
    communityUploading.value = false
  }
}

async function loadOwnedCapes() {
  capesLoading.value = true
  capesError.value   = null
  const collected: CapeEntry[] = []
  let   bejaActiveUrl: string | null = null
  const debugLines: string[] = []

  // 1. BejaClient cosmetics (primary); fall back to localStorage when API is down
  if (account.value?.uuid) {
    try {
      const cos = await window.api.cosmetics.get(account.value.uuid) as Record<string, unknown> | null
      debugLines.push(`beja cosmetics: ${JSON.stringify(cos).slice(0, 80)}`)
      if (cos && typeof cos.cape_url === 'string' && cos.cape_url) bejaActiveUrl = cos.cape_url
    } catch (e) {
      debugLines.push(`beja cosmetics error: ${e}`)
    }
    if (!bejaActiveUrl) {
      const local = loadLocalCapeUrl()
      if (local) { bejaActiveUrl = local; debugLines.push('cape_url from local fallback') }
    }
  } else {
    debugLines.push('no uuid on account')
  }

  // 2. Mojang profile capes (secondary)
  if (account.value?.accessToken) {
    try {
      let data = await window.api.players.mcProfile(account.value.accessToken) as Record<string, unknown>
      debugLines.push(`mojang profile: ${JSON.stringify(data)}`)
      if (!data || data.error || !data.id) {
        debugLines.push(`token invalid/expired, refreshing...`)
        await accountStore.refreshAccount(account.value.id)
        if (account.value?.accessToken) {
          data = await window.api.players.mcProfile(account.value.accessToken) as Record<string, unknown>
          debugLines.push(`mojang profile after refresh: ${JSON.stringify(data)}`)
        }
      }
      if (data && !data.error && data.id) {
        const rawCapes = (data.capes ?? []) as { id: string; url: string; alias?: string; state: string }[]
        debugLines.push(`mojang capes count: ${rawCapes.length}`)
        rawCapes.forEach(c => collected.push({
          id:       c.id,
          url:      c.url,
          alias:    c.alias ?? 'Mojang Cape',
          state:    c.state as 'ACTIVE' | 'INACTIVE',
          official: true,
        }))
      }
    } catch (e) {
      debugLines.push(`mojang error: ${e}`)
    }
  } else {
    debugLines.push('no accessToken on account')
  }

  // 3. Owned Cape Shop purchases — same shop_purchases-backed ownership the
  // Shop page checks, listed here too so equipping doesn't only work from
  // the Shop page.
  if (!shopStore.items.length) {
    try { await shopStore.load() } catch (e) { debugLines.push(`shop load error: ${e}`) }
  }
  shopStore.items
    .filter(i => i.type === 'cape' && i.capeUrl && shopStore.owned.includes(i.id))
    .forEach(i => {
      if (!collected.some(c => c.url === i.capeUrl)) {
        collected.push({ id: i.id, url: i.capeUrl as string, alias: i.name, state: 'INACTIVE', shop: true })
      }
    })

  // 4. If BejaClient cape URL isn't already in the list, add it
  if (bejaActiveUrl && bejaActiveUrl !== BEJA_ORIGINAL_URL && !collected.some(c => c.url === bejaActiveUrl)) {
    collected.unshift({
      id:    bejaActiveUrl,
      url:   bejaActiveUrl,
      alias: 'BejaClient Cape',
      state: 'ACTIVE',
    })
  }

  // 5. Always prepend the built-in BejaClient Original cape
  if (!collected.some(c => c.id === BEJA_ORIGINAL_ID)) {
    collected.unshift({
      id:      BEJA_ORIGINAL_ID,
      url:     BEJA_ORIGINAL_URL,
      alias:   'BejaClient Original',
      state:   'ACTIVE',
      builtin: true,
    })
  }

  ownedCapes.value = collected
  if (bejaActiveUrl) {
    selectedCapeUuid.value = collected.find(c => c.url === bejaActiveUrl)?.id ?? null
    lockerStore.selectSkin({ skinUrl: lockerStore.skinUrl, capeUrl: bejaActiveUrl, model: lockerStore.model })
  } else {
    selectedCapeUuid.value = collected.find(c => c.state === 'ACTIVE')?.id ?? null
  }

  console.log('[Capes]', debugLines.join(' | '))
  if (!collected.length) capesError.value = debugLines.join(' | ')

  capesLoading.value = false
}

function capeFrontStyle(url: string | null) {
  if (!url) return {}
  const s = 20
  return {
    backgroundImage:    `url(${url})`,
    backgroundSize:     `${64 * s}px ${32 * s}px`,
    backgroundPosition: `-${s}px -${s}px`,
    width:              `${10 * s}px`,
    height:             `${16 * s}px`,
    imageRendering:     'pixelated' as const,
  }
}

function saveLocalCapeUrl(url: string | null) {
  if (url) localStorage.setItem('beja_local_cape_url', url)
  else     localStorage.removeItem('beja_local_cape_url')
}

function loadLocalCapeUrl(): string | null {
  return localStorage.getItem('beja_local_cape_url')
}

// The account object cached in accountStore (skinUrl/capeUrl/skinModel) is
// only ever populated at login/refresh — nothing kept it in sync with what
// you actually equip in the Locker. Anything reading it afterwards (like the
// lobby party snapshot taken on create/join) kept showing stale data, e.g.
// "no cape" equipped still showing a cape, or a custom cape never showing at
// all. Mirroring the resolved values here keeps it truthful, and broadcasting
// to an active party makes a live cape/skin change show up for everyone
// immediately instead of only the next time someone (re)joins.
function syncLocalCosmetics(patch: { skinUrl?: string | null; capeUrl?: string | null; skinModel?: 'default' | 'slim' }) {
  const acc = account.value
  if (acc) {
    if (patch.skinUrl   !== undefined) acc.skinUrl   = patch.skinUrl
    if (patch.capeUrl   !== undefined) acc.capeUrl   = patch.capeUrl
    if (patch.skinModel !== undefined) acc.skinModel = patch.skinModel
  }

  const party = lobbyStore.party
  const uuid  = lobbyStore.localUuid
  if (!party || !uuid) return

  const member = party.members.find(m => m.uuid === uuid)
  if (member) {
    if (patch.skinUrl   !== undefined) member.skinUrl   = patch.skinUrl
    if (patch.capeUrl   !== undefined) member.capeUrl   = patch.capeUrl
    if (patch.skinModel !== undefined) member.skinModel = patch.skinModel
  }
  // Server listens on 'party:skin' and derives the party/uuid from the socket
  // session itself — it then rebroadcasts to other members as 'party:skin_update'.
  window.api.lobby.emit('party:skin', {
    skinUrl:   acc?.skinUrl   ?? null,
    capeUrl:   acc?.capeUrl   ?? null,
    skinModel: acc?.skinModel ?? 'default',
  }).catch(() => {})
}

async function equipNoCape() {
  selectedCapeUuid.value = null
  saveLocalCapeUrl(null)
  lockerStore.selectSkin({ skinUrl: lockerStore.skinUrl, capeUrl: null, model: lockerStore.model })
  syncLocalCosmetics({ capeUrl: null })
  try { await window.api.cosmetics.update({ cape_url: null }) } catch { /* non-fatal */ }
  // Clears the *official* Mojang cape selection too — otherwise a previously
  // equipped real cape would keep showing in vanilla Minecraft even though
  // BejaClient itself now shows no cape.
  if (account.value?.accessToken) {
    try { await window.api.players.clearCape(account.value.accessToken) } catch (e) { console.warn('[Capes] clear official cape failed', e) }
  }
}

async function equipCape(cape: CapeEntry) {
  selectedCapeUuid.value = cape.id
  saveLocalCapeUrl(cape.url)
  lockerStore.selectSkin({ skinUrl: lockerStore.skinUrl, capeUrl: cape.url, model: lockerStore.model })
  syncLocalCosmetics({ capeUrl: cape.url })
  try { await window.api.cosmetics.update({ cape_url: cape.url }) } catch { /* non-fatal */ }

  // Only real Mojang cape entitlements can be pushed to the official profile —
  // Mojang has no API to upload a custom cape texture, so BejaClient-only
  // capes stay BejaClient-only (they render via the mod's own mixin instead).
  if (cape.official && account.value?.accessToken) {
    try { await window.api.players.setCape(account.value.accessToken, cape.id) } catch (e) { console.warn('[Capes] set official cape failed', e) }
  }
}

// ── Persist to localStorage ───────────────────────────────────────────────────
function saveSkins() {
  localStorage.setItem('beja_locker_skins', JSON.stringify(skins.value))
}

function loadSkins() {
  try {
    const raw = localStorage.getItem('beja_locker_skins')
    if (raw) skins.value = JSON.parse(raw)
  } catch { /* ignore */ }

  // seed current account if not already in list
  if (account.value?.uuid && !skins.value.some(s => s.uuid === account.value!.uuid)) {
    skins.value.push({
      uuid:    account.value.uuid,
      username: account.value.username,
      model:   account.value.skinModel ?? 'default',
      skinUrl: account.value.skinUrl,
      capeUrl: account.value.capeUrl,
    })
    saveSkins()
  }
}

// ── Actions ───────────────────────────────────────────────────────────────────
function resolvedSkinUrl(skin: SkinEntry): string | null {
  if (skin.uuid === account.value?.uuid) return account.value.skinUrl ?? skin.skinUrl ?? null
  return skin.skinUrl ?? null
}

function resolvedCapeUrl(skin: SkinEntry): string | null {
  if (skin.uuid === account.value?.uuid) return account.value.capeUrl ?? skin.capeUrl ?? null
  return skin.capeUrl ?? null
}

function removeSkin(uuid: string) {
  skins.value = skins.value.filter(s => s.uuid !== uuid)
  saveSkins()
}

async function equipSkin(skin: SkinEntry) {
  selectedUuid.value = skin.uuid
  const model = skin.uuid === account.value?.uuid
    ? (account.value.skinModel ?? 'default')
    : (skin.model ?? 'default')
  const skinUrl = resolvedSkinUrl(skin)
  const capeUrl = resolvedCapeUrl(skin)

  lockerStore.selectSkin({ skinUrl, capeUrl, model })
  syncLocalCosmetics({ skinUrl, capeUrl, skinModel: model })

  // Pushes to the account's real Mojang profile — this is what actually
  // makes vanilla Minecraft (and other players/servers) see the new skin,
  // not just BejaClient's own preview. Locally-uploaded skins only exist as
  // a data: URL, so those go through the raw multipart upload endpoint
  // instead (the same one the official Minecraft Launcher uses for local
  // files) rather than the URL-based endpoint.
  if (skinUrl && account.value?.accessToken) {
    const variant = model === 'slim' ? 'slim' : 'classic'
    try {
      if (skinUrl.startsWith('data:')) {
        const base64 = skinUrl.slice(skinUrl.indexOf(',') + 1)
        await window.api.players.setSkinFile(account.value.accessToken, base64, variant)
      } else {
        await window.api.players.setSkin(account.value.accessToken, skinUrl, variant)
      }
    } catch (e) {
      console.warn('[Skins] set official skin failed', e)
    }
  }
}

// ── Add skin modal ────────────────────────────────────────────────────────────
const addModalOpen      = ref(false)
const usernameInput     = ref('')
const lookingUp         = ref(false)
const lookupError       = ref('')
const modalInputFocused = ref(false)
const modalInputRef     = ref<HTMLInputElement | null>(null)

// Upload-a-local-PNG path — kept separate from the username lookup above.
// A raw local file becomes a data: URL for BejaClient's own viewers, and
// equipSkin() pushes it to the real Mojang profile via a multipart upload
// (see players:set-skin-file) so it becomes your official skin too.
const uploadInputRef   = ref<HTMLInputElement | null>(null)
const uploadPreviewUrl = ref<string | null>(null)
const uploadModel      = ref<'default' | 'slim'>('default')
const uploadName       = ref('')
const uploadError      = ref('')

function closeModal() {
  addModalOpen.value  = false
  usernameInput.value = ''
  lookupError.value   = ''
  uploadPreviewUrl.value = null
  uploadName.value    = ''
  uploadModel.value   = 'default'
  uploadError.value   = ''
}

function pickSkinFile() {
  uploadInputRef.value?.click()
}

function readSkinFile(file: File): Promise<{ dataUrl: string; width: number; height: number }> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onerror = () => reject(reader.error)
    reader.onload = () => {
      const dataUrl = reader.result as string
      const img = new Image()
      img.onload  = () => resolve({ dataUrl, width: img.naturalWidth, height: img.naturalHeight })
      img.onerror = () => reject(new Error('invalid image'))
      img.src = dataUrl
    }
    reader.readAsDataURL(file)
  })
}

async function onSkinFileSelected(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  uploadError.value = ''

  if (file.type !== 'image/png') {
    uploadError.value = 'Please choose a PNG file'
    return
  }

  try {
    // Never resize/recompress a skin texture — every pixel maps to a fixed
    // body-part UV, so only the two exact vanilla dimensions are valid.
    const { dataUrl, width, height } = await readSkinFile(file)
    const validSize = (width === 64 && height === 64) || (width === 64 && height === 32)
    if (!validSize) {
      uploadError.value = `Not a valid skin size (${width}×${height}) — must be 64×64 or 64×32`
      return
    }
    uploadPreviewUrl.value = dataUrl
    uploadName.value = file.name.replace(/\.png$/i, '').trim() || 'Custom Skin'
  } catch {
    uploadError.value = 'Could not read that file'
  }
}

function confirmUpload() {
  if (!uploadPreviewUrl.value) return
  skins.value.push({
    uuid:     `local-${Date.now()}`,
    username: uploadName.value.trim() || 'Custom Skin',
    model:    uploadModel.value,
    skinUrl:  uploadPreviewUrl.value,
    capeUrl:  null,
  })
  saveSkins()
  closeModal()
}

async function lookupSkin() {
  const name = usernameInput.value.trim()
  if (!name || lookingUp.value) return
  lookingUp.value  = true
  lookupError.value = ''
  try {
    const profile = await window.api.players.lookup(name)
    if (!profile) { lookupError.value = `Player "${name}" not found`; return }
    if (!skins.value.some(s => s.uuid === profile.uuid)) {
      skins.value.push({
        uuid:     profile.uuid,
        username: profile.username,
        model:    profile.skinModel ?? 'default',
        skinUrl:  profile.skinUrl,
        capeUrl:  profile.capeUrl,
      })
      saveSkins()
    }
    closeModal()
  } catch {
    lookupError.value = 'Lookup failed. Try again.'
  } finally {
    lookingUp.value = false
  }
}

// focus input when modal opens
import { watch } from 'vue'
watch(activeTab, async (tab) => {
  if (tab === 'Capes' && ownedCapes.value.length === 0 && !capesLoading.value) {
    await loadOwnedCapes()
  }
  if (tab === 'Cosmetics' && !cosmeticsLoading.value) {
    await loadCosmetics()
  }
  if (tab === 'Community' && communityCapes.value.length === 0 && !communityLoading.value) {
    await loadCommunityCapes(true)
  }
})
watch(addModalOpen, async (v) => {
  if (v) { await nextTick(); modalInputRef.value?.focus() }
})

onMounted(() => {
  loadSkins()
  selectedUuid.value = account.value?.uuid ?? null
  if (account.value?.skinUrl) {
    const savedCapeUrl = loadLocalCapeUrl()
    lockerStore.selectSkin({
      skinUrl: account.value.skinUrl,
      capeUrl: savedCapeUrl ?? account.value.capeUrl,
      model:   account.value.skinModel ?? 'default',
    })
  }
})
</script>

<style lang="scss" scoped>
@font-face {
  font-family: 'Mojangles';
  src: url('../assets/fonts/mojangles.ttf') format('truetype');
  font-weight: normal;
  font-style: normal;
}

// ── Page shell ────────────────────────────────────────────────────────────────
.locker-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px 20px;
  gap: 12px;
  overflow: hidden;
  position: relative;
}

// ── Tab row ───────────────────────────────────────────────────────────────────
.tab-row {
  position: relative;
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.tab-btn {
  padding: 8px 22px;
  background: #0d0d0d;
  border: none;
  color: #aaa;
  font-family: 'Mojangles', monospace;
  font-size: 13px;
  cursor: pointer;
  letter-spacing: 0.02em;
  transition: background 80ms, color 80ms;
  border-radius: 0;

  &:hover { background: #1a1a1a; color: #ccc; }

  &.active {
    background: #111;
    color: #d0d0d0;
  }
}

.tab-indicator {
  position: absolute;
  bottom: 0;
  height: 2px;
  background: rgba(255, 255, 255, 0.3);
  pointer-events: none;
  transition: left 260ms cubic-bezier(0.16, 1, 0.3, 1), width 260ms cubic-bezier(0.16, 1, 0.3, 1);
}

// ── Search bar ────────────────────────────────────────────────────────────────
.search-bar {
  display: flex;
  align-items: center;
  background: none;
  border: none;
  height: 36px;
  padding: 0 2px;
  flex-shrink: 0;
  max-width: 360px;
  gap: 8px;
}

.search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  color: #aaa;
  letter-spacing: 0.03em;
  &::placeholder { color: #555; }
}

.search-icon {
  width: 16px;
  height: 16px;
  opacity: 0.5;
  flex-shrink: 0;
  filter: brightness(0) invert(1);
}

// ── Skin grid ─────────────────────────────────────────────────────────────────
.skin-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 32px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
  align-content: flex-start;
  scrollbar-width: thin;
  scrollbar-color: #333 transparent;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: #333; }
}

// ── Base card ─────────────────────────────────────────────────────────────────
@property --sc-angle {
  syntax: '<angle>';
  initial-value: 45deg;
  inherits: false;
}

.skin-card {
  width: 320px;
  height: 460px;
  border: 1px solid transparent;
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
  cursor: pointer;
  flex-shrink: 0;
  overflow: hidden;

  &:hover {
    --sc-angle: 135deg;
    transform: perspective(900px) rotateY(-7deg) rotateX(4deg) translateZ(6px);
    box-shadow: 8px 16px 40px rgba(0,0,0,0.7);
  }

  &--active {
    --sc-angle: 135deg;
    transform: perspective(900px) rotateY(-4deg) rotateX(2deg) translateZ(3px);
    box-shadow: 6px 12px 30px rgba(0,0,0,0.6);
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

  &--add {
    justify-content: center;
    background-image:
      linear-gradient(#0d0d0d, #0d0d0d),
      conic-gradient(
        from var(--sc-angle),
        rgba(255,255,255,.02) 0%,
        rgba(255,255,255,.35) 18%,
        rgba(255,255,255,.02) 36%,
        rgba(255,255,255,.02) 100%
      );
    &:hover { --sc-angle: 135deg; }
  }
}

// ── Add card ghost bg ─────────────────────────────────────────────────────────
.add-card-ghost {
  position: absolute;
  inset: 0;
  filter: saturate(0.15);
  opacity: 0.22;
  pointer-events: none;
}

.add-card-content {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

// ── 3D viewer inside active card ──────────────────────────────────────────────
.skin-viewer-wrap {
  flex: 1;
  width: 100%;
  min-height: 0;
  transition: transform 400ms cubic-bezier(.2,0,0,1);
}

.skin-card:hover .skin-viewer-wrap {
  transform: perspective(700px) rotateY(-6deg) rotateX(3deg) scale(1.04);
}

.skin-card--active .skin-viewer-wrap {
  transform: perspective(700px) rotateY(-3deg) rotateX(2deg) scale(1.02);
}

// ── Add card innards ──────────────────────────────────────────────────────────
.add-icon-img {
  width: 32px;
  height: 32px;
  opacity: 0.4;
  filter: brightness(0) invert(1);
}

.add-title {
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  color: #777;
  text-align: center;
  letter-spacing: 0.03em;
}

.add-sub {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  color: #444;
  text-align: center;
  letter-spacing: 0.02em;
  padding: 0 8px;
  line-height: 1.4;
}

// ── Skin card innards ─────────────────────────────────────────────────────────
.skin-del-btn {
  position: absolute;
  top: 5px;
  right: 5px;
  width: 22px;
  height: 22px;
  background: rgba(0,0,0,0.7);
  border: 1px solid #444;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity 100ms, background 100ms;
  z-index: 2;
  img {
    width: 12px;
    height: 12px;
    filter: brightness(0) invert(1);
    opacity: 0.7;
  }
  &:hover { background: rgba(180,40,40,0.5); img { opacity: 1; } }
}
.skin-card:hover .skin-del-btn { opacity: 1; }

.skin-body-wrap {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding-top: 10px;
}

.skin-body-img {
  max-height: 148px;
  image-rendering: pixelated;
  display: block;
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

// ── Built-in cape badge ───────────────────────────────────────────────────────
.cape-builtin-badge {
  position: absolute;
  top: 6px;
  left: 6px;
  font-family: 'Mojangles', monospace;
  font-size: 7px;
  letter-spacing: 0.1em;
  color: #fff;
  background: rgba(74,255,224,0.18);
  border: 1px solid rgba(74,255,224,0.55);
  padding: 2px 6px;
  pointer-events: none;
  z-index: 2;
}

.cape-shop-badge {
  color: #fff;
  background: rgba(255,179,0,0.18);
  border-color: rgba(255,179,0,0.55);
}

// ── No-cape card ─────────────────────────────────────────────────────────────
.no-cape-wrap {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.no-cape-label {
  font-family: 'Mojangles', monospace;
  font-size: 13px;
  color: #555;
  letter-spacing: 0.06em;
  position: relative;
  &::after {
    content: '';
    position: absolute;
    left: 0;
    top: 50%;
    width: 100%;
    height: 1px;
    background: #444;
  }
}

// ── Cape display ─────────────────────────────────────────────────────────────
.cape-front-wrap {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  perspective: 600px;
}

.cape-front {
  display: block;
  flex-shrink: 0;
  transform: rotateY(-22deg) rotateX(6deg);
  transform-origin: center center;
  transition: transform 400ms cubic-bezier(.2,0,0,1);
  filter:
    drop-shadow(4px 10px 20px rgba(0,0,0,0.85))
    drop-shadow(-2px 0px 6px rgba(0,0,0,0.5));
}

.skin-card:hover .cape-front {
  transform: rotateY(-10deg) rotateX(3deg);
}

.skin-card--active .cape-front {
  transform: rotateY(-10deg) rotateX(3deg);
}

// ── Cosmetics grid ────────────────────────────────────────────────────────────
.cosmetics-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
  align-content: flex-start;
  scrollbar-width: thin;
  scrollbar-color: #333 transparent;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: #333; }
}

.cosmetic-card {
  width: 160px;
  height: 200px;
  border: 1px solid transparent;
  border-radius: var(--edr-radius, 0px);
  outline: var(--edr-outline, none);
  outline-offset: 3px;
  color: var(--edr-color, inherit);
  font-family: var(--edr-font, inherit);
  display: flex;
  flex-direction: column;
  cursor: pointer;
  flex-shrink: 0;
  position: relative;
  overflow: hidden;
  transition: transform 250ms cubic-bezier(.2,0,0,1), box-shadow 250ms;

  &:hover { transform: translateY(-3px); }

  &--equipped {
    &::after {
      content: 'EQUIPPED';
      position: absolute;
      top: 6px;
      left: 6px;
      font-family: 'Mojangles', monospace;
      font-size: 7px;
      color: #fff;
      background: rgba(74,255,224,0.25);
      border: 1px solid rgba(74,255,224,0.6);
      padding: 2px 5px;
      letter-spacing: 0.08em;
      pointer-events: none;
    }
  }
}

.cosmetic-viewer {
  flex: 1;
  position: relative;
  min-height: 0;
}

.cosmetic-label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 5px 7px;
  background: rgba(0,0,0,0.55);
  flex-shrink: 0;
}

.cosmetic-name {
  font-family: var(--edr-font, 'Mojangles', monospace);
  font-size: 9px;
  color: var(--edr-color, #ccc);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  letter-spacing: 0.02em;
}

.cosmetic-rarity {
  font-family: 'Mojangles', monospace;
  font-size: 7px;
  flex-shrink: 0;
  margin-left: 4px;
  letter-spacing: 0.05em;
}

// ── Empty tabs ────────────────────────────────────────────────────────────────
.empty-tab {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-tab-text {
  font-family: 'Mojangles', monospace;
  font-size: 12px;
  color: #444;
  letter-spacing: 0.04em;
}

// ── Add skin modal ────────────────────────────────────────────────────────────
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
  backdrop-filter: blur(3px);
}

.add-modal {
  background: #111;
  border: 1px solid #444;
  padding: 22px 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 300px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.8);
}

.modal-title {
  font-family: 'Mojangles', monospace;
  font-size: 13px;
  color: #ccc;
  margin: 0;
  letter-spacing: 0.04em;
}

.modal-input-wrap {
  display: flex;
  align-items: center;
  background: #0d0d0d;
  border: 1px solid #3a3a3a;
  height: 36px;
  padding: 0 10px;
  transition: border-color 100ms;
  &.focused { border-color: #777; }
}

.modal-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  color: #ccc;
  &::placeholder { color: #444; }
}

.modal-error {
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  color: #c05050;
  margin: 0;
  letter-spacing: 0.03em;
}

// ── Upload-a-PNG path ─────────────────────────────────────────────────────────
.modal-divider {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #444;
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  letter-spacing: 0.06em;

  &::before, &::after {
    content: '';
    flex: 1;
    height: 1px;
    background: #333;
  }
}

.upload-skin-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  background: #0d0d0d;
  border: 1px dashed #3a3a3a;
  color: #999;
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  padding: 10px;
  cursor: pointer;
  transition: background 100ms, border-color 100ms, color 100ms;

  img {
    width: 14px;
    height: 14px;
    opacity: 0.6;
    filter: brightness(0) invert(1);
  }

  &:hover {
    background: #161616;
    border-color: #666;
    color: #ccc;
  }
}

.upload-skin-hint {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  color: #444;
  letter-spacing: 0.03em;
  text-align: center;
}

.upload-preview-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.upload-preview-img {
  width: 72px;
  height: 72px;
  object-fit: contain;
  image-rendering: pixelated;
  background:
    repeating-conic-gradient(#1a1a1a 0% 25%, #111 0% 50%) 50% / 10px 10px;
  border: 1px solid #333;
  flex-shrink: 0;
}

.upload-preview-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.upload-name-input {
  background: #0d0d0d;
  border: 1px solid #3a3a3a;
  height: 32px;
  padding: 0 10px;
}

.model-toggle {
  display: flex;
  gap: 6px;

  button {
    flex: 1;
    padding: 6px 0;
    background: #0d0d0d;
    border: 1px solid #3a3a3a;
    color: #777;
    font-family: 'Mojangles', monospace;
    font-size: 9px;
    letter-spacing: 0.03em;
    cursor: pointer;
    transition: background 100ms, border-color 100ms, color 100ms;

    &:hover { color: #aaa; }

    &.active {
      background: #1c1c1c;
      border-color: #888;
      color: #ddd;
    }
  }
}

.modal-footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.modal-btn {
  padding: 7px 18px;
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  cursor: pointer;
  border: 1px solid #444;
  letter-spacing: 0.04em;
  transition: background 80ms, border-color 80ms;
  display: flex;
  align-items: center;
  gap: 6px;

  &--cancel {
    background: #0d0d0d;
    color: #777;
    &:hover { background: #1a1a1a; color: #aaa; }
  }

  &--confirm {
    background: #222;
    color: #ccc;
    border-color: #666;
    &:hover:not(:disabled) { background: #2a2a2a; border-color: #999; }
    &:disabled { opacity: 0.35; cursor: not-allowed; }
  }
}

.btn-spinner {
  width: 11px;
  height: 11px;
  border: 1.5px solid #444;
  border-top-color: #ccc;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

// ── Transitions ───────────────────────────────────────────────────────────────
.modal-enter-active { transition: opacity 150ms ease, transform 150ms ease; }
.modal-leave-active { transition: opacity 100ms ease; }
.modal-enter-from   { opacity: 0; transform: scale(0.97); }
.modal-leave-to     { opacity: 0; }

.fade-enter-active, .fade-leave-active { transition: opacity 150ms; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
