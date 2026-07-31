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
    <!-- Tabs -->
    <div class="profiles-tabs">
      <button
        v-for="tab in TABS"
        :key="tab.id"
        :ref="(el) => setTabBtnRef(tab.id, el as HTMLElement | null)"
        class="tab-btn"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
      </button>
      <span class="tab-indicator" :style="indicatorStyle" />
    </div>

    <!-- Search bar -->
    <div class="search-bar">
      <input
        v-model="search"
        class="search-input"
        placeholder="Search profile..."
        @keydown.enter="refresh"
      />
      <div class="search-actions">
        <span class="search-sep" />
        <button class="search-icon-btn" title="Refresh" @click="refresh">
          <img :src="iconRefresh" class="search-icon" :class="{ spinning: refreshing }" />
        </button>
        <span class="search-sep" />
        <span class="search-icon-btn non-interactive">
          <img :src="iconSearch" class="search-icon" />
        </span>
      </div>
    </div>

    <!-- Skeleton loading -->
    <div v-if="loading" class="profiles-grid">
      <Skeleton v-for="i in 6" :key="i" variant="block" height="260px" radius="3px" />
    </div>

    <!-- Empty state -->
    <div v-else-if="filteredProfiles.length === 0 && search === ''" class="empty-state">
      <p>No profiles yet.</p>
      <button class="new-profile-btn" @click="openWizard">Create your first profile</button>
    </div>

    <!-- Grid -->
    <TransitionGroup v-else tag="div" name="card-pop" class="profiles-grid">
      <!-- Profile cards -->
      <div
        v-for="profile in filteredProfiles"
        :key="profile.id"
        class="profile-card"
        :class="{ 'profile-card--active': profile.id === activeProfileId }"
        @click="openDetail(profile)"
        @contextmenu="openProfileMenu($event, profile)"
      >
        <div class="card-media">
          <img class="card-media-img" :src="profile.backgroundUrl || bgPlaceholder" alt="" />
          <div class="card-icon">
            <img :src="profile.imageUrl || bejaLogo" :class="{ ph: !profile.imageUrl }" alt="" />
          </div>
          <span v-if="profile.useBejaClient" class="card-maint-badge" title="BejaClient versions are under maintenance. We're working at full speed to bring them back. Sorry!">Under Maintenance</span>
          <button class="card-delete-btn" title="Delete profile" @click.stop="confirmDelete(profile)">
            <img :src="iconRemove" class="card-btn-icon" />
          </button>
        </div>

        <div class="card-info">
          <span class="card-name">{{ profile.name }}</span>
          <div class="card-extra">
            <div class="card-extra-inner">
              <span v-if="profile.description" class="card-desc">{{ profile.description }}</span>
              <span class="card-meta">{{ cardMetaParts(profile).join(' · ') || 'No version yet' }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- New profile slot -->
      <button key="new-profile" class="profile-card profile-card--new" @click="openWizard">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        <span class="new-card-label">New Profile</span>
      </button>
    </TransitionGroup>

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
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useLauncherStore } from '../../store/launcherStore'
import type { LaunchProfile } from '../../types'
import ProfileDetailView from '../../components/profiles/ProfileDetailView.vue'
import Skeleton from '../../components/common/Skeleton.vue'
import { openContextMenu } from '../../composables/useContextMenu'
import { showToast } from '../../composables/useToasts'
import { useSlidingTabIndicator } from '../../composables/useSlidingTabIndicator'

import iconRefresh from '../../assets/icons8-refresh-64.png'
import iconSearch  from '../../assets/icons8-search-50.png'
import iconRemove  from '../../assets/icons8-remove-24.png'
import bgPlaceholder  from '../../assets/wizard/default-banner.jpg'
import bejaLogo       from '../../assets/bc-logo-new.png'

const LOADER_NAMES: Record<string, string> = {
  fabric: 'Fabric', forge: 'Forge', neoforge: 'NeoForge', quilt: 'Quilt', vanilla: 'Vanilla',
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
  { id: 'all',         label: 'All' },
  { id: 'server',      label: 'Per-Server' },
  { id: 'recommended', label: 'Recommended' },
  { id: 'beja',        label: 'Beja' },
] as const

type TabId = typeof TABS[number]['id']

const store = useLauncherStore()

const activeTab      = ref<TabId>('all')
const { setTabBtnRef, indicatorStyle } = useSlidingTabIndicator(activeTab)
const search         = ref('')
const loading        = ref(false)
const refreshing     = ref(false)
const modCounts      = ref<Record<string, number>>({})
const deleteTarget   = ref<LaunchProfile | null>(null)
const selectedProfile = ref<LaunchProfile | null>(null)

const activeProfileId = computed(() => store.activeProfile?.id ?? null)

const filteredProfiles = computed(() => {
  const q = search.value.toLowerCase()
  return store.profiles
    .filter(p => {
      if (activeTab.value === 'beja')        return p.useBejaClient
      if (activeTab.value === 'recommended') return p.useBejaClient
      return true
    })
    .filter(p => !q || p.name.toLowerCase().includes(q) || p.version.includes(q))
})

async function loadModCounts(): Promise<void> {
  const counts: Record<string, number> = {}
  await Promise.all(
    store.profiles.map(async p => {
      try {
        const mods = await window.api.mods.list(p.id)
        counts[p.id] = mods.filter(m => m.enabled).length
      } catch {
        counts[p.id] = 0
      }
    })
  )
  modCounts.value = counts
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

function openProfileMenu(event: MouseEvent, profile: LaunchProfile): void {
  openContextMenu(event, [
    { label: 'Launch',   icon: 'play',   onClick: () => launchProfile(profile) },
    { label: 'Edit',     icon: 'pencil', onClick: () => openDetail(profile) },
    { label: 'Duplicate', icon: 'copy',  onClick: () => duplicateProfile(profile) },
    { separator: true, label: '' },
    { label: 'Delete', icon: 'trash', danger: true, onClick: () => confirmDelete(profile) },
  ])
}

onMounted(async () => {
  loading.value = true
  try {
    await store.loadProfiles()
    await loadModCounts()
  } finally {
    loading.value = false
  }
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

// ── Tabs ──────────────────────────────────────────────────────────────────────
.profiles-tabs {
  position: relative;
  display: flex;
  gap: 10px;
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
  white-space: nowrap;

  &:hover { background: #1a1a1a; color: #ccc; }

  &.active {
    background: #111;
    color: #d9d9d9;
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
  width: 240px;
  background: none;
  border: none;
  height: 38px;
  flex-shrink: 0;
  overflow: hidden;
}

.search-input {
  width: 140px;
  background: transparent;
  border: none;
  outline: none;
  color: #fff;
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  padding: 0 12px;
  height: 100%;

  &::placeholder {
    color: rgba(255, 255, 255, 0.35);
  }
}

.search-actions {
  display: flex;
  align-items: center;
  height: 100%;
  flex-shrink: 0;
}

.search-sep {
  width: 1px;
  height: 20px;
  background: rgba(255, 255, 255, 0.12);
}

.search-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 100%;
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 0;

  &:hover:not(.non-interactive) {
    background: rgba(255, 255, 255, 0.07);
  }

  &.non-interactive {
    cursor: default;
  }
}

.search-icon {
  width: 16px;
  height: 16px;
  object-fit: contain;
  filter: brightness(0) invert(1);
  opacity: 0.75;

  &.spinning {
    animation: spin 0.6s linear infinite;
  }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

// ── Grid ──────────────────────────────────────────────────────────────────────
.profiles-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  align-items: start;
  gap: 14px;
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
// Mirrors InstanceWizard's live-preview card (.wiz-pcard): media block with a
// centered icon, then a separate solid info panel with name/desc/meta below.
// The wizard promises "this is what you'll see on the launch grid" — this
// card is what makes that true instead of the plain boxed list it replaced.
.profile-card {
  background: #000;
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 3px;
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
    background: transparent;
    border: 2px dashed rgba(255, 255, 255, 0.14);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-height: 260px;
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

.new-card-label {
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  letter-spacing: 0.03em;
  color: inherit;
}

.card-media {
  position: relative;
  aspect-ratio: 515 / 232;
  background: #191a1f;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.card-media-img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0.9;
}

.card-icon {
  position: relative;
  width: 54px;
  height: 54px;
  border-radius: 2px;
  background: #101114;
  border: 1px solid rgba(255, 255, 255, 0.12);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.5);

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
  border-radius: 2px;
  color: #ff8a8a;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  pointer-events: none;
}

.card-delete-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  background: rgba(0, 0, 0, 0.45);
  border: none;
  border-radius: 1px;
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

// ── Info panel ──────────────────────────────────────────────────────────────
// The name always stays visible in a compact black strip below the artwork.
// Only the desc/meta grow in on hover (via grid-template-rows 0fr → 1fr,
// which animates to the content's true intrinsic height — unlike max-height,
// it doesn't need a guessed px cap that reads as a sudden reveal followed by
// empty easing once the content is already fully shown).
.card-info {
  padding: 10px 14px;
  background: #000;
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-top: none;
  transition: border-color 460ms ease;

  .profile-card:hover & {
    border-color: #fff;
    transition: border-color 500ms ease 60ms;
  }
}

.card-name {
  display: block;
  font-size: 14px;
  font-weight: 700;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-extra {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows 480ms cubic-bezier(0.16, 1, 0.3, 1);

  .profile-card:hover & {
    grid-template-rows: 1fr;
  }
}

.card-extra-inner {
  overflow: hidden;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
  opacity: 0;
  transition: opacity 320ms ease;

  .profile-card:hover & {
    opacity: 1;
    padding-top: 7px;
    transition: opacity 380ms ease 60ms, padding-top 480ms cubic-bezier(0.16, 1, 0.3, 1);
  }
}

.card-desc {
  font-size: 11.5px;
  color: rgba(255, 255, 255, 0.55);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.card-meta {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.38);
  margin-top: 3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
  border-radius: $radius;
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
