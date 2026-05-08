<template>
  <div class="mods-page page-content">

    <!-- Header -->
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">Mods</h1>
        <span class="page-subtitle">{{ activeProfile?.name ?? 'No profile selected' }}</span>
      </div>
      <div class="header-right" v-if="activeTab === 'installed'">
        <Button variant="ghost" size="sm" @click="openFolder">Open Folder</Button>
        <Button variant="primary" size="sm" :disabled="!activeProfile" @click="importMods">
          <template #icon>
            <svg width="12" height="12" viewBox="0 0 12 12"><line x1="6" y1="1" x2="6" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><line x1="1" y1="6" x2="11" y2="6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
          </template>
          Import .jar
        </Button>
      </div>
    </div>

    <!-- Vanilla warning -->
    <div v-if="activeProfile?.loader === 'vanilla' && activeTab !== 'installed'" class="vanilla-warn">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.2"/><line x1="7" y1="4" x2="7" y2="7.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><circle cx="7" cy="9.5" r="0.7" fill="currentColor"/></svg>
      Your active profile uses Vanilla — mods require Fabric or Forge. Create a modded profile in Settings → Profiles.
    </div>

    <!-- Tab bar -->
    <div class="tab-bar">
      <button v-for="t in tabs" :key="t.id" class="tab-btn" :class="{ active: activeTab === t.id }" @click="activeTab = t.id">
        {{ t.label }}
      </button>
    </div>

    <!-- ── INSTALLED ─────────────────────────────────────────────────────── -->
    <template v-if="activeTab === 'installed'">
      <div v-if="!activeProfile" class="empty-state">
        <span class="empty-title">No profile selected</span>
        <span class="empty-text">Create a profile in Settings → Profiles</span>
      </div>
      <div v-else-if="mods.length === 0" class="empty-state">
        <span class="empty-title">No mods installed</span>
        <span class="empty-text">Browse the Mods tab or import a .jar file</span>
      </div>
      <div v-else class="mods-list">
        <div class="mods-stats">
          <span class="stat-text">{{ enabledCount }} enabled · {{ mods.length }} total</span>
        </div>
        <div class="mods-items">
          <div v-for="mod in mods" :key="mod.id" class="mod-item-wrap">
            <!-- Mod row -->
            <div class="mod-item" :class="{ disabled: !mod.enabled, editing: editingModId === mod.id }">
              <div class="mod-left">
                <div class="mod-icon">
                  <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M7 1L12.5 4.5V9.5L7 13L1.5 9.5V4.5L7 1Z" stroke="currentColor" stroke-width="1" stroke-linejoin="round"/></svg>
                </div>
                <div class="mod-info">
                  <span class="mod-name" :class="{ disabled: !mod.enabled }">{{ mod.name }}</span>
                  <span class="mod-file">{{ mod.fileName }}</span>
                </div>
              </div>
              <div class="mod-right">
                <span class="mod-size">{{ formatSize(mod.fileSize) }}</span>
                <Toggle :model-value="mod.enabled" @update:model-value="toggleMod(mod.id)" />
                <button class="mod-edit-btn" :class="{ active: editingModId === mod.id }" @click="toggleEdit(mod)" title="Change version">
                  <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M8.5 1.5L10.5 3.5L4 10H2V8L8.5 1.5Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/></svg>
                </button>
                <button class="mod-delete" @click="deleteMod(mod.id)">
                  <svg width="12" height="12" viewBox="0 0 12 12"><line x1="1" y1="1" x2="11" y2="11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/><line x1="11" y1="1" x2="1" y2="11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
                </button>
              </div>
            </div>

            <!-- Version-swap panel -->
            <Transition name="edit-expand">
              <div v-if="editingModId === mod.id" class="mod-edit-panel">
                <div class="edit-panel-title">Change Version</div>

                <div class="edit-search-row">
                  <input
                    v-model="editQuery"
                    class="edit-input"
                    placeholder="Search Modrinth…"
                    @keydown.enter="searchEdit"
                  />
                  <button class="edit-search-btn" :disabled="editLoading" @click="searchEdit">
                    {{ editLoading ? '…' : 'Search' }}
                  </button>
                </div>

                <!-- Project hits -->
                <div v-if="editHits.length" class="edit-hits">
                  <button
                    v-for="hit in editHits"
                    :key="hit.project_id"
                    class="edit-hit"
                    :class="{ selected: editProjectId === hit.project_id }"
                    @click="selectEditProject(hit)"
                  >
                    <img v-if="hit.icon_url" :src="hit.icon_url" class="edit-hit-icon" @error="(e:Event)=>((e.target as HTMLImageElement).style.display='none')" />
                    <div v-else class="edit-hit-icon-placeholder" />
                    <span class="edit-hit-name">{{ hit.title }}</span>
                    <svg v-if="editProjectId === hit.project_id" width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 6L5 9L10 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
                  </button>
                </div>

                <!-- Version picker -->
                <div v-if="editVersions.length" class="edit-version-row">
                  <label class="edit-label">Version</label>
                  <select v-model="editVersionId" class="edit-select">
                    <option v-for="v in editVersions" :key="v.id" :value="v.id">
                      {{ v.name }} ({{ v.game_versions.slice(-1)[0] }})
                    </option>
                  </select>
                </div>

                <div class="edit-actions">
                  <button class="edit-cancel" @click="cancelEdit">Cancel</button>
                  <button
                    class="edit-swap"
                    :disabled="!editVersionId || editSwapping"
                    @click="swapVersion(mod)"
                  >
                    {{ editSwapping ? 'Installing…' : 'Swap Version' }}
                  </button>
                </div>
              </div>
            </Transition>
          </div>
        </div>
      </div>
    </template>

    <!-- ── BROWSE (mods / modpacks / resourcepacks / shaders / datapacks) ── -->
    <template v-else>
      <div class="browse-bar">
        <input
          v-model="query"
          class="search-input"
          :placeholder="searchPlaceholder"
          @keydown.enter="doSearch(true)"
        />
        <Button variant="primary" size="sm" :loading="loading" @click="doSearch(true)">Search</Button>
      </div>

      <div v-if="loading && results.length === 0" class="browse-loading">
        <div class="browse-spinner" />
        <span>Searching…</span>
      </div>
      <div v-else-if="browseError" class="browse-error">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.2"/><line x1="7" y1="4" x2="7" y2="7.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><circle cx="7" cy="9.5" r="0.7" fill="currentColor"/></svg>
        {{ browseError }}
      </div>
      <div v-else-if="results.length === 0" class="empty-state">
        <span class="empty-title">No results found</span>
        <span class="empty-text">Try a different search term</span>
      </div>

      <div v-else class="results-grid">
        <div v-for="hit in results" :key="hit.project_id" class="result-card">
          <img
            v-if="hit.icon_url"
            class="result-icon"
            :src="hit.icon_url"
            :alt="hit.title"
            @error="(e: Event) => (e.target as HTMLImageElement).style.display = 'none'"
          />
          <div v-else class="result-icon-placeholder">
            <svg width="18" height="18" viewBox="0 0 14 14" fill="none"><path d="M7 1L12.5 4.5V9.5L7 13L1.5 9.5V4.5L7 1Z" stroke="currentColor" stroke-width="1"/></svg>
          </div>
          <div class="result-body">
            <div class="result-top">
              <span class="result-name">{{ hit.title }}</span>
              <span class="result-downloads">⬇ {{ formatDownloads(hit.downloads) }}</span>
            </div>
            <p class="result-desc">{{ hit.description }}</p>
            <div class="result-tags">
              <span v-for="cat in displayTags(hit)" :key="cat" class="tag">{{ cat }}</span>
            </div>
          </div>
          <div class="result-actions">
            <Button
              variant="primary"
              size="sm"
              :loading="installing === hit.project_id"
              :disabled="!!installing"
              @click="installHit(hit)"
            >Install</Button>
          </div>
        </div>
      </div>

      <div v-if="total > results.length" class="load-more">
        <Button variant="secondary" size="sm" :loading="loading" @click="doSearch(false)">Load more</Button>
      </div>
    </template>

    <!-- Progress overlay -->
    <Transition name="fade">
      <div v-if="progressMsg" class="progress-overlay">
        <div class="progress-box">
          <div class="progress-spinner" />
          <span class="progress-text">{{ progressMsg }}</span>
        </div>
      </div>
    </Transition>

    <!-- Toast -->
    <Transition name="fade">
      <div v-if="toast" class="toast" :class="toast.type">{{ toast.text }}</div>
    </Transition>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import Button from '../components/common/Button.vue'
import Toggle from '../components/common/Toggle.vue'
import { useLauncherStore } from '../store/launcherStore'
import type { ModInfo, ModrinthHit, ModrinthVersion, ModrinthProjectType } from '../types'

const launcherStore = useLauncherStore()
const activeProfile = computed(() => launcherStore.activeProfile)

const tabs = [
  { id: 'installed',    label: 'Installed'      },
  { id: 'mods',         label: 'Browse Mods'    },
  { id: 'modpacks',     label: 'Modpacks'       },
  { id: 'resourcepacks',label: 'Resource Packs' },
  { id: 'shaders',      label: 'Shaders'        },
  { id: 'datapacks',    label: 'Datapacks'      },
]
const activeTab = ref('installed')

// ── installed ─────────────────────────────────────────────────────────────────
const mods = ref<ModInfo[]>([])
const enabledCount = computed(() => mods.value.filter(m => m.enabled).length)

async function loadMods() {
  if (!activeProfile.value) { mods.value = []; return }
  mods.value = await window.api.mods.list(activeProfile.value.id)
}
async function importMods() {
  if (!activeProfile.value) return
  mods.value = await window.api.mods.install(activeProfile.value.id)
}
async function toggleMod(modId: string) {
  if (!activeProfile.value) return
  mods.value = await window.api.mods.toggle(activeProfile.value.id, modId)
}
async function deleteMod(modId: string) {
  if (!activeProfile.value) return
  mods.value = await window.api.mods.delete(activeProfile.value.id, modId)
}
async function openFolder() {
  if (!activeProfile.value) return
  await window.api.mods.openFolder(activeProfile.value.id)
}
watch(() => activeProfile.value?.id, loadMods)
onMounted(loadMods)

// ── version-swap edit state ───────────────────────────────────────────────────
const editingModId  = ref<string | null>(null)
const editQuery     = ref('')
const editLoading   = ref(false)
const editHits      = ref<ModrinthHit[]>([])
const editProjectId = ref<string | null>(null)
const editVersions  = ref<ModrinthVersion[]>([])
const editVersionId = ref<string>('')
const editSwapping  = ref(false)

function extractSearchTerm(fileName: string): string {
  const base = fileName.replace(/\.jar(\.disabled)?$/, '')
  const parts = base.split(/[-_+]/)
  return parts.filter(p => !/^\d/.test(p) && p.length > 1).slice(0, 2).join(' ') || base
}

function toggleEdit(mod: ModInfo) {
  if (editingModId.value === mod.id) { cancelEdit(); return }
  editingModId.value = mod.id
  editQuery.value    = extractSearchTerm(mod.fileName)
  editHits.value     = []
  editProjectId.value = null
  editVersions.value  = []
  editVersionId.value = ''
  searchEdit()
}

function cancelEdit() {
  editingModId.value  = null
  editHits.value      = []
  editProjectId.value = null
  editVersions.value  = []
  editVersionId.value = ''
}

async function searchEdit() {
  if (!editQuery.value.trim()) return
  editLoading.value   = true
  editHits.value      = []
  editProjectId.value = null
  editVersions.value  = []
  editVersionId.value = ''
  try {
    const { hits } = await window.api.modrinth.search(
      editQuery.value,
      'mod',
      activeProfile.value?.version,
      activeProfile.value?.loader,
    )
    editHits.value = hits.slice(0, 5)
  } catch { /* ignore */ } finally {
    editLoading.value = false
  }
}

async function selectEditProject(hit: ModrinthHit) {
  editProjectId.value = hit.project_id
  editVersionId.value = ''
  editVersions.value  = []
  try {
    editVersions.value = await window.api.modrinth.versions(
      hit.project_id,
      activeProfile.value?.version,
      activeProfile.value?.loader,
    )
    if (editVersions.value.length) editVersionId.value = editVersions.value[0].id
  } catch { /* ignore */ }
}

async function swapVersion(mod: ModInfo) {
  if (!editProjectId.value || !editVersionId.value || !activeProfile.value) return
  editSwapping.value = true
  progressMsg.value  = 'Starting…'
  try {
    mods.value = await window.api.modrinth.swapMod(
      activeProfile.value.id,
      mod.id,
      editProjectId.value,
      editVersionId.value,
    )
    cancelEdit()
    showToast('Version updated', 'success')
  } catch (e) {
    showToast(String(e), 'error')
  } finally {
    editSwapping.value = false
    progressMsg.value  = ''
  }
}

// ── browse (shared state) ─────────────────────────────────────────────────────
const query   = ref('')
const results = ref<ModrinthHit[]>([])
const total   = ref(0)
const loading = ref(false)
const browseError = ref('')
const installing = ref<string | null>(null)
const progressMsg = ref('')
const toast = ref<{ text: string; type: 'success' | 'error' } | null>(null)

const modrinthType = computed((): ModrinthProjectType => {
  if (activeTab.value === 'modpacks')      return 'modpack'
  if (activeTab.value === 'resourcepacks') return 'resourcepack'
  if (activeTab.value === 'shaders')       return 'shader'
  if (activeTab.value === 'datapacks')     return 'datapack'
  return 'mod'
})

const searchPlaceholder = computed(() => {
  if (activeTab.value === 'modpacks')      return 'Search modpacks…'
  if (activeTab.value === 'resourcepacks') return 'Search resource packs…'
  if (activeTab.value === 'shaders')       return 'Search shaders…'
  if (activeTab.value === 'datapacks')     return 'Search datapacks…'
  return 'Search mods…'
})

function displayTags(hit: ModrinthHit): string[] {
  if (activeTab.value === 'modpacks') return hit.game_versions.slice(-3).reverse()
  return hit.categories.slice(0, 3)
}

async function doSearch(reset: boolean) {
  if (reset) { results.value = []; browseError.value = '' }
  loading.value = true
  try {
    const gameVersion = (activeTab.value === 'mods') ? activeProfile.value?.version : undefined
    const loader      = (activeTab.value === 'mods') ? activeProfile.value?.loader  : undefined
    const { hits, total_hits } = await window.api.modrinth.search(
      query.value,
      modrinthType.value,
      gameVersion,
      loader,
      reset ? 0 : results.value.length,
    )
    results.value = reset ? hits : [...results.value, ...hits]
    total.value = total_hits
  } catch (e) {
    console.error('[ModsPage] search failed:', e)
    browseError.value = String(e)
  } finally {
    loading.value = false
  }
}

// Reset + auto-load when switching browse tabs
watch(activeTab, tab => {
  if (tab === 'installed') return
  query.value = ''
  results.value = []
  total.value = 0
  doSearch(true)
})

function showToast(text: string, type: 'success' | 'error') {
  toast.value = { text, type }
  setTimeout(() => { toast.value = null }, 3500)
}

// Register once per app lifetime (module-level guard prevents stacking on remount)
let _progressRegistered = false
if (!_progressRegistered) {
  _progressRegistered = true
  window.api.modrinth.onProgress(msg => {
    progressMsg.value = msg === 'Done' ? '' : msg
  })
}

async function installHit(hit: ModrinthHit) {
  if (!activeProfile.value) { showToast('Select a profile first', 'error'); return }
  installing.value = hit.project_id
  progressMsg.value = 'Starting…'
  try {
    if (activeTab.value === 'mods') {
      await window.api.modrinth.installMod(hit.project_id, activeProfile.value.id)
      await loadMods()
      showToast(`${hit.title} installed`, 'success')
    } else if (activeTab.value === 'modpacks') {
      const result = await window.api.modrinth.installModpack(hit.project_id, null)
      await launcherStore.loadProfiles()
      showToast(`${result.name} installed as a new profile`, 'success')
    } else if (activeTab.value === 'resourcepacks') {
      await window.api.modrinth.installResourcePack(hit.project_id, activeProfile.value.id)
      showToast(`${hit.title} installed`, 'success')
    } else if (activeTab.value === 'shaders') {
      await window.api.modrinth.installShader(hit.project_id, activeProfile.value.id)
      showToast(`${hit.title} installed to shaderpacks`, 'success')
    } else if (activeTab.value === 'datapacks') {
      await window.api.modrinth.installDatapack(hit.project_id, activeProfile.value.id)
      showToast(`${hit.title} downloaded — copy it to your world's datapacks folder`, 'success')
    }
  } catch (e) {
    showToast(String(e), 'error')
  } finally {
    installing.value = null
    progressMsg.value = ''
  }
}

// ── utils ─────────────────────────────────────────────────────────────────────
function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(0)} KB`
  return `${(bytes / 1048576).toFixed(1)} MB`
}
function formatDownloads(n: number) {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(0)}K`
  return String(n)
}
</script>

<style lang="scss" scoped>
.mods-page { display: flex; flex-direction: column; gap: $sp-4; position: relative; }

.page-header { display: flex; align-items: flex-start; justify-content: space-between; }
.header-left { display: flex; flex-direction: column; gap: 2px; }
.page-title { font-size: 20px; font-weight: 800; color: $text-primary; }
.page-subtitle { font-size: 12px; color: $muted; }
.header-right { display: flex; gap: $sp-2; }

.vanilla-warn {
  display: flex; align-items: center; gap: $sp-2;
  padding: $sp-2 $sp-3; border-radius: $radius;
  background: $surface-elevated; border: 1px solid $border;
  font-size: 12px; color: $text-secondary;
}

.tab-bar {
  display: flex; gap: 1px; background: $border; padding: 1px; border-radius: $radius;
  align-self: flex-start;
}
.tab-btn {
  padding: 5px $sp-4; font-size: 12px; font-weight: 600; color: $text-secondary;
  background: transparent; border: none; border-radius: $radius-sm; cursor: pointer;
  transition: background $transition, color $transition;
  &:hover { color: $text-primary; }
  &.active { background: $surface-elevated; color: $text-primary; }
}

// Installed
.mods-list { display: flex; flex-direction: column; gap: $sp-3; }
.mods-stats { padding: $sp-1 0; }
.stat-text { font-size: 11px; color: $muted; }
.mods-items { display: flex; flex-direction: column; gap: 1px; }
.mod-item-wrap { display: flex; flex-direction: column; }
.mod-item {
  display: flex; align-items: center; justify-content: space-between;
  padding: $sp-3 $sp-4; background: $surface; border-radius: $radius;
  transition: background $transition, opacity $transition, border-radius $transition;
  &:hover { background: $surface-elevated; }
  &.disabled { opacity: 0.5; }
  &.editing { border-radius: $radius $radius 0 0; background: $surface-elevated; }
}
.mod-left { display: flex; align-items: center; gap: $sp-3; min-width: 0; }
.mod-icon {
  width: 32px; height: 32px; display: flex; align-items: center; justify-content: center;
  background: $surface-elevated; border: 1px solid $border; border-radius: $radius;
  color: $muted; flex-shrink: 0;
}
.mod-info { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.mod-name {
  font-size: 13px; font-weight: 600; color: $text-primary;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  &.disabled { color: $muted; }
}
.mod-file { font-size: 10px; color: $muted; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.mod-right { display: flex; align-items: center; gap: $sp-2; flex-shrink: 0; }
.mod-size { font-size: 11px; color: $muted; min-width: 50px; text-align: right; }
.mod-edit-btn {
  width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
  background: transparent; border: none; border-radius: $radius; color: $muted; cursor: pointer;
  transition: background $transition, color $transition;
  &:hover { background: $surface-elevated; color: $text-primary; }
  &.active { background: $border; color: $text-primary; }
}
.mod-delete {
  width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
  background: transparent; border: none; border-radius: $radius; color: $muted; cursor: pointer;
  transition: background $transition, color $transition;
  &:hover { background: $border; color: $text-primary; }
}

// Version-swap panel
.mod-edit-panel {
  background: $surface-elevated; border: 1px solid $border; border-top: none;
  border-radius: 0 0 $radius $radius; padding: $sp-3 $sp-4; display: flex;
  flex-direction: column; gap: $sp-3;
}
.edit-panel-title { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.07em; color: $muted; }
.edit-search-row { display: flex; gap: $sp-2; }
.edit-input {
  flex: 1; padding: 6px $sp-3; font-size: 12px; color: $text-primary;
  background: $surface; border: 1px solid $border; border-radius: $radius;
  outline: none; font-family: inherit;
  &:focus { border-color: $primary; }
  &::placeholder { color: $muted; }
}
.edit-search-btn {
  padding: 6px $sp-3; font-size: 12px; font-weight: 600; color: $bg;
  background: $text-primary; border: none; border-radius: $radius; cursor: pointer;
  &:disabled { opacity: 0.5; cursor: not-allowed; }
  &:hover:not(:disabled) { background: $text-secondary; }
}
.edit-hits { display: flex; flex-direction: column; gap: 2px; }
.edit-hit {
  display: flex; align-items: center; gap: $sp-2; padding: 6px $sp-2;
  background: transparent; border: 1px solid transparent; border-radius: $radius;
  cursor: pointer; text-align: left; transition: background $transition, border-color $transition;
  &:hover { background: $surface-elevated; }
  &.selected { background: $border; border-color: $border-strong; }
}
.edit-hit-icon { width: 24px; height: 24px; border-radius: 4px; object-fit: cover; flex-shrink: 0; }
.edit-hit-icon-placeholder { width: 24px; height: 24px; border-radius: 4px; background: $surface; flex-shrink: 0; }
.edit-hit-name { flex: 1; font-size: 12px; font-weight: 600; color: $text-primary; }
.edit-version-row { display: flex; align-items: center; gap: $sp-2; }
.edit-label { font-size: 11px; color: $muted; flex-shrink: 0; }
.edit-select {
  flex: 1; padding: 6px $sp-3; font-size: 12px; color: $text-primary;
  background: $surface; border: 1px solid $border; border-radius: $radius;
  outline: none; font-family: inherit; cursor: pointer;
  &:focus { border-color: $primary; }
}
.edit-actions { display: flex; justify-content: flex-end; gap: $sp-2; }
.edit-cancel {
  padding: 6px $sp-3; font-size: 12px; font-weight: 600; color: $muted;
  background: transparent; border: 1px solid $border; border-radius: $radius; cursor: pointer;
  &:hover { color: $text-primary; border-color: $border-strong; }
}
.edit-swap {
  padding: 6px $sp-4; font-size: 12px; font-weight: 700; color: $bg;
  background: $text-primary; border: none; border-radius: $radius; cursor: pointer;
  &:disabled { opacity: 0.45; cursor: not-allowed; }
  &:hover:not(:disabled) { background: $text-secondary; }
}

.edit-expand-enter-active { transition: all 0.15s ease; }
.edit-expand-leave-active { transition: all 0.12s ease; }
.edit-expand-enter-from, .edit-expand-leave-to { opacity: 0; transform: translateY(-4px); }

// Browse
.browse-bar { display: flex; gap: $sp-2; align-items: center; }
.search-input {
  flex: 1; padding: 8px $sp-3; font-size: 13px; color: $text-primary;
  background: $surface; border: 1px solid $border; border-radius: $radius;
  outline: none; font-family: inherit;
  &:focus { border-color: $primary; }
  &::placeholder { color: $muted; }
}

.browse-loading {
  display: flex; align-items: center; justify-content: center; gap: $sp-3;
  padding: $sp-8; color: $muted; font-size: 13px;
}
.browse-spinner {
  width: 16px; height: 16px; border: 2px solid $border;
  border-top-color: $text-secondary; border-radius: 50%;
  animation: spin 0.7s linear infinite; flex-shrink: 0;
}

.browse-error {
  display: flex; align-items: center; gap: $sp-2; padding: $sp-3;
  background: $surface-elevated; border: 1px solid $border;
  border-radius: $radius; font-size: 12px; color: $text-secondary;
}

.results-grid { display: flex; flex-direction: column; gap: $sp-2; overflow-y: auto; }
.result-card {
  display: flex; align-items: center; gap: $sp-3; padding: $sp-3 $sp-4;
  background: $surface; border: 1px solid $border; border-radius: $radius;
  transition: border-color $transition, background $transition;
  &:hover { border-color: $border-strong; background: $surface-elevated; }
}
.result-icon {
  width: 48px; height: 48px; border-radius: $radius; object-fit: cover; flex-shrink: 0;
}
.result-icon-placeholder {
  width: 48px; height: 48px; border-radius: $radius; flex-shrink: 0;
  background: $surface-elevated; border: 1px solid $border;
  display: flex; align-items: center; justify-content: center; color: $muted;
}
.result-body { flex: 1; display: flex; flex-direction: column; gap: $sp-1; min-width: 0; }
.result-top { display: flex; align-items: baseline; gap: $sp-2; }
.result-name { font-size: 13px; font-weight: 700; color: $text-primary; }
.result-downloads { font-size: 10px; color: $muted; }
.result-desc {
  font-size: 12px; color: $text-secondary; line-height: 1.4;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.result-tags { display: flex; gap: $sp-1; flex-wrap: wrap; }
.tag {
  font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em;
  padding: 2px $sp-2; border-radius: $radius-sm;
  background: $surface-elevated; border: 1px solid $border; color: $muted;
}
.result-actions { flex-shrink: 0; }
.load-more { display: flex; justify-content: center; padding: $sp-3 0; }

.empty-state {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: $sp-8; gap: $sp-2; text-align: center;
}
.empty-title { font-size: 14px; font-weight: 700; color: $text-secondary; }
.empty-text { font-size: 12px; color: $muted; }

.progress-overlay {
  position: absolute; inset: 0; background: rgba(9,9,9,0.8);
  display: flex; align-items: center; justify-content: center;
  z-index: 50; border-radius: $radius; backdrop-filter: blur(4px);
}
.progress-box {
  display: flex; align-items: center; gap: $sp-3;
  background: $surface; border: 1px solid $border; border-radius: $radius;
  padding: $sp-4 $sp-5;
}
.progress-spinner {
  width: 16px; height: 16px; border: 2px solid $border;
  border-top-color: $text-secondary; border-radius: 50%;
  animation: spin 0.7s linear infinite; flex-shrink: 0;
}
.progress-text { font-size: 13px; color: $text-primary; }
@keyframes spin { to { transform: rotate(360deg); } }

.toast {
  position: absolute; bottom: $sp-4; left: 50%; transform: translateX(-50%);
  padding: $sp-2 $sp-5; border-radius: $radius; font-size: 12px; font-weight: 600;
  z-index: 60; white-space: nowrap;
  background: $surface-elevated; border: 1px solid $border; color: $text-primary;
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.slide-up-enter-active { transition: all 0.2s ease; }
.slide-up-enter-from { opacity: 0; transform: translateY(6px); }


</style>
