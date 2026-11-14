<template>
  <div class="locker">

    <!-- Search + tab pills (same row as Explore) -->
    <div class="search-tab-row">
      <div class="search-bar">
        <input v-model="search" class="search-input" :placeholder="'Search ' + activeTab.toLowerCase() + '...'" />
      </div>
      <button v-for="t in tabs" :key="t" class="pill" :class="{ active: activeTab === t }" @click="activeTab = t">{{ t }}</button>
      <button v-if="activeTab === 'Skins'" class="cta" @click="addModalOpen = true">Add skin</button>
      <button v-else-if="activeTab === 'Community'" class="cta" @click="communityUploadModalOpen = true">Upload cape</button>
    </div>

    <!-- Sort + view toggle -->
    <div class="controls-row">
      <select v-model="sort" class="filter-select">
        <option value="default">Default order</option>
        <option value="name">Name A-Z</option>
      </select>
      <div v-if="activeTab !== 'Cosmetics'" class="view-toggle">
        <button class="view-toggle-btn" :class="{ active: viewMode === 'list' }" title="List view" @click="viewMode = 'list'">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="1" y="2" width="12" height="2" fill="currentColor"/>
            <rect x="1" y="6" width="12" height="2" fill="currentColor"/>
            <rect x="1" y="10" width="12" height="2" fill="currentColor"/>
          </svg>
        </button>
        <button class="view-toggle-btn" :class="{ active: viewMode === 'grid' }" title="Grid view" @click="viewMode = 'grid'">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="1" y="1" width="5" height="5" fill="currentColor"/>
            <rect x="8" y="1" width="5" height="5" fill="currentColor"/>
            <rect x="1" y="8" width="5" height="5" fill="currentColor"/>
            <rect x="8" y="8" width="5" height="5" fill="currentColor"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Active filter chips -->
    <div v-if="category !== 'All' || search" class="chip-row">
      <button v-if="category !== 'All'" class="chip" @click="category = 'All'">{{ category }} ×</button>
      <button v-if="search" class="chip" @click="search = ''">"{{ search }}" ×</button>
      <button class="chip chip--clear" @click="category = 'All'; search = ''">Clear all</button>
    </div>

    <div class="locker-body">

    <!-- Collection sidebar (same panel as the Explore categories) -->
    <aside class="categories-sidebar">
      <p class="categories-title">COLLECTION</p>
      <button
        v-for="c in categories"
        :key="c"
        class="category-row"
        :class="{ active: category === c }"
        @click="category = c"
      >{{ c }}</button>
    </aside>

    <div class="locker-main">

    <!-- Cosmetics: a grid, tap one to wear it or take it off -->
    <div v-if="activeTab === 'Cosmetics'" class="cos-grid">
      <p v-if="cosmeticsLoading" class="hint">Loading…</p>
      <p v-else-if="!ownedCosmetics.length" class="hint">No cosmetics in your collection yet.</p>
      <p v-else-if="!filteredCosmetics.length" class="hint">Nothing matches your filters.</p>
      <button
        v-for="cos in filteredCosmetics"
        :key="cos.id"
        class="cos-card"
        :class="{ on: cos.equipped }"
        :style="cosmeticCardStyle(cos.rarity)"
        @click="toggleEquip(cos)"
      >
        <span class="cos-view"><CosmeticModelViewer :model-url="cos.model_url" :rarity="cos.rarity" /></span>
        <span class="cos-label">
          <span class="cos-name">{{ cos.name || '???' }}</span>
          <span class="cos-rarity" :style="{ color: RARITIES[cos.rarity]?.color }">{{ cos.equipped ? 'Equipped' : RARITIES[cos.rarity]?.label }}</span>
        </span>
      </button>
    </div>

    <!-- Skins, Capes, Community: pick on the left, see it on the right, one Equip button -->
    <div v-else class="body" :class="{ 'body--grid': viewMode === 'grid' }">
      <div class="list" :class="{ 'list--grid': viewMode === 'grid' }">
        <p v-if="listLoading" class="hint">Loading…</p>
        <div v-for="r in filteredRows" :key="r.id" class="item">
        <button
          class="row"
          :class="{ sel: r.id === selectedRowId, worn: r.worn }"
          @click="pick(r.id)"
          @mouseenter="selectRow(r.id)"
          @focus="selectRow(r.id)"
        >
          <span class="row-art">
            <SkinPortrait v-if="r.skin && resolvedSkinUrl(r.skin)" :skin-url="resolvedSkinUrl(r.skin)" :model="r.skin.model ?? 'default'" :scale="6" />
            <img v-else-if="r.skin" class="row-head" :src="`https://mc-heads.net/player/${r.skin.uuid}/120`" alt="" />
            <CapePortrait v-else-if="r.capeUrl" :cape-url="r.capeUrl" :scale="6" />
            <span v-else class="row-none">&ndash;</span>
          </span>
          <span class="row-text">
            <span class="row-name">{{ r.name }}</span>
            <span v-if="r.sub" class="row-sub">{{ r.sub }}</span>
          </span>
          <span v-if="r.worn" class="row-tag">Equipped</span>
        </button>
        <button v-if="!r.worn && activeTab === 'Skins' && skins.length > 1" class="row-action" title="Remove skin" aria-label="Remove skin" @click="removeSkin(r.id)">&times;</button>
        <button v-else-if="!r.worn && activeTab === 'Community'" class="row-action row-action--text" title="Report this cape" @click="reportRow(r.id)">Report</button>
        </div>
        <p v-if="listError" class="hint hint--error">{{ listError }}</p>
        <p v-else-if="!listLoading && !filteredRows.length" class="hint">{{ rows.length ? 'Nothing matches your filters.' : emptyText }}</p>
        <button v-if="activeTab === 'Community' && !communityLoading && communityCapes.length < communityTotal" class="more" @click="loadCommunityCapes(false)">Load more</button>
      </div>

      <section class="preview">
        <div class="preview-view">
          <HeroSkinViewer
            v-if="preview.skinUrl"
            :skin-url="preview.skinUrl"
            :cape-url="preview.capeUrl"
            :model="preview.model"
            animation="static-pose"
            :arm-spread="0.2"
            :zoom="0.72"
            :initial-rotation-y="activeTab === 'Skins' ? 0.4 : Math.PI"
          />
        </div>
        <div class="preview-bar">
          <div class="preview-text">
            <span class="preview-name">{{ preview.name }}</span>
            <span class="preview-sub">{{ busy ? 'Equipping…' : preview.equipped ? 'You are wearing this' : preview.sub }}</span>
          </div>
        </div>
      </section>
    </div>

    </div>
    </div>

    <!-- Add skin -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="addModalOpen" class="overlay" @click.self="closeModal">
          <div class="dialog">
            <p class="dialog-title">Add skin</p>

            <template v-if="!uploadPreviewUrl">
              <input
                ref="modalInputRef"
                v-model="usernameInput"
                class="field"
                placeholder="Minecraft username"
                @keyup.enter="lookupSkin"
              />
              <p class="dialog-or">or</p>
              <input ref="uploadInputRef" type="file" accept="image/png" style="display:none" @change="onSkinFileSelected" />
              <button class="btn btn--ghost btn--wide" @click="pickSkinFile">Upload a skin PNG</button>
              <span class="dialog-hint">64×64 or 64×32</span>
              <p v-if="lookupError || uploadError" class="dialog-error">{{ lookupError || uploadError }}</p>
              <div class="dialog-foot">
                <button class="btn btn--ghost" @click="closeModal">Cancel</button>
                <button class="btn btn--primary" :disabled="!usernameInput.trim() || lookingUp" @click="lookupSkin">{{ lookingUp ? 'Searching…' : 'Search' }}</button>
              </div>
            </template>

            <template v-else>
              <div class="dialog-preview">
                <img :src="uploadPreviewUrl" class="dialog-img" alt="" />
                <div class="dialog-col">
                  <input v-model="uploadName" class="field" placeholder="Skin name" />
                  <div class="seg">
                    <button type="button" :class="{ on: uploadModel === 'default' }" @click="uploadModel = 'default'">Classic</button>
                    <button type="button" :class="{ on: uploadModel === 'slim' }" @click="uploadModel = 'slim'">Slim</button>
                  </div>
                </div>
              </div>
              <div class="dialog-foot">
                <button class="btn btn--ghost" @click="uploadPreviewUrl = null">Back</button>
                <button class="btn btn--primary" @click="confirmUpload">Add skin</button>
              </div>
            </template>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Upload cape -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="communityUploadModalOpen" class="overlay" @click.self="closeCommunityUploadModal">
          <div class="dialog">
            <p class="dialog-title">Upload a cape</p>
            <input ref="communityUploadInputRef" type="file" accept="image/png,image/jpeg" style="display:none" @change="onCommunityCapeFileSelected" />

            <template v-if="!communityUploadPreviewUrl">
              <button class="btn btn--ghost btn--wide" @click="pickCommunityCapeFile">Choose a PNG or JPG</button>
              <span class="dialog-hint">Max 5 MB. Capes are checked before they go public.</span>
              <p v-if="communityUploadError" class="dialog-error">{{ communityUploadError }}</p>
              <div class="dialog-foot">
                <button class="btn btn--ghost" @click="closeCommunityUploadModal">Cancel</button>
              </div>
            </template>

            <template v-else>
              <div class="dialog-preview">
                <img :src="communityUploadPreviewUrl" class="dialog-img" alt="" />
                <div class="dialog-col">
                  <input v-model="communityUploadName" class="field" placeholder="Cape name" maxlength="32" />
                </div>
              </div>
              <p v-if="communityUploadError" class="dialog-error">{{ communityUploadError }}</p>
              <div class="dialog-foot">
                <button class="btn btn--ghost" @click="communityUploadPreviewUrl = null">Back</button>
                <button class="btn btn--primary" :disabled="communityUploading" @click="confirmCommunityUpload">{{ communityUploading ? 'Uploading…' : 'Upload' }}</button>
              </div>
            </template>
          </div>
        </div>
      </Transition>
    </Teleport>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { useAccountStore } from '../store/accountStore'
import SkinPortrait from '../components/skin/SkinPortrait.vue'
import CapePortrait from '../components/skin/CapePortrait.vue'
import HeroSkinViewer from '../components/skin/HeroSkinViewer.vue'
import CosmeticModelViewer from '../components/cosmetics/CosmeticModelViewer.vue'
import { useLockerStore } from '../store/lockerStore'
import { useLobbyStore } from '../store/lobbyStore'
import { useShopStore } from '../store/shopStore'
import type { Rarity, PlayerCosmetic } from '../types/cosmetics'
import type { CommunityCape } from '../types'
import { RARITIES, RARITY_ORDER } from '../types/cosmetics'
import { showToast } from '../composables/useToasts'

const lockerStore = useLockerStore()
const lobbyStore  = useLobbyStore()
const shopStore   = useShopStore()

const accountStore = useAccountStore()
const account = computed(() => accountStore.selectedAccount)

const tabs      = ['Skins', 'Capes', 'Community', 'Cosmetics'] as const
type Tab        = typeof tabs[number]
const activeTab = ref<Tab>('Skins')
const busy      = ref(false)

// ── Cosmetics ─────────────────────────────────────────────────────────────────
const ownedCosmetics   = ref<PlayerCosmetic[]>([])
const cosmeticsLoading = ref(false)

function cosmeticCardStyle(rarity: Rarity) {
  const r = RARITIES[rarity]
  if (!r) return {}
  return { background: r.bg, borderColor: r.color + '50', boxShadow: `0 0 12px ${r.glow}` }
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

// ── Skins ─────────────────────────────────────────────────────────────────────
interface SkinEntry {
  uuid: string
  username: string
  model: 'default' | 'slim'
  skinUrl?: string | null
  capeUrl?: string | null
}

const skins        = ref<SkinEntry[]>([])
const selectedUuid = ref<string | null>(null)

// the cape is one setting for the whole account, shown on whichever skin is on stage
const equippedCapeUrl = computed(() => lockerStore.capeUrl ?? account.value?.capeUrl ?? null)
const heroSkin = computed(() => skins.value.find(s => s.uuid === selectedUuid.value) ?? skins.value[0] ?? null)

// ── Capes ─────────────────────────────────────────────────────────────────────
interface CapeEntry {
  id: string
  url: string
  alias: string
  state: 'ACTIVE' | 'INACTIVE'
  builtin?: boolean
  // A real Mojang cape entitlement - id can be passed to the official
  // capes/active endpoint. Custom/BejaClient capes have no such id.
  official?: boolean
  // Bought from the Cape Shop (routes/shop.js) - same underlying cape_url
  // mechanism as any other BejaClient cape, just sourced from shop_purchases.
  shop?: boolean
}

const BEJA_ORIGINAL_ID  = 'beja-original'
const BEJA_ORIGINAL_URL = 'http://127.0.0.1:25588/beja-default.png'

const ownedCapes    = ref<CapeEntry[]>([])
const capesLoading  = ref(false)
const selectedCapeUuid = ref<string | null>(null)

const capesError = ref<string | null>(null)

// ── Community capes (browse, upload, report: routes/capes.js) ─────────────────
const communityCapes       = ref<CommunityCape[]>([])
const communityTotal       = ref(0)
const communityLoading     = ref(false)
const communityError       = ref<string | null>(null)
const equippingCommunityId = ref<number | null>(null)
const reportedCommunityIds = new Set<number>()

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
  try {
    saveLocalCapeUrl(cape.url)
    lockerStore.selectSkin({ skinUrl: lockerStore.skinUrl, capeUrl: cape.url, model: lockerStore.model })
    syncLocalCosmetics({ capeUrl: cape.url })
    await window.api.cosmetics.update({ cape_url: cape.url })
  } catch (e) {
    console.error('[Community] equip failed:', e)
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
        : 'Upload failed - try again.'
      return
    }
    closeCommunityUploadModal()
    await loadCommunityCapes(true)
  } catch {
    communityUploadError.value = 'Upload failed - try again.'
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

  // 3. Owned Cape Shop purchases - same shop_purchases-backed ownership the
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

function saveLocalCapeUrl(url: string | null) {
  if (url) localStorage.setItem('beja_local_cape_url', url)
  else     localStorage.removeItem('beja_local_cape_url')
}

function loadLocalCapeUrl(): string | null {
  return localStorage.getItem('beja_local_cape_url')
}

// The account object cached in accountStore (skinUrl/capeUrl/skinModel) is
// only ever populated at login/refresh - nothing kept it in sync with what
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
  // session itself - it then rebroadcasts to other members as 'party:skin_update'.
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
  // Clears the *official* Mojang cape selection too - otherwise a previously
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

  // Only real Mojang cape entitlements can be pushed to the official profile -
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
// A saved skin always means the skin saved with it. The account's own entry used to resolve to the live account skin
// instead, which after switching to another skin was that other skin: clicking "yourself" again changed nothing.
function resolvedSkinUrl(skin: SkinEntry): string | null {
  if (skin.skinUrl) return skin.skinUrl
  return skin.uuid === account.value?.uuid ? (account.value.skinUrl ?? null) : null
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
  try { localStorage.setItem('beja_locker_selected_skin', skin.uuid) } catch { /* ignore */ }
  const model = skin.model ?? 'default'
  const skinUrl = resolvedSkinUrl(skin)
  // keep the cape that is equipped: switching skins must not swap it for the stale one stored with the saved skin
  const capeUrl = equippedCapeUrl.value

  lockerStore.selectSkin({ skinUrl, capeUrl, model })
  syncLocalCosmetics({ skinUrl, capeUrl, skinModel: model })

  // Pushes to the account's real Mojang profile - this is what actually
  // makes vanilla Minecraft (and other players/servers) see the new skin,
  // not just BejaClient's own preview. Locally-uploaded skins only exist as
  // a data: URL, so those go through the raw multipart upload endpoint
  // instead (the same one the official Minecraft Launcher uses for local
  // files) rather than the URL-based endpoint.
  if (skinUrl && account.value?.accessToken) {
    const variant = model === 'slim' ? 'slim' : 'classic'
    try {
      // Mojang answers the URL-based endpoint with 403 for textures.minecraft.net links, so everything that is not
      // already a data: URL is downloaded first and sent as a file like a local skin.
      let dataUrl = skinUrl
      if (!dataUrl.startsWith('data:')) dataUrl = await window.api.players.fetchImage(skinUrl)
      if (dataUrl.startsWith('data:')) {
        const base64 = dataUrl.slice(dataUrl.indexOf(',') + 1)
        await window.api.players.setSkinFile(account.value.accessToken, base64, variant)
      } else {
        await window.api.players.setSkin(account.value.accessToken, skinUrl, variant)
      }
    } catch (e) {
      console.warn('[Skins] set official skin failed', e)
      const reason = String(e)
      showToast({
        title: 'Skin changed here, not on your Minecraft account',
        body: reason.includes('429')
          ? 'Mojang only allows a few skin changes in a short time. Wait a minute, then pick it again.'
          : reason.includes('401') || reason.includes('403')
            ? 'Your Minecraft login has expired. Sign in again from the account menu.'
            : 'Mojang rejected the update. In-game you keep the old skin until it works.',
        variant: 'warning',
      })
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

// Upload-a-local-PNG path - kept separate from the username lookup above.
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
    // Never resize/recompress a skin texture - every pixel maps to a fixed
    // body-part UV, so only the two exact vanilla dimensions are valid.
    const { dataUrl, width, height } = await readSkinFile(file)
    const validSize = (width === 64 && height === 64) || (width === 64 && height === 32)
    if (!validSize) {
      uploadError.value = `Not a valid skin size (${width}×${height}) - must be 64×64 or 64×32`
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

// ── What the page shows ───────────────────────────────────────────────────────
const wornCapeId = computed(() => {
  const url = equippedCapeUrl.value
  if (!url) return 'none'
  return ownedCapes.value.find(c => c.url === url)?.id ?? 'none'
})

const browseSkinUuid    = ref<string | null>(null)
const browseCapeId      = ref<string | null>(null)   // 'none' = no cape
const browseCommunityId = ref<number | null>(null)

const browsedSkin = computed(() => skins.value.find(s => s.uuid === browseSkinUuid.value) ?? heroSkin.value)

interface CapeItem { id: string; url: string | null; alias: string; entry: CapeEntry | null }
const capeItems = computed<CapeItem[]>(() => [
  { id: 'none', url: null, alias: 'No cape', entry: null },
  ...ownedCapes.value.map(c => ({ id: c.id, url: c.url, alias: c.alias, entry: c })),
])
const browsedCape = computed<CapeItem>(() =>
  capeItems.value.find(c => c.id === (browseCapeId.value ?? wornCapeId.value)) ?? capeItems.value[0])

const browsedCommunity = computed(() =>
  communityCapes.value.find(c => c.id === browseCommunityId.value) ?? communityCapes.value[0] ?? null)

interface Row { id: string; name: string; sub: string; worn: boolean; skin?: SkinEntry; capeUrl?: string | null }

function capeSource(c: CapeEntry): string {
  if (c.builtin) return 'BejaClient'
  if (c.shop) return 'Shop'
  if (c.official) return 'Mojang'
  return 'Custom'
}

const rows = computed<Row[]>(() => {
  if (activeTab.value === 'Skins') {
    return skins.value.map(s => ({
      id: s.uuid, name: s.username, sub: s.model === 'slim' ? 'Slim arms' : 'Classic arms',
      worn: s.uuid === selectedUuid.value, skin: s,
    }))
  }
  if (activeTab.value === 'Capes') {
    return capeItems.value.map(c => ({
      id: c.id, name: c.alias, sub: c.entry ? capeSource(c.entry) : '',
      worn: c.id === wornCapeId.value, capeUrl: c.url,
    }))
  }
  return communityCapes.value.map(c => ({
    id: String(c.id), name: c.name, sub: `by ${c.uploader_username}`,
    worn: c.url === equippedCapeUrl.value, capeUrl: c.url,
  }))
})

// ── Search, collection filter, sort, view (same controls as Explore) ──────────
const search   = ref('')
const category = ref('All')
const sort     = ref<'default' | 'name'>('default')
const viewMode = ref<'list' | 'grid'>('list')

const categories = computed<string[]>(() => {
  if (activeTab.value === 'Skins') return ['All', 'Equipped', 'Slim arms', 'Classic arms']
  if (activeTab.value === 'Capes') return ['All', 'Equipped', 'BejaClient', 'Mojang', 'Shop', 'Custom']
  if (activeTab.value === 'Community') return ['All', 'Equipped']
  return ['All', 'Equipped', ...RARITY_ORDER.map(r => RARITIES[r].label)]
})
watch(activeTab, () => { category.value = 'All'; search.value = '' })

const filteredRows = computed<Row[]>(() => {
  const q = search.value.trim().toLowerCase()
  let list = rows.value.filter(r => {
    if (category.value === 'Equipped' && !r.worn) return false
    if (category.value !== 'All' && category.value !== 'Equipped' && r.sub !== category.value) return false
    return !q || r.name.toLowerCase().includes(q)
  })
  if (sort.value === 'name') {
    const pinned = list.filter(r => r.id === 'none')   // "No cape" stays on top
    const rest = list.filter(r => r.id !== 'none').sort((a, b) => a.name.localeCompare(b.name))
    list = [...pinned, ...rest]
  }
  return list
})

const filteredCosmetics = computed<PlayerCosmetic[]>(() => {
  const q = search.value.trim().toLowerCase()
  let list = ownedCosmetics.value.filter(c => {
    if (category.value === 'Equipped' && !c.equipped) return false
    if (category.value !== 'All' && category.value !== 'Equipped' && RARITIES[c.rarity]?.label !== category.value) return false
    return !q || (c.name || '').toLowerCase().includes(q)
  })
  if (sort.value === 'name') list = [...list].sort((a, b) => (a.name || '').localeCompare(b.name || ''))
  return list
})

const selectedRowId = computed(() => {
  if (activeTab.value === 'Skins') return browsedSkin.value?.uuid ?? null
  if (activeTab.value === 'Capes') return browsedCape.value.id
  return browsedCommunity.value ? String(browsedCommunity.value.id) : null
})

// a click wears it; hovering or moving the controller onto it only previews it
function pick(id: string) {
  selectRow(id)
  void equipShown()
}

async function reportRow(id: string) {
  const c = communityCapes.value.find(x => String(x.id) === id)
  if (!c) return
  await reportCommunityCape(c)
  showToast({ title: 'Reported', body: 'Thanks, we will take a look.', variant: 'info' })
}

function selectRow(id: string) {
  if (activeTab.value === 'Skins') browseSkinUuid.value = id
  else if (activeTab.value === 'Capes') browseCapeId.value = id
  else browseCommunityId.value = Number(id)
}

const listLoading = computed(() =>
  activeTab.value === 'Capes' ? capesLoading.value && !ownedCapes.value.length
  : activeTab.value === 'Community' ? communityLoading.value && !communityCapes.value.length
  : false)
const listError = computed(() => activeTab.value === 'Community' ? communityError.value : null)
const emptyText = computed(() =>
  activeTab.value === 'Community' ? 'No community capes yet. Be the first to upload one.' : 'Nothing here yet.')

// The character on the right: the browsed skin (Skins), or the worn skin with the browsed cape (Capes, Community)
const preview = computed(() => {
  const worn = heroSkin.value
  if (activeTab.value === 'Skins') {
    const s = browsedSkin.value
    return {
      skinUrl: s ? resolvedSkinUrl(s) : null, capeUrl: equippedCapeUrl.value, model: (s?.model ?? 'default') as 'default' | 'slim',
      name: s?.username ?? 'No skin', sub: 'Not equipped yet',
      equipped: !!s && s.uuid === selectedUuid.value, canEquip: !!s,
    }
  }
  const base = { skinUrl: worn ? resolvedSkinUrl(worn) : null, model: (worn?.model ?? 'default') as 'default' | 'slim' }
  if (activeTab.value === 'Capes') {
    const c = browsedCape.value
    return { ...base, capeUrl: c.url, name: c.alias, sub: 'Not equipped yet', equipped: c.id === wornCapeId.value, canEquip: true }
  }
  const c = browsedCommunity.value
  return {
    ...base, capeUrl: c?.url ?? null, name: c?.name ?? 'Community capes', sub: c ? `by ${c.uploader_username}` : 'Pick a cape on the left',
    equipped: !!c && c.url === equippedCapeUrl.value, canEquip: !!c,
  }
})

async function equipShown(): Promise<void> {
  if (busy.value || preview.value.equipped) return
  busy.value = true
  try {
    if (activeTab.value === 'Skins') {
      if (browsedSkin.value) await equipSkin(browsedSkin.value)
    } else if (activeTab.value === 'Capes') {
      const c = browsedCape.value
      if (c.entry) await equipCape(c.entry)
      else await equipNoCape()
    } else if (browsedCommunity.value) {
      await equipCommunityCape(browsedCommunity.value)
    }
  } finally {
    busy.value = false
  }
}

watch(activeTab, async (tab) => {
  if (tab === 'Capes' && ownedCapes.value.length === 0 && !capesLoading.value) await loadOwnedCapes()
  if (tab === 'Cosmetics' && !cosmeticsLoading.value) await loadCosmetics()
  if (tab === 'Community' && communityCapes.value.length === 0 && !communityLoading.value) await loadCommunityCapes(true)
})
watch(addModalOpen, async (v) => {
  if (v) { await nextTick(); modalInputRef.value?.focus() }
})

// which saved skin is the equipped one: what was last equipped here, else the entry matching the account's skin,
// else the account's own entry
function initSelectedSkin(): void {
  let remembered: string | null = null
  try { remembered = localStorage.getItem('beja_locker_selected_skin') } catch { /* ignore */ }
  selectedUuid.value =
    (remembered && skins.value.some(s => s.uuid === remembered) ? remembered : null)
    ?? skins.value.find(s => !!account.value?.skinUrl && s.skinUrl === account.value.skinUrl)?.uuid
    ?? account.value?.uuid ?? null
}

onMounted(() => {
  loadSkins()
  initSelectedSkin()
  if (account.value?.skinUrl) {
    const savedCapeUrl = loadLocalCapeUrl()
    lockerStore.selectSkin({
      skinUrl: account.value.skinUrl,
      capeUrl: savedCapeUrl ?? account.value.capeUrl,
      model:   account.value.skinModel ?? 'default',
    })
  }
  // the capes are needed to know which one is worn
  void loadOwnedCapes()
})

// the account can still be loading when the page opens right after a launch: pick the equipped skin once it is there
watch(() => account.value?.uuid, () => {
  loadSkins()
  if (!selectedUuid.value) initSelectedSkin()
})
</script>

<style lang="scss" scoped>
// Same surfaces as Explore and Social: #13141A cards, #232328 hover, #2a2a30 selected, white for the one main action.
.locker {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px 20px;
  gap: 12px;
  overflow: hidden;
}

.search-tab-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
  flex-wrap: wrap;
}

.search-bar {
  display: flex;
  align-items: center;
  background: #13141A;
  border-radius: 12px;
  height: 38px;
  padding: 0 22px;
  flex: 1 1 280px;
  min-width: 180px;
}
.search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.85);
  letter-spacing: 0.03em;
  &::placeholder { color: #666; }
}

.controls-row {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
  align-items: center;
}
.filter-select {
  height: 32px;
  background: none;
  border: none;
  color: #888;
  font-size: 10px;
  letter-spacing: 0.03em;
  padding: 0 8px;
  cursor: pointer;
  outline: none;
  appearance: none;
  transition: color 80ms;
  &:hover, &:focus { color: #bbb; }
  option { background: #111; color: #aaa; }
}
.view-toggle {
  display: flex;
  flex-shrink: 0;
  background: #0a0a0b;
  border: 1px solid rgba(255, 255, 255, 0.08);
}
.view-toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  background: none;
  border: none;
  color: #555;
  cursor: pointer;
  transition: background 80ms, color 80ms;
  & + & { border-left: 1px solid rgba(255, 255, 255, 0.08); }
  &:hover { background: #1a1a1a; color: #aaa; }
  &.active { background: rgba(255, 255, 255, 0.07); color: #d9d9d9; }
}

.chip-row { display: flex; flex-wrap: wrap; gap: 5px; flex-shrink: 0; }
.chip {
  font-size: 9px;
  color: #aaa;
  background: rgba(255, 255, 255, 0.06);
  border: none;
  padding: 2px 8px;
  cursor: pointer;
  letter-spacing: 0.03em;
  transition: background 60ms, color 60ms;
  &:hover { background: rgba(255, 255, 255, 0.1); color: #ccc; }
  &--clear { color: #555; background: transparent; &:hover { color: #888; } }
}

// Sidebar + content, like the Explore body
.locker-body {
  display: flex;
  gap: 16px;
  flex: 1;
  min-height: 0;
}
.locker-main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.categories-sidebar {
  width: 220px;
  flex-shrink: 0;
  background: #13141a;
  border-radius: 8px;
  padding: 18px 14px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: #333 transparent;
}
.categories-title {
  font-size: 12px;
  font-weight: 700;
  color: #F6AE35;
  letter-spacing: 0.06em;
  font-family: 'Minecrafter', 'Mojangles', monospace;
  margin: 0 0 10px;
  padding: 0 10px;
}
.category-row {
  text-align: left;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.6);
  font-size: 12.5px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 100ms, color 100ms;
  &:hover { background: rgba(255, 255, 255, 0.05); color: rgba(255, 255, 255, 0.9); }
  &.active { background: rgba(255, 255, 255, 0.08); color: #fff; }
}

.pill {
  height: 38px;
  padding: 0 22px;
  background: #13141A;
  border: none;
  border-radius: 12px;
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px;
  letter-spacing: 0.02em;
  cursor: pointer;
  transition: background 120ms, color 120ms;
  &:hover { background: #232328; color: rgba(255, 255, 255, 0.9); }
  &.active { background: #2a2a30; color: #fff; }
}

.cta {
  height: 38px;
  padding: 0 22px;
  border: none;
  border-radius: 12px;
  background: #fff;
  color: #0b0b0f;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 120ms;
  &:hover { opacity: 0.88; }
}

// ── Skins, Capes, Community ───────────────────────────────────────────────────
.body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 380px minmax(0, 1fr);
  gap: 16px;
  &--grid { grid-template-columns: 460px minmax(0, 1fr); }
}

// Grid view: the same rows, as tall cards (art on top, name below)
.list--grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  grid-auto-rows: 190px;
  align-content: start;
  gap: 8px;
  .item { height: 100%; }
  .row {
    height: 100%;
    flex-direction: column;
    justify-content: center;
    gap: 8px;
    padding: 12px 8px;
    text-align: center;
  }
  .row-art { width: 64px; height: 92px; }
  .row-text { align-items: center; width: 100%; }
  .row-tag { position: absolute; top: 8px; right: 8px; }
}

.list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 0;
  overflow-y: auto;
  padding-right: 4px;
  scrollbar-width: none;
  &::-webkit-scrollbar { display: none; }
}

.item {
  position: relative;
  flex-shrink: 0;
  &:hover .row-tag, &:focus-within .row-tag { opacity: 0; }
  &:hover .row-action, &:focus-within .row-action { opacity: 1; }
}

.row-action {
  position: absolute;
  top: 50%;
  right: 12px;
  transform: translateY(-50%);
  min-width: 30px;
  height: 30px;
  padding: 0 10px;
  border: none;
  border-radius: 10px;
  background: #2a2a30;
  color: rgba(255, 255, 255, 0.8);
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  opacity: 0;
  transition: opacity 100ms, background 100ms, color 100ms;
  &:hover { background: #34343c; color: #fff; }
  &--text { font-size: 12px; }
}

.row {
  width: 100%;
  flex-shrink: 0;
  height: 76px;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 0 14px 0 10px;
  border: 1px solid transparent;
  border-radius: 12px;
  background: #13141A;
  color: inherit;
  text-align: left;
  cursor: pointer;
  transition: background 80ms, border-color 80ms;
  &:hover { background: #181920; }
  &.sel { background: #232328; border-color: rgba(255, 255, 255, 0.3); }
}

.row-art {
  flex: none;
  width: 46px;
  height: 62px;
  display: flex;
  align-items: center;
  justify-content: center;
  :deep(canvas) { max-height: 100%; max-width: 100%; }
}
.row-head { height: 100%; width: auto; image-rendering: pixelated; }
.row-none { color: rgba(255, 255, 255, 0.3); font-size: 20px; }

.row-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.row-name {
  font-size: 14px;
  color: #fff;
  letter-spacing: 0.02em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.row-sub { font-size: 12px; color: rgba(255, 255, 255, 0.45); }

.row-tag {
  flex-shrink: 0;
  height: 22px;
  padding: 0 9px;
  display: inline-flex;
  align-items: center;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
  font-size: 11px;
  letter-spacing: 0.03em;
}

.more {
  flex-shrink: 0;
  height: 38px;
  border: none;
  border-radius: 12px;
  background: #13141A;
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px;
  cursor: pointer;
  &:hover { background: #232328; color: #fff; }
}

.hint {
  margin: 0;
  padding: 18px 4px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.4);
  &--error { color: #e05050; }
}

// ── Preview ───────────────────────────────────────────────────────────────────
.preview {
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-radius: 12px;
  overflow: hidden;
  background: #13141A;
}
.preview-view { flex: 1; min-height: 0; position: relative; }

.preview-bar {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 22px 20px;
}
.preview-text { min-width: 0; display: flex; flex-direction: column; gap: 4px; }
.preview-name {
  font-size: 20px;
  color: #fff;
  letter-spacing: 0.02em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.preview-sub { font-size: 12px; color: rgba(255, 255, 255, 0.45); }

.btn {
  height: 38px;
  padding: 0 22px;
  border: none;
  border-radius: 12px;
  font-size: 13px;
  letter-spacing: 0.02em;
  cursor: pointer;
  transition: background 120ms, opacity 120ms, color 120ms;

  &--primary {
    background: #fff;
    color: #0b0b0f;
    font-weight: 600;
    &:hover:not(:disabled) { opacity: 0.88; }
    &:disabled { background: #2a2a30; color: rgba(255, 255, 255, 0.5); cursor: default; font-weight: 400; }
  }
  &--ghost {
    background: #232328;
    color: rgba(255, 255, 255, 0.8);
    &:hover { background: #2a2a30; color: #fff; }
  }
  &--wide { width: 100%; }
}

// ── Cosmetics ─────────────────────────────────────────────────────────────────
.cos-grid {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
  grid-auto-rows: 210px;
  gap: 12px;
  align-content: start;
  scrollbar-width: none;
  &::-webkit-scrollbar { display: none; }
  .hint { grid-column: 1 / -1; }
}

.cos-card {
  position: relative;
  display: flex;
  flex-direction: column;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 12px;
  overflow: hidden;
  color: inherit;
  text-align: left;
  cursor: pointer;
  transition: transform 160ms;
  &:hover { transform: translateY(-2px); }
  &.on { outline: 2px solid #fff; outline-offset: -2px; }
}
.cos-view { flex: 1; min-height: 0; position: relative; }
.cos-label {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 9px 12px;
  background: rgba(0, 0, 0, 0.55);
}
.cos-name {
  font-size: 13px;
  color: #fff;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.cos-rarity { font-size: 11px; flex-shrink: 0; letter-spacing: 0.03em; }

// ── Dialogs ───────────────────────────────────────────────────────────────────
.overlay {
  position: fixed;
  inset: 0;
  z-index: 3000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(3px);
}
.dialog {
  width: 360px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 22px 24px;
  border-radius: 12px;
  background: #13141A;
}
.dialog-title { margin: 0; font-size: 16px; color: #fff; }
.dialog-or { margin: 0; text-align: center; font-size: 12px; color: rgba(255, 255, 255, 0.35); }
.dialog-hint { font-size: 11px; color: rgba(255, 255, 255, 0.4); text-align: center; }
.dialog-error { margin: 0; font-size: 12px; color: #e05050; }
.dialog-foot { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }

.field {
  height: 38px;
  padding: 0 14px;
  border: none;
  border-radius: 12px;
  outline: none;
  background: #0f1014;
  color: #fff;
  font-size: 13px;
  &::placeholder { color: rgba(255, 255, 255, 0.35); }
  &:focus { background: #181920; }
}

.dialog-preview { display: flex; align-items: center; gap: 16px; }
.dialog-img {
  width: 72px;
  height: 72px;
  flex-shrink: 0;
  object-fit: contain;
  image-rendering: pixelated;
  border-radius: 8px;
  background: repeating-conic-gradient(#1a1a1a 0% 25%, #111 0% 50%) 50% / 10px 10px;
}
.dialog-col { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 8px; }

.seg {
  display: flex;
  gap: 6px;
  button {
    flex: 1;
    height: 32px;
    border: none;
    border-radius: 10px;
    background: #232328;
    color: rgba(255, 255, 255, 0.6);
    font-size: 12px;
    cursor: pointer;
    &:hover { color: #fff; }
    &.on { background: #2a2a30; color: #fff; outline: 1px solid rgba(255, 255, 255, 0.3); outline-offset: -1px; }
  }
}

.modal-enter-active { transition: opacity 150ms ease, transform 150ms ease; }
.modal-leave-active { transition: opacity 100ms ease; }
.modal-enter-from   { opacity: 0; transform: scale(0.97); }
.modal-leave-to     { opacity: 0; }
</style>
