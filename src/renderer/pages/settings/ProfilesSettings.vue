<template>
  <div class="profiles-page">

    <!-- ── Detail view (slide in) ──────────────────────────── -->
    <Transition name="detail-slide">
      <ProfileDetailView
        v-if="selectedProfile"
        :profile="selectedProfile"
        @back="selectedProfile = null"
        @settings="openWizard"
        @deleted="onDetailDeleted"
      />
    </Transition>

    <!-- ── Grid view ────────────────────────────────────────── -->
    <template v-if="!selectedProfile">
    <!-- Tabs -->
    <div class="profiles-tabs">
      <button
        v-for="tab in TABS"
        :key="tab.id"
        class="tab-btn"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        {{ $t(`profiles.tabs.${tab.id}`) }}
      </button>
    </div>

    <!-- Search bar -->
    <div class="search-bar">
      <input
        v-model="search"
        class="search-input"
        :placeholder="$t('profiles.searchPlaceholder')"
        @keydown.enter="refresh"
      />
      <div class="search-actions">
        <span class="search-sep" />
        <button class="search-icon-btn" :title="$t('profiles.refresh')" @click="refresh">
          <img :src="iconRefresh" class="search-icon" :class="{ spinning: refreshing }" />
        </button>
        <span class="search-sep" />
        <span class="search-icon-btn non-interactive">
          <img :src="iconSearch" class="search-icon" />
        </span>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="!loading && filteredProfiles.length === 0 && search === ''" class="empty-state">
      <p>{{ $t('profiles.noProfiles') }}</p>
      <button class="new-profile-btn" @click="openWizard">{{ $t('profiles.createFirst') }}</button>
    </div>

    <!-- Grid -->
    <div v-else class="profiles-grid">
      <!-- Profile cards -->
      <div
        v-for="profile in filteredProfiles"
        :key="profile.id"
        class="profile-card"
        :class="{ 'profile-card--active': profile.id === activeProfileId }"
      >
        <!-- Top row: icons + delete -->
        <div class="card-top">
          <div class="card-top-icons">
            <img :src="loaderIconSrc(profile.loader)" class="card-loader-icon" :title="profile.loader" />
            <img :src="iconModrinth" class="card-mod-icon" title="Modrinth" />
          </div>
          <button class="card-delete-btn" :title="$t('profiles.deleteProfile')" @click.stop="confirmDelete(profile)">
            <img :src="iconRemove" class="card-btn-icon" />
          </button>
        </div>

        <div class="card-divider" />

        <!-- Body -->
        <div class="card-body">
          <div class="card-name">„{{ profile.name }}"</div>
          <div class="card-version">{{ profile.version }}</div>
          <div class="card-mods">
            <span v-if="modCounts[profile.id] !== undefined">{{ $t('profiles.modsCount', { count: modCounts[profile.id] }) }}</span>
            <span v-else class="mods-loading">…</span>
          </div>
        </div>

        <!-- Edit button -->
        <button class="card-edit-btn" :title="$t('profiles.profileSettings')" @click="openDetail(profile)">
          <img :src="iconCreate" class="card-btn-icon" />
        </button>
      </div>

      <!-- New profile slot -->
      <button class="profile-card profile-card--new" @click="openWizard">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
    </div>

    <!-- Delete confirm modal -->
    <Teleport to="body">
      <Transition name="modal-fade">
        <div v-if="deleteTarget" class="modal-overlay" @click.self="deleteTarget = null">
          <div class="modal">
            <p class="modal-title">{{ $t('profiles.deleteTitle', { name: deleteTarget.name }) }}</p>
            <p class="modal-body">{{ $t('profiles.deleteBody') }}</p>
            <div class="modal-actions">
              <button class="modal-btn modal-btn--cancel" @click="deleteTarget = null">{{ $t('profiles.cancel') }}</button>
              <button class="modal-btn modal-btn--danger" @click="doDelete">{{ $t('profiles.deleteConfirm') }}</button>
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
import { useI18n } from 'vue-i18n'
import { useLauncherStore } from '../../store/launcherStore'
import type { LaunchProfile } from '../../types'
import ProfileDetailView from '../../components/profiles/ProfileDetailView.vue'

const { t } = useI18n()

import iconRefresh from '../../assets/icons8-refresh-64.png'
import iconSearch  from '../../assets/icons8-search-50.png'
import iconRemove  from '../../assets/icons8-remove-24.png'
import iconCreate  from '../../assets/icons8-create-48.png'
import iconModrinth from '../../assets/modrinth.png'
import loaderFabric   from '../../assets/loaders/fabric.png'
import loaderForge    from '../../assets/loaders/forge.png'
import loaderNeoforge from '../../assets/loaders/neoforge.png'
import loaderQuilt    from '../../assets/loaders/quilt.webp'
import iconBlocks     from '../../assets/icons8-blocks-middle.png'

const LOADER_ICONS: Record<string, string> = {
  fabric:   loaderFabric,
  forge:    loaderForge,
  neoforge: loaderNeoforge,
  quilt:    loaderQuilt,
  vanilla:  iconBlocks,
}

function loaderIconSrc(loader: string): string {
  return LOADER_ICONS[loader] ?? iconBlocks
}

const TABS = [
  { id: 'all',       label: 'ALL' },
  { id: 'server',    label: 'Per-server' },
  { id: 'recommended', label: 'recomended' },
  { id: 'beja',      label: 'Beja' },
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
  display: flex;
  gap: 10px;
  flex-shrink: 0;
}

.tab-btn {
  padding: 7px 18px;
  background: #111;
  border: 2px solid rgba(255, 255, 255, 0.18);
  color: rgba(255, 255, 255, 0.65);
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  cursor: pointer;
  transition: background $transition, color $transition, border-color $transition;
  white-space: nowrap;

  &:hover {
    border-color: rgba(255, 255, 255, 0.32);
    color: #fff;
    background: rgba(255, 255, 255, 0.06);
  }

  &.active {
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.55);
    color: #fff;
  }
}

// ── Search bar ────────────────────────────────────────────────────────────────
.search-bar {
  display: flex;
  align-items: center;
  background: #111;
  border: 2px solid rgba(255, 255, 255, 0.18);
  height: 38px;
  flex-shrink: 0;
  overflow: hidden;

  &:focus-within {
    border-color: rgba(255, 255, 255, 0.32);
  }
}

.search-input {
  flex: 1;
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
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}

// ── Card ──────────────────────────────────────────────────────────────────────
.profile-card {
  background: #07070a;
  border: 2px solid rgba(255, 255, 255, 0.14);
  display: flex;
  flex-direction: column;
  align-items: stretch;
  min-height: 190px;
  padding: 10px;
  gap: 0;
  position: relative;
  transition: border-color $transition, box-shadow $transition;
  cursor: default;

  &:hover {
    border-color: rgba(255, 255, 255, 0.28);
    box-shadow: 0 0 12px rgba(255, 255, 255, 0.04);
  }

  &--active {
    border-color: rgba(62, 184, 255, 0.6);
    box-shadow: 0 0 14px rgba(62, 184, 255, 0.14);
  }

  &--new {
    background: transparent;
    border: 2px solid rgba(255, 255, 255, 0.12);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.22);
    min-height: 190px;
    transition: border-color $transition, color $transition, background $transition;

    &:hover {
      border-color: rgba(255, 255, 255, 0.28);
      color: rgba(255, 255, 255, 0.5);
      background: rgba(255, 255, 255, 0.03);
    }
  }
}

// ── Card top row ──────────────────────────────────────────────────────────────
.card-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.card-top-icons {
  display: flex;
  align-items: center;
  gap: 4px;
}

.card-loader-icon,
.card-mod-icon {
  width: 18px;
  height: 18px;
  object-fit: contain;
}

.card-delete-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 2px;
  opacity: 0.5;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: opacity $transition;

  &:hover {
    opacity: 1;
  }
}

.card-btn-icon {
  width: 14px;
  height: 14px;
  object-fit: contain;
  filter: brightness(0) invert(1);
}

// ── Divider ───────────────────────────────────────────────────────────────────
.card-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.1);
  margin-bottom: 10px;
}

// ── Card body ─────────────────────────────────────────────────────────────────
.card-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.card-name {
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  color: #fff;
  line-height: 1.3;
  word-break: break-word;
}

.card-version {
  font-size: 9px;
  color: rgba(255, 255, 255, 0.45);
}

.card-mods {
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  color: #fff;
  margin-top: auto;
  padding-top: 6px;
}

.mods-loading {
  opacity: 0.35;
}

// ── Card edit button ──────────────────────────────────────────────────────────
.card-edit-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  border: 2px solid rgba(255, 255, 255, 0.14);
  cursor: pointer;
  padding: 6px;
  margin-top: 10px;
  transition: background $transition, border-color $transition;

  &:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.32);
  }

  .card-btn-icon {
    width: 16px;
    height: 16px;
  }
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
  background: $primary;
  border: none;
  color: #000;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: opacity $transition;

  &:hover { opacity: 0.85; }
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
  background: #111;
  border: 1px solid rgba(255, 255, 255, 0.12);
  padding: 24px;
  min-width: 280px;
  display: flex;
  flex-direction: column;
  gap: 10px;
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

.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 150ms ease;
}
.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
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
