<template>
  <div class="profiles-page">

    <!-- ── Detail view (slide in) ──────────────────────────── -->
    <Transition name="detail-slide">
      <ProfileDetailView
        v-if="selectedProfile"
        :key="selectedProfile.id"
        :profile="selectedProfile"
        :has-prev="hasPrevProfile"
        :has-next="hasNextProfile"
        @back="selectedProfile = null"
        @settings="openWizard"
        @deleted="onDetailDeleted"
        @prev="goPrevProfile"
        @next="goNextProfile"
      />
    </Transition>

    <!-- ── Grid view ────────────────────────────────────────── -->
    <template v-if="!selectedProfile">
    <div v-if="pickMode" class="pick-banner">
      <span>Choose the profile to launch with</span>
      <button class="pick-cancel" @click="cancelPick">Cancel</button>
    </div>

    <!-- Toolbar: search + tabs + import banner, all in one row -->
    <div class="profiles-toolbar">
      <div class="toolbar-search">
        <input
          v-model="search"
          class="toolbar-search-input"
          placeholder="Search profiles..."
          @keydown.enter="refresh"
        />
      </div>

      <div class="toolbar-actions">
      <button
        v-for="tab in TABS"
        :key="tab.id"
        class="toolbar-tab"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >{{ tab.label }}</button>

      <button
        class="toolbar-tab"
        :class="{ active: selectMode }"
        title="Select several profiles to delete them at once"
        @click="toggleSelectMode"
      >{{ selectMode ? 'Done' : 'Select' }}</button>

      <button class="toolbar-optifine-btn" title="Create an OptiFine profile" @click="optifineWizardOpen = true">
        <img :src="optifineIcon" alt="OptiFine" />
      </button>

      <button class="toolbar-optifine-btn" title="Check for mod updates across all profiles" @click="updateCenterOpen = true">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
        </svg>
      </button>

      <div v-if="detected.length" class="import-banner" @click="importReviewOpen = true">
        <div class="import-banner-icons">
          <img v-if="detectedSources.has('modrinth_app')" :src="modrinthIcon" class="import-banner-icon" alt="Modrinth" />
          <img v-if="detectedSources.has('lunar')" :src="lunarIcon" class="import-banner-icon" alt="Lunar Client" />
          <img v-if="detectedSources.has('curseforge')" :src="curseforgeIcon" class="import-banner-icon" alt="CurseForge" />
        </div>
        <span class="import-banner-label">Import profiles</span>
        <button class="import-banner-btn" @click.stop="importReviewOpen = true">View</button>
      </div>

      <button v-if="detected.length" class="toolbar-chevron" title="Import profiles" @click="importReviewOpen = true">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="9 6 15 12 9 18"/>
        </svg>
      </button>
      </div>

      <!-- hosting partner: a bar under the buttons, as wide as they are, not under the search -->
      <SponsorCard class="profiles-sponsor" />
    </div>

    <!-- Skeleton loading -->
    <div v-if="loading" class="profiles-grid">
      <Skeleton v-for="i in 6" :key="i" variant="block" height="260px" radius="14px" />
    </div>

    <!-- Empty state -->
    <div v-else-if="filteredProfiles.length === 0 && search === ''" class="empty-state">
      <p>No profiles yet.</p>
      <button class="new-profile-btn" @click="openWizard">Create your first profile</button>
    </div>

    <!-- Grid -->
    <TransitionGroup v-else tag="div" name="card-pop" class="profiles-grid">
      <!-- Profile cards -->
      <div v-for="profile in filteredProfiles" :key="profile.id" class="profile-card-wrap">
        <div
          class="profile-card"
          :class="{
            'profile-card--active': profile.id === activeProfileId && !selectMode,
            'profile-card--selected': selectMode && selectedIds.has(profile.id),
          }"
          @click="onCardClick(profile)"
          @contextmenu="openProfileMenu($event, profile)"
        >
          <div
            class="card-banner"
            :style="profile.backgroundUrl ? { backgroundImage: `url(${profile.backgroundUrl})` } : {}"
          />
          <div class="card-icon-panel">
            <div class="card-icon">
              <img :src="profile.imageUrl || bejaLogo" :class="{ ph: !profile.imageUrl }" alt="" />
            </div>
          </div>
          <button v-if="!selectMode" class="card-delete-btn" title="Delete profile" @click.stop="confirmDelete(profile)">
            <img :src="iconRemove" class="card-btn-icon" />
          </button>
          <span v-else class="card-check" :class="{ on: selectedIds.has(profile.id) }" aria-hidden="true">
            <svg v-if="selectedIds.has(profile.id)" viewBox="0 0 12 12" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="square"><path d="M2.5 6.5l2.5 2.5 4.5-5.5" /></svg>
          </span>
        </div>
        <span class="card-name-label">{{ profile.name }}</span>
      </div>

      <!-- New profile slot -->
      <div v-if="!selectMode" key="new-profile" class="profile-card-wrap">
        <button class="profile-card profile-card--new" @click="openWizard">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19" />
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
        <span class="card-name-label card-name-label--muted">New Profile</span>
      </div>
    </TransitionGroup>

    <!-- Bulk selection bar -->
    <Transition name="bulk-bar">
      <div v-if="selectMode" class="bulk-bar">
        <span class="bulk-count">{{ selectedIds.size }} selected</span>
        <button class="bulk-btn" @click="allSelected ? clearSelection() : selectAllVisible()">
          {{ allSelected ? 'Clear' : 'Select all' }}
        </button>
        <button class="bulk-btn bulk-btn--danger" :disabled="!selectedIds.size" @click="bulkDeleteOpen = true">
          Delete{{ selectedIds.size ? ` (${selectedIds.size})` : '' }}
        </button>
        <button class="bulk-btn" @click="exitSelectMode">Cancel</button>
      </div>
    </Transition>

    <!-- Bulk delete confirm modal -->
    <Teleport to="body">
      <Transition name="modal-fade">
        <div v-if="bulkDeleteOpen" class="modal-overlay" @click.self="bulkDeleteOpen = false">
          <div class="modal">
            <p class="modal-title">Delete {{ selectedProfiles.length }} {{ selectedProfiles.length === 1 ? 'profile' : 'profiles' }}?</p>
            <ul class="bulk-names">
              <li v-for="p in selectedProfiles.slice(0, 6)" :key="p.id">{{ p.name }}</li>
              <li v-if="selectedProfiles.length > 6" class="bulk-names-more">and {{ selectedProfiles.length - 6 }} more</li>
            </ul>
            <p class="modal-body">The profiles and their mods and settings are removed. This cannot be undone.</p>
            <div class="modal-actions">
              <button class="modal-btn modal-btn--cancel" :disabled="bulkDeleting" @click="bulkDeleteOpen = false">Cancel</button>
              <button class="modal-btn modal-btn--danger" :disabled="bulkDeleting" @click="doBulkDelete">
                {{ bulkDeleting ? 'Deleting…' : 'Delete' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Delete confirm modal -->
    <Teleport to="body">
      <Transition name="modal-fade">
        <div v-if="deleteTarget" class="modal-overlay" @click.self="deleteTarget = null">
          <div class="modal">
            <p class="modal-title">Delete "{{ deleteTarget.name }}"?</p>
            <p class="modal-body">This cannot be undone.</p>
            <div class="modal-actions">
              <button class="modal-btn modal-btn--cancel" @click="deleteTarget = null">Cancel</button>
              <button class="modal-btn modal-btn--danger" @click="doDelete">Delete</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- External-launcher import review modal -->
    <Teleport to="body">
      <Transition name="modal-fade">
        <div v-if="importReviewOpen" class="modal-overlay" @click.self="importReviewOpen = false">
          <div class="modal modal--import">
            <p class="modal-title">Import profiles</p>
            <p class="modal-body">Detected on this machine. Mods/saves stay where they are - BejaClient just installs the matching Minecraft version and plays from the same folder.</p>

            <div class="import-list">
              <div v-for="p in detected" :key="p.id" class="import-row">
                <div class="import-row-icon">
                  <img v-if="p.iconPath" :src="convertFileSrc(p.iconPath)" alt="" />
                  <div v-else class="import-row-icon-ph">{{ p.name[0] }}</div>
                </div>
                <div class="import-row-info">
                  <span class="import-row-name">{{ p.name }}</span>
                  <span class="import-row-meta">
                    {{ p.version }} · {{ LOADER_NAMES[p.loader] ?? p.loader }} · {{ p.modCount }} mods
                    <span class="import-row-source">{{ sourceLabel(p.source) }}</span>
                  </span>
                </div>
                <button
                  class="import-row-btn"
                  :disabled="importingIds.has(p.id)"
                  @click="importOne(p)"
                >{{ importingIds.has(p.id) ? 'Importing…' : 'Import' }}</button>
              </div>
            </div>

            <div class="modal-actions">
              <button class="modal-btn modal-btn--cancel" @click="importReviewOpen = false">Close</button>
              <button class="modal-btn modal-btn--confirm" :disabled="!detected.length || importingIds.size > 0" @click="importAll">Import all</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <OptifineWizard v-model="optifineWizardOpen" @created="onOptifineCreated" />
    <ModUpdateCenterModal v-model="updateCenterOpen" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useLauncherStore } from '../../store/launcherStore'
import type { LaunchProfile, DetectedProfile } from '../../types'
import ProfileDetailView from '../../components/profiles/ProfileDetailView.vue'
import OptifineWizard from '../../components/profiles/OptifineWizard.vue'
import ModUpdateCenterModal from '../../components/common/ModUpdateCenterModal.vue'
import Skeleton from '../../components/common/Skeleton.vue'
import SponsorCard from '../../components/home/SponsorCard.vue'
import { openContextMenu } from '../../composables/useContextMenu'
import { showToast } from '../../composables/useToasts'

import iconRemove  from '../../assets/icons8-remove-24.png'
import bejaLogo    from '../../assets/bc-logo-new.png'
import modrinthIcon    from '../../assets/modrinth.png'
import curseforgeIcon  from '../../assets/curseforge-icon.png'
import lunarIcon       from '../../assets/lunar-icon.png'
import optifineIcon    from '../../assets/optifine-icon.png'

const LOADER_NAMES: Record<string, string> = {
  fabric: 'Fabric', forge: 'Forge', neoforge: 'NeoForge', quilt: 'Quilt', optifine: 'OptiFine', vanilla: 'Vanilla',
}

function cardMetaParts(profile: LaunchProfile): string[] {
  if (!profile.version) return []
  const flavor = profile.useBejaClient ? 'BejaClient' : (LOADER_NAMES[profile.loader] ?? profile.loader)
  const parts = [profile.version, flavor]
  const mods = modCounts.value[profile.id]
  if (mods !== undefined) parts.push(`${mods} mods`)
  return parts
}

const TABS = [
  { id: 'all', label: 'All' },
] as const

type TabId = typeof TABS[number]['id']

const store = useLauncherStore()

const activeTab      = ref<TabId>('all')
const search         = ref('')
const loading        = ref(false)
const refreshing     = ref(false)
const modCounts      = ref<Record<string, number>>({})
const deleteTarget   = ref<LaunchProfile | null>(null)
const selectedProfile = ref<LaunchProfile | null>(null)

const detected          = ref<DetectedProfile[]>([])
const importReviewOpen  = ref(false)
const importingIds      = ref<Set<string>>(new Set())

const optifineWizardOpen = ref(false)
const updateCenterOpen = ref(false)
function onOptifineCreated(profile: LaunchProfile): void {
  showToast({ title: `Created "${profile.name}"`, body: 'Finish the OptiFine installer window, then hit Play.', variant: 'success' })
}

const SOURCE_NAMES: Record<string, string> = {
  curseforge: 'CurseForge', lunar: 'Lunar Client', modrinth_app: 'Modrinth App',
}
function sourceLabel(source: string): string { return SOURCE_NAMES[source] ?? source }
const detectedSources = computed(() => new Set(detected.value.map(p => p.source)))

async function loadDetected(): Promise<void> {
  try {
    detected.value = await window.api.profiles.detectExternal()
  } catch {
    detected.value = []
  }
}

async function importOne(p: DetectedProfile): Promise<void> {
  importingIds.value = new Set([...importingIds.value, p.id])
  try {
    await window.api.profiles.importExternal(p.id)
    detected.value = detected.value.filter(d => d.id !== p.id)
    await store.loadProfiles()
    await loadModCounts()
    showToast({ title: `Imported "${p.name}"`, body: `From ${sourceLabel(p.source)}`, variant: 'success' })
  } catch (e) {
    showToast({ title: `Failed to import "${p.name}"`, body: String(e), variant: 'error' })
  } finally {
    importingIds.value = new Set([...importingIds.value].filter(id => id !== p.id))
  }
}

async function importAll(): Promise<void> {
  for (const p of [...detected.value]) {
    await importOne(p)
  }
  importReviewOpen.value = false
}

watch(selectedProfile, (p) => { store.editingProfileName = p?.name ?? null })
onUnmounted(() => { store.editingProfileName = null })

const activeProfileId = computed(() => store.activeProfile?.id ?? null)

const filteredProfiles = computed(() => {
  const q = search.value.toLowerCase()
  return store.profiles
    .filter(p => !q || p.name.toLowerCase().includes(q) || p.version.includes(q))
})

// Counts only need the folder listing (no jar is opened), and each card fills in as soon as its own count arrives.
async function loadModCounts(): Promise<void> {
  await Promise.all(
    store.profiles.map(async p => {
      try {
        modCounts.value = { ...modCounts.value, [p.id]: await window.api.mods.countEnabled(p.id) }
      } catch {
        modCounts.value = { ...modCounts.value, [p.id]: 0 }
      }
    })
  )
}

async function refresh(): Promise<void> {
  refreshing.value = true
  try {
    await store.loadProfiles()
    await loadModCounts()
  } finally {
    refreshing.value = false
  }
}

function openDetail(profile: LaunchProfile): void {
  selectedProfile.value = profile
}

const selectedIndex = computed(() => {
  if (!selectedProfile.value) return -1
  return filteredProfiles.value.findIndex(p => p.id === selectedProfile.value!.id)
})
const hasPrevProfile = computed(() => selectedIndex.value > 0)
const hasNextProfile = computed(() =>
  selectedIndex.value >= 0 && selectedIndex.value < filteredProfiles.value.length - 1)

function goPrevProfile(): void {
  if (hasPrevProfile.value) selectedProfile.value = filteredProfiles.value[selectedIndex.value - 1]
}

function goNextProfile(): void {
  if (hasNextProfile.value) selectedProfile.value = filteredProfiles.value[selectedIndex.value + 1]
}

async function onDetailDeleted(id: string): Promise<void> {
  selectedProfile.value = null
  await window.api.profiles.delete(id)
  await store.loadProfiles()
  delete modCounts.value[id]
}

// ── Select several profiles and delete them together ─────────────────────────
const selectMode     = ref(false)
const selectedIds    = ref<Set<string>>(new Set())
const bulkDeleteOpen = ref(false)
const bulkDeleting   = ref(false)

const selectedProfiles = computed(() => store.profiles.filter(p => selectedIds.value.has(p.id)))
const allSelected = computed(() =>
  filteredProfiles.value.length > 0 && filteredProfiles.value.every(p => selectedIds.value.has(p.id)))

function toggleSelectMode(): void {
  if (selectMode.value) exitSelectMode()
  else selectMode.value = true
}

function exitSelectMode(): void {
  selectMode.value = false
  selectedIds.value = new Set()
}

function toggleSelected(id: string): void {
  const next = new Set(selectedIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedIds.value = next
}

function selectAllVisible(): void {
  selectedIds.value = new Set([...selectedIds.value, ...filteredProfiles.value.map(p => p.id)])
}

function clearSelection(): void {
  selectedIds.value = new Set()
}

// Pick mode: arrived via the hub's launch arrow (?pick=1). A click picks the profile and returns to the hub.
const route = useRoute()
const router = useRouter()
const pickMode = computed(() => route.name === 'profiles' && route.query.pick === '1')

async function pickProfile(profile: LaunchProfile): Promise<void> {
  await store.setActiveProfile(profile.id)
  void router.push({ name: 'home' })
}

function cancelPick(): void {
  void router.push({ name: 'home' })
}

function onCardClick(profile: LaunchProfile): void {
  if (pickMode.value) void pickProfile(profile)
  else if (selectMode.value) toggleSelected(profile.id)
  else openDetail(profile)
}

async function doBulkDelete(): Promise<void> {
  const ids = [...selectedIds.value]
  if (!ids.length || bulkDeleting.value) return
  bulkDeleting.value = true
  let failed = 0
  for (const id of ids) {
    try {
      await window.api.profiles.delete(id)
      delete modCounts.value[id]
    } catch {
      failed++
    }
  }
  await store.loadProfiles()
  bulkDeleting.value = false
  bulkDeleteOpen.value = false
  exitSelectMode()
  const done = ids.length - failed
  showToast({
    title: failed ? `Deleted ${done} of ${ids.length} profiles` : `Deleted ${done} ${done === 1 ? 'profile' : 'profiles'}`,
    body: failed ? `${failed} could not be deleted.` : undefined,
    variant: failed ? 'warning' : 'success',
  })
}

function onSelectKey(e: KeyboardEvent): void {
  if (e.key === 'Escape' && selectMode.value && !bulkDeleteOpen.value) exitSelectMode()
}
onMounted(() => window.addEventListener('keydown', onSelectKey))
onUnmounted(() => window.removeEventListener('keydown', onSelectKey))

function confirmDelete(profile: LaunchProfile): void {
  deleteTarget.value = profile
}

async function doDelete(): Promise<void> {
  if (!deleteTarget.value) return
  const id = deleteTarget.value.id
  deleteTarget.value = null
  await window.api.profiles.delete(id)
  await store.loadProfiles()
  delete modCounts.value[id]
}

function openWizard(): void {
  store.wizardOpen = true
}

async function launchProfile(profile: LaunchProfile): Promise<void> {
  await store.setActiveProfile(profile.id)
  await store.launch()
}

async function duplicateProfile(profile: LaunchProfile): Promise<void> {
  const { id, createdAt, lastPlayed, playtimeMs, ...rest } = profile
  await store.createProfile({ ...rest, name: `${profile.name} (copy)` })
  await loadModCounts()
  showToast({ title: `Duplicated "${profile.name}"`, variant: 'success' })
}

async function exportProfileZip(profile: LaunchProfile): Promise<void> {
  const result = await window.api.profiles.exportZip(profile.id, true)
  if (!result) return // dialog cancelled
  if (!result.ok) {
    if (result.error === 'cancelled') return
    showToast({ title: `Failed to export "${profile.name}"`, body: result.error, variant: 'error' })
    return
  }
  showToast({ title: `Exported "${profile.name}"`, body: `${result.mods ?? 0} mod(s), no BejaClient required`, variant: 'success' })
}

async function exportProfileCurseforge(profile: LaunchProfile): Promise<void> {
  const result = await window.api.profiles.exportCurseforge(profile.id)
  if (!result) return // dialog cancelled
  if (!result.ok) {
    if (result.error === 'cancelled') return
    showToast({ title: `Failed to export "${profile.name}"`, body: result.error, variant: 'error' })
    return
  }
  showToast({ title: `Exported "${profile.name}"`, body: `${result.mods ?? 0} mod(s) as CurseForge modpack`, variant: 'success' })
}

function openProfileMenu(event: MouseEvent, profile: LaunchProfile): void {
  openContextMenu(event, [
    { label: 'Launch',   icon: 'play',   onClick: () => launchProfile(profile) },
    { label: 'Edit',     icon: 'pencil', onClick: () => openDetail(profile) },
    { label: 'Duplicate', icon: 'copy',  onClick: () => duplicateProfile(profile) },
    { label: 'Export as ZIP (for others)', icon: 'download', onClick: () => exportProfileZip(profile) },
    { label: 'Export as CurseForge Modpack', icon: 'download', onClick: () => exportProfileCurseforge(profile) },
    { separator: true, label: '' },
    { label: 'Select several…', icon: 'check', onClick: () => { selectMode.value = true; toggleSelected(profile.id) } },
    { label: 'Delete', icon: 'trash', danger: true, onClick: () => confirmDelete(profile) },
  ])
}

onMounted(async () => {
  loading.value = true
  try {
    await store.loadProfiles()
  } finally {
    loading.value = false
  }
  void loadModCounts()   // not awaited: the cards show right away, the counts fill in
  loadDetected()
})
</script>

<style lang="scss" scoped>
@import '../../styles/motion';

.profiles-page {
  display: flex;
  flex-direction: column;
  gap: 14px;
  height: 100%;
  overflow-y: auto;
  padding: 2px 4px 16px;
  position: relative;
}

// hosting partner slot: second row of the toolbar, only under the buttons
.profiles-sponsor { grid-column: 2; contain: inline-size; max-width: 300px; justify-self: end; }   // never widens the column, only fills it
.toolbar-actions { display: flex; align-items: center; gap: 14px; }

// ── Tabs ──────────────────────────────────────────────────────────────────────
// ── Toolbar: search + tabs + import banner, one row ────────────────────────────
.pick-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
  padding: 10px 14px;
  background: $surface;
  border: 1px solid $border-strong;
  border-radius: $radius-sm;
  color: $text-primary;
  font-size: 13px;
  font-weight: 600;
}
.pick-cancel {
  padding: 4px 12px;
  background: none;
  border: 1px solid $border-strong;
  border-radius: $radius-sm;
  color: $text-secondary;
  cursor: pointer;
  &:hover { color: $text-primary; border-color: rgba(255, 255, 255, 0.25); }
}

.profiles-toolbar {
  display: grid;
  grid-template-columns: minmax(240px, 1fr) auto;
  align-items: center;
  gap: 10px 14px;
}

.toolbar-search {
  display: flex;
  align-items: center;
  background: #13141A;
  border-radius: $radius-btn;
  height: 38px;
  padding: 0 20px;
  flex: 1 1 360px;
  min-width: 200px;
}

.toolbar-search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.9);
  letter-spacing: 0.02em;

  &::placeholder { color: rgba(255, 255, 255, 0.35); }
}

.toolbar-tab {
  height: 38px;
  padding: 0 22px;
  background: #13141A;
  border: none;
  border-radius: $radius-btn;
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px;
  cursor: pointer;
  letter-spacing: 0.02em;
  white-space: nowrap;
  flex-shrink: 0;
  transition: background 120ms, color 120ms;

  &:hover { background: #232328; color: rgba(255, 255, 255, 0.9); }
  &.active { background: #2a2a30; color: #fff; }
}

.toolbar-chevron {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  flex-shrink: 0;
  margin-left: auto;
  transition: color $transition;
  &:hover { color: #fff; }
}

.toolbar-optifine-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  padding: 5px;
  background: #13141A;
  border: none;
  border-radius: $radius-btn;
  cursor: pointer;
  flex-shrink: 0;
  overflow: hidden;
  transition: background $transition;

  img { width: 100%; height: 100%; object-fit: cover; border-radius: $radius-sm; }

  &:hover { background: #232328; }
}

// ── Grid ──────────────────────────────────────────────────────────────────────
.profiles-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  align-items: start;
  gap: 22px 14px;
}

// ── Card enter/leave/reorder (filtering, delete) ───────────────────────────────
.card-pop-enter-active,
.card-pop-leave-active {
  transition: opacity 180ms ease, transform 180ms cubic-bezier(0.25, 1, 0.5, 1);
}
.card-pop-move { transition: transform 220ms cubic-bezier(0.25, 1, 0.5, 1); }
.card-pop-enter-from { opacity: 0; transform: scale(0.94) translateY(6px); }
.card-pop-leave-to   { opacity: 0; transform: scale(0.94); }

// ── Card ──────────────────────────────────────────────────────────────────────
// Single rounded surface (background image when the profile has one, flat
// otherwise) with just a centered icon - the name lives outside the card as
// its own label, not in an info strip glued to the artwork.
.profile-card-wrap {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.profile-card {
  position: relative;
  aspect-ratio: 515 / 260;
  background: #191a1f;
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: $radius-lg;
  overflow: hidden;
  cursor: pointer;
  transition: border-color $transition, box-shadow $transition, transform $transition;

  &:hover {
    border-color: rgba(255, 255, 255, 0.2);
    box-shadow: 0 10px 26px rgba(0, 0, 0, 0.4);
    transform: translateY(-1px);

    .card-delete-btn { opacity: 0.85; }
  }

  &--active {
    border-color: rgba(255, 255, 255, 0.7);
    box-shadow: 0 0 14px rgba(255, 255, 255, 0.18);
  }

  &--new {
    border: 2px dashed rgba(255, 255, 255, 0.14);
    color: rgba(255, 255, 255, 0.22);
    transition: border-color $transition, color $transition, background $transition;

    &:hover {
      border-color: rgba(255, 255, 255, 0.32);
      color: rgba(255, 255, 255, 0.5);
      background: rgba(255, 255, 255, 0.03);
      transform: none;
      box-shadow: none;
    }
  }
}

.card-name-label {
  display: block;
  font-size: 15px;
  font-weight: 600;
  color: #fff;
  padding-left: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;

  &--muted { color: rgba(255, 255, 255, 0.4); }
}

// Right side: the profile's artwork, full-bleed behind the icon panel.
.card-banner {
  position: absolute;
  inset: 0;
  background: #191a1f;
  background-size: cover;
  background-position: center;
}

// Left side: solid panel sliced by a diagonal edge, icon anchored inside it.
.card-icon-panel {
  position: absolute;
  inset: 0;
  width: 46%;
  background: #101114;
  clip-path: polygon(0 0, 100% 0, 74% 100%, 0 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  padding-right: 14%;
}

.card-icon {
  position: relative;
  width: 68px;
  height: 68px;
  border-radius: $radius;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;

  img { width: 100%; height: 100%; object-fit: cover; }
  img.ph { object-fit: contain; padding: 16%; opacity: 0.4; box-sizing: border-box; }
}

.card-maint-badge {
  position: absolute;
  top: 8px;
  left: 8px;
  padding: 3px 7px;
  background: rgba(0, 0, 0, 0.6);
  border: 1px solid rgba(255, 100, 100, 0.5);
  border-radius: $radius-sm;
  color: #ff8a8a;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  pointer-events: none;
}

.profile-card--selected {
  border-color: #fff;
  box-shadow: 0 0 0 1px #fff, 0 10px 26px rgba(0, 0, 0, 0.4);
}

.card-check {
  position: absolute;
  top: 10px;
  right: 10px;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.55);
  border: 1px solid rgba(255, 255, 255, 0.5);
  border-radius: 0;
  color: #0a0a0b;

  &.on { background: #fff; border-color: #fff; }
}

// Bar that appears while several profiles are being picked.
.bulk-bar {
  position: fixed;
  bottom: 28px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 40;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px 8px 18px;
  background: #13141A;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: $radius-btn;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.55);
}

.bulk-count {
  margin-right: 8px;
  font-size: 13px;
  color: #fff;
  letter-spacing: 0.02em;
  white-space: nowrap;
}

.bulk-btn {
  height: 34px;
  padding: 0 16px;
  border: none;
  border-radius: $radius-btn;
  background: #232328;
  color: rgba(255, 255, 255, 0.8);
  font-size: 12px;
  cursor: pointer;
  letter-spacing: 0.02em;
  white-space: nowrap;
  transition: background 120ms, color 120ms, opacity 120ms;

  &:hover:not(:disabled) { background: #2f2f36; color: #fff; }
  &:disabled { opacity: 0.3; cursor: not-allowed; }

  &--danger {
    background: $error;
    color: #fff;
    &:hover:not(:disabled) { background: color-mix(in srgb, #{$error} 85%, #fff); }
  }
}

.bulk-bar-enter-active, .bulk-bar-leave-active { transition: opacity 160ms ease, transform 160ms ease; }
.bulk-bar-enter-from, .bulk-bar-leave-to { opacity: 0; transform: translate(-50%, 10px); }

.bulk-names {
  margin: 0 0 10px;
  padding: 0 0 0 18px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.85);
  line-height: 1.6;
}
.bulk-names-more { list-style: none; margin-left: -18px; color: rgba(255, 255, 255, 0.45); }

.card-delete-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  background: rgba(0, 0, 0, 0.45);
  border: none;
  border-radius: $radius-sm;
  cursor: pointer;
  padding: 5px;
  opacity: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: opacity $transition, background $transition;

  &:hover {
    opacity: 1 !important;
    background: rgba(224, 80, 80, 0.5);
  }
}

.card-btn-icon {
  width: 12px;
  height: 12px;
  object-fit: contain;
  filter: brightness(0) invert(1);
}

// ── Empty state ───────────────────────────────────────────────────────────────
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 48px;
  color: $muted;
  font-size: 13px;
}

.new-profile-btn {
  padding: 8px 20px;
  border-radius: 12px;
  background: $primary;
  border: none;
  color: #000;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: opacity $transition, transform 140ms $ease-out;

  &:hover { opacity: 0.85; transform: translateY(-1px); }
}

// ── Delete modal ──────────────────────────────────────────────────────────────
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  @extend %glass-panel;
  background: rgba(17, 17, 17, 0.78);
  padding: 24px;
  min-width: 280px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.modal-fade-enter-active .modal,
.modal-fade-leave-active .modal {
  transition: transform 180ms $ease-out;
}
.modal-fade-enter-from .modal,
.modal-fade-leave-to .modal {
  transform: scale(0.95) translateY(6px);
}

.modal-title {
  font-size: 14px;
  font-weight: 700;
  color: $text-primary;
}

.modal-body {
  font-size: 12px;
  color: $muted;
}

.modal-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 4px;
}

.modal-btn {
  padding: 7px 16px;
  border: none;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity $transition;

  &:hover { opacity: 0.85; }

  &--cancel {
    background: rgba(255, 255, 255, 0.1);
    color: $text-primary;
  }

  &--danger {
    background: $error;
    color: #fff;
  }

  &--confirm {
    background: var(--accent, #{$primary});
    color: #fff;
    &:disabled { opacity: 0.4; cursor: not-allowed; }
  }
}

// ── External-launcher import ────────────────────────────────────────────────
.import-banner {
  display: flex;
  align-items: center;
  gap: 14px;
  height: 38px;
  padding: 0 6px 0 20px;
  flex-shrink: 0;
  cursor: pointer;
  border-radius: $radius-btn;
  background: #13141A;
  transition: background $transition;
  &:hover { background: #232328; }
}

.import-banner-icons {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.import-banner-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.import-banner-label {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.85);
  letter-spacing: 0.02em;
  white-space: nowrap;
}

.import-banner-btn {
  flex-shrink: 0;
  padding: 8px 16px;
  background: #2a2a30;
  color: #fff;
  border: none;
  border-radius: $radius-btn;
  font-family: 'Minecrafter', 'Plus Jakarta Sans', sans-serif;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  font-size: 10px;
  cursor: pointer;
  transition: background $transition;
  &:hover { background: #38383f; }
}

.modal--import {
  width: 460px;
  max-width: 92vw;
  max-height: 80vh;
}

.import-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 50vh;
  overflow-y: auto;
  margin: 4px 0;
}

.import-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.import-row-icon {
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  overflow: hidden;

  img { width: 100%; height: 100%; object-fit: cover; }
}

.import-row-icon-ph {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.06);
  color: $muted;
  font-size: 13px;
  font-weight: 700;
}

.import-row-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.import-row-name {
  font-size: 12.5px;
  font-weight: 600;
  color: $text-primary;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.import-row-meta {
  font-size: 10.5px;
  color: $muted;
  display: flex;
  align-items: center;
  gap: 6px;
}

.import-row-source {
  font-size: 8.5px;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: #999;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  padding: 1px 6px;
}

.import-row-btn {
  flex-shrink: 0;
  padding: 6px 14px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: 12px;
  color: $text-primary;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: background $transition;
  &:hover:not(:disabled) { background: rgba(255, 255, 255, 0.16); }
  &:disabled { opacity: 0.5; cursor: wait; }
}

// ── Detail slide transition ───────────────────────────────────────────────────
.detail-slide-enter-active {
  transition: opacity 180ms ease, transform 180ms $ease-out;
}
.detail-slide-leave-active {
  transition: opacity 140ms ease, transform 140ms $ease-in;
  position: absolute;
  inset: 0;
}
.detail-slide-enter-from {
  opacity: 0;
  transform: translateX(18px);
}
.detail-slide-leave-to {
  opacity: 0;
  transform: translateX(18px);
}
</style>
