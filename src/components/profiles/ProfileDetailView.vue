<template>
  <div class="profile-detail">
    <input ref="imageInputRef" type="file" accept="image/png,image/jpeg,image/webp,image/gif" style="display:none" @change="onImageSelected" />

    <!-- Header: back + title + search + browse ────────────────── -->
    <div class="detail-header-row">
      <button class="back-chevron" title="Back" @click="$emit('back')">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6" />
        </svg>
      </button>

      <div class="detail-title">
        <span class="detail-title-name">{{ profile.name }}</span>
        <div class="detail-title-tabs">
          <button
            class="detail-title-tab"
            :class="{ active: activeSection === 'content' }"
            @click="activeSection = 'content'"
          >Content</button>
          <button
            class="detail-title-tab"
            :class="{ active: activeSection === 'worlds' }"
            @click="activeSection = 'worlds'; loadWorldsOnce()"
          >Worlds<span v-if="worlds.length" class="detail-title-tab-count">{{ worlds.length }}</span></button>
          <button
            class="detail-title-tab"
            :class="{ active: activeSection === 'screenshots' }"
            @click="activeSection = 'screenshots'; loadScreenshotsOnce()"
          >Screenshots<span v-if="screenshots.length" class="detail-title-tab-count">{{ screenshots.length }}</span></button>
        </div>
      </div>

      <button class="detail-edit-btn" title="Profile settings" @click="$emit('settings', profile.id)">
        <Icon name="pencil" :size="16" />
      </button>

      <div ref="dotsWrapRef" class="dots-menu-wrap">
        <button class="detail-edit-btn" title="More" @click="menuOpen = !menuOpen">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <circle cx="5" cy="12" r="2"/><circle cx="12" cy="12" r="2"/><circle cx="19" cy="12" r="2"/>
          </svg>
        </button>
        <Transition name="dots-dd">
          <div v-if="menuOpen" class="dots-dropdown">
            <button class="dots-item" @click="exportPack">Export as .beja</button>
            <button class="dots-item" @click="exportZip">Export as ZIP</button>
            <button class="dots-item" @click="shareProfile">Share with a link</button>
            <div class="dots-sep" />
            <button class="dots-item" @click="openHealthCheck">Health Check</button>
            <button class="dots-item" @click="rollbackLastModUpdate">Undo last mod update</button>
          </div>
        </Transition>
      </div>

      <HealthCheckModal v-model="healthCheckOpen" :profile-id="profile.id" />

      <template v-if="activeSection === 'content'">
        <div class="toolbar-search detail-search">
          <input v-model="modSearch" class="toolbar-search-input" placeholder="Search mod / shader / resourcepack..." />
        </div>

        <button class="detail-compass-btn" title="Import external .jar" @click="importMod">
          <img :src="iconFile" class="btn-icon" />
        </button>

        <button class="detail-compass-btn" title="Browse content" @click="browseMods">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"/>
            <polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/>
          </svg>
        </button>
      </template>

      <template v-else-if="activeSection === 'worlds'">
        <div class="toolbar-search detail-search">
          <input v-model="worldSearch" class="toolbar-search-input" placeholder="Search worlds..." />
        </div>

        <button class="detail-compass-btn" title="Restore world from backup" :disabled="restoring" @click="restoreWorld">
          <Icon name="upload" :size="17" />
        </button>

        <button class="detail-compass-btn" title="Refresh" @click="loadWorlds">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spinning: worldsLoading }">
            <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
        </button>
      </template>

      <template v-else>
        <div class="toolbar-search detail-search">
          <input v-model="screenshotSearch" class="toolbar-search-input" placeholder="Search by date..." />
        </div>

        <button class="detail-compass-btn" title="Open screenshots folder" @click="openScreenshotsFolder">
          <Icon name="folder" :size="16" />
        </button>

        <button class="detail-compass-btn" title="Refresh" @click="loadScreenshots">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spinning: screenshotsLoading }">
            <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
        </button>
      </template>
    </div>

    <!-- Mods area ───────────────────────────────────────────── -->
    <div v-if="activeSection === 'content'" class="mods-area">
      <div v-if="modsLoading" class="mods-spinner">
        <div class="spinner" />
      </div>

      <template v-else-if="mods.length === 0">
        <div class="empty-state">
          <img :src="iconNotFound" class="empty-icon" />
          <p class="empty-text">No content installed yet...</p>
          <div class="empty-actions">
            <button class="import-btn" @click="importMod">
              <img :src="iconFile" class="btn-icon" />
              Import
            </button>
            <button class="browse-btn" @click="browseMods">
              Browse
              <img :src="iconModrinth" class="btn-icon btn-icon--modrinth" />
            </button>
          </div>
        </div>
      </template>

      <template v-else>
        <button class="update-all-link" :disabled="updatingAll" @click="updateAllMods">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spinning: updatingAll }">
            <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
          update all
        </button>

        <TransitionGroup tag="div" name="row-stagger" class="mods-list">
          <div
            v-for="mod in filteredMods"
            :key="mod.id"
            class="mod-row"
          >
            <div class="mod-icon-wrap">
              <img v-if="mod.iconDataUrl" :src="mod.iconDataUrl" class="mod-icon" :alt="mod.name" />
              <div v-else class="mod-icon-fallback">{{ mod.name[0] }}</div>
            </div>

            <div class="mod-info">
              <span class="mod-name">{{ mod.name }}</span>
              <p class="mod-desc">{{ mod.fileName }}</p>
            </div>

            <button class="mod-enabled-toggle" :class="{ on: mod.enabled }" @click="toggleMod(mod.id)">
              <span class="toggle-check">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
              </span>
              {{ mod.enabled ? 'Enabled' : 'Disabled' }}
            </button>

            <button class="update-btn" :disabled="updatingIds.has(mod.id)" @click="updateOneMod(mod)">
              {{ updatingIds.has(mod.id) ? '…' : 'update' }}
            </button>

            <button
              class="mod-delete-btn"
              :class="{ confirm: pendingDeleteId === mod.id }"
              :title="pendingDeleteId === mod.id ? 'Click again to confirm' : 'Delete mod'"
              @click="deleteMod(mod.id)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
              <span v-if="pendingDeleteId === mod.id">confirm?</span>
            </button>
          </div>
        </TransitionGroup>
      </template>
    </div>

    <!-- Worlds area ─────────────────────────────────────────── -->
    <div v-else-if="activeSection === 'worlds'" class="mods-area">
      <div v-if="worldsLoading" class="mods-spinner">
        <div class="spinner" />
      </div>

      <template v-else-if="worlds.length === 0">
        <div class="empty-state">
          <img :src="iconNotFound" class="empty-icon" />
          <p class="empty-text">No singleplayer worlds yet...</p>
          <div class="empty-actions">
            <button class="import-btn" :disabled="restoring" @click="restoreWorld">
              <Icon name="upload" :size="15" />
              Restore backup
            </button>
          </div>
        </div>
      </template>

      <template v-else>
        <TransitionGroup tag="div" name="row-stagger" class="mods-list">
          <div v-for="world in filteredWorlds" :key="world.id" class="mod-row world-row">
            <div class="mod-icon-wrap world-icon-wrap">
              <img v-if="world.iconDataUrl" :src="world.iconDataUrl" class="mod-icon world-icon" :alt="world.name" />
              <div v-else class="mod-icon-fallback">{{ world.name[0]?.toUpperCase() }}</div>
            </div>

            <div class="mod-info">
              <template v-if="renamingId === world.id">
                <input
                  v-model="renameDraft"
                  class="world-rename-input"
                  @keyup.enter="confirmRename(world)"
                  @keyup.esc="renamingId = null"
                  @blur="confirmRename(world)"
                  ref="renameInputRef"
                />
              </template>
              <span v-else class="mod-name">
                {{ world.name }}
                <span v-if="world.hardcore" class="world-badge world-badge--hardcore">Hardcore</span>
              </span>
              <p class="mod-desc">
                {{ formatLastPlayed(world.lastPlayed) }} · {{ formatSize(world.sizeBytes) }}
              </p>
            </div>

            <button class="world-action-btn" title="Rename" @click="startRename(world)">
              <Icon name="pencil" :size="14" />
            </button>
            <button class="world-action-btn" title="Duplicate" :disabled="duplicatingId === world.id" @click="duplicateWorld(world)">
              <Icon name="copy" :size="14" />
            </button>
            <button class="world-action-btn" title="Backup to ZIP" :disabled="backingUpId === world.id" @click="backupWorld(world)">
              <Icon name="download" :size="14" />
            </button>
            <button class="world-action-btn" title="Open folder" @click="openWorldFolder(world)">
              <Icon name="folder" :size="14" />
            </button>

            <div class="backups-menu-wrap">
              <button class="world-action-btn" title="Auto-backups" @click="toggleBackupsMenu(world)">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="12" cy="12" r="9"/><polyline points="12 7 12 12 15 14"/>
                </svg>
              </button>
              <Transition name="dots-dd">
                <div v-if="backupsMenuWorldId === world.id" class="dots-dropdown backups-dropdown">
                  <p v-if="!autoBackupsList.length" class="backups-empty">No automatic backups yet - enable Auto-backup Worlds in Launcher Settings.</p>
                  <button v-for="name in autoBackupsList" :key="name" class="dots-item" @click="restoreAutoBackup(world, name)">
                    {{ formatBackupName(name) }}
                  </button>
                </div>
              </Transition>
            </div>

            <button
              class="mod-delete-btn"
              :class="{ confirm: pendingDeleteWorldId === world.id }"
              :title="pendingDeleteWorldId === world.id ? 'Click again to confirm' : 'Delete world'"
              @click="deleteWorld(world)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
              <span v-if="pendingDeleteWorldId === world.id">confirm?</span>
            </button>
          </div>
        </TransitionGroup>
      </template>
    </div>

    <!-- Screenshots area ────────────────────────────────────── -->
    <div v-else class="screenshots-area">
      <div v-if="screenshotsLoading" class="mods-spinner">
        <div class="spinner" />
      </div>

      <template v-else-if="screenshots.length === 0">
        <div class="empty-state">
          <img :src="iconNotFound" class="empty-icon" />
          <p class="empty-text">No screenshots yet - press F2 in-game to take one.</p>
        </div>
      </template>

      <template v-else>
        <TransitionGroup tag="div" name="row-stagger" class="screenshot-grid">
          <div v-for="shot in filteredScreenshots" :key="shot.id" class="screenshot-tile" @click="openScreenshot(shot)">
            <img v-if="thumbnails[shot.id]" :src="thumbnails[shot.id]!" class="screenshot-thumb" :alt="shot.id" />
            <div v-else class="screenshot-thumb screenshot-thumb--loading"><div class="spinner" /></div>

            <div class="screenshot-overlay">
              <span class="screenshot-date">{{ formatShotDate(shot.takenAt) }}</span>
              <div class="screenshot-overlay-actions">
                <button class="screenshot-icon-btn" title="Save as..." @click.stop="exportScreenshot(shot)">
                  <Icon name="download" :size="13" />
                </button>
                <button
                  class="screenshot-icon-btn screenshot-icon-btn--danger"
                  :class="{ confirm: pendingDeleteShotId === shot.id }"
                  :title="pendingDeleteShotId === shot.id ? 'Click again to confirm' : 'Delete'"
                  @click.stop="deleteScreenshot(shot)"
                >
                  <Icon name="trash" :size="13" />
                </button>
              </div>
            </div>
          </div>
        </TransitionGroup>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { useLauncherStore } from '../../store/launcherStore'
import type { LaunchProfile, ModInfo, WorldInfo, ScreenshotInfo } from '../../types'
import Icon from '../common/Icon.vue'
import HealthCheckModal from '../common/HealthCheckModal.vue'
import { showToast } from '../../composables/useToasts'

import iconNotFound from '../../assets/icons8-not-found-50.png'
import iconFile     from '../../assets/icons8-file-64.png'
import iconModrinth from '../../assets/modrinth.png'

const props = withDefaults(
  defineProps<{ profile: LaunchProfile; hasPrev?: boolean; hasNext?: boolean }>(),
  { hasPrev: false, hasNext: false },
)
const emit  = defineEmits<{
  back: []
  settings: [id: string]
  deleted: [id: string]
  prev: []
  next: []
}>()

const store = useLauncherStore()

const localImageUrl  = ref<string | null>(props.profile.imageUrl ?? null)
const imageInputRef  = ref<HTMLInputElement | null>(null)

function pickImage() { imageInputRef.value?.click() }

async function onImageSelected(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  ;(e.target as HTMLInputElement).value = ''
  if (!file) return
  const resized = await resizeImage(file, 128)
  localImageUrl.value = resized
  await store.updateProfile(props.profile.id, { imageUrl: resized })
}

function resizeImage(file: File, size: number): Promise<string> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    const url = URL.createObjectURL(file)
    img.onload = () => {
      URL.revokeObjectURL(url)
      const canvas = document.createElement('canvas')
      canvas.width = size; canvas.height = size
      const ctx = canvas.getContext('2d')!
      const scale = Math.max(size / img.width, size / img.height)
      const w = img.width * scale, h = img.height * scale
      ctx.drawImage(img, (size - w) / 2, (size - h) / 2, w, h)
      resolve(canvas.toDataURL('image/png', 0.85))
    }
    img.onerror = reject
    img.src = url
  })
}

const mods       = ref<ModInfo[]>([])
const modsLoading = ref(false)
const menuOpen   = ref(false)
const dotsWrapRef = ref<HTMLElement | null>(null)
const modSearch  = ref('')
const updatingIds = ref<Set<string>>(new Set())
const updatingAll = ref(false)
const pendingDeleteId = ref<string | null>(null)
let pendingDeleteTimer: ReturnType<typeof setTimeout> | null = null

const filteredMods = computed(() => {
  const q = modSearch.value.trim().toLowerCase()
  if (!q) return mods.value
  return mods.value.filter(m => m.name.toLowerCase().includes(q) || m.fileName.toLowerCase().includes(q))
})

const playtimeLabel = computed(() => {
  const ms = props.profile.playtimeMs ?? 0
  if (ms === 0) return 'never played'
  const h = Math.floor(ms / 3600000)
  const m = Math.floor((ms % 3600000) / 60000)
  return h > 0 ? `${h}h ${m}m played` : `${m}m played`
})

function formatSize(bytes: number): string {
  if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(1)} MB`
  if (bytes >= 1024)    return `${(bytes / 1024).toFixed(0)} KB`
  return `${bytes} B`
}

async function loadMods(): Promise<void> {
  modsLoading.value = true
  try {
    mods.value = await window.api.mods.list(props.profile.id)
  } finally {
    modsLoading.value = false
  }
}

async function launchProfile(): Promise<void> {
  await store.setActiveProfile(props.profile.id)
  await store.launch()
}

async function toggleMod(modId: string): Promise<void> {
  mods.value = await window.api.mods.toggle(props.profile.id, modId)
}

async function deleteMod(modId: string): Promise<void> {
  if (pendingDeleteId.value !== modId) {
    pendingDeleteId.value = modId
    if (pendingDeleteTimer) clearTimeout(pendingDeleteTimer)
    pendingDeleteTimer = setTimeout(() => { pendingDeleteId.value = null }, 3000)
    return
  }
  if (pendingDeleteTimer) clearTimeout(pendingDeleteTimer)
  pendingDeleteId.value = null
  const mod = mods.value.find(m => m.id === modId)
  mods.value = await window.api.mods.delete(props.profile.id, modId)
  if (mod) showToast({ title: `Removed "${mod.name}"`, variant: 'info' })
}

async function importMod(): Promise<void> {
  mods.value = await window.api.mods.install(props.profile.id)
}

// Manual fallback for mods whose own auto-updater didn't run - best-effort: guesses a
// Modrinth project from the filename, so a miss just means "couldn't check", not "current".
async function updateOneMod(mod: ModInfo): Promise<void> {
  if (updatingIds.value.has(mod.id)) return
  updatingIds.value = new Set([...updatingIds.value, mod.id])
  try {
    const info = await window.api.mods.checkUpdate(props.profile.id, mod.id)
    if (!info) {
      showToast({ title: `"${mod.name}" is already up to date`, body: 'Or it isn\'t on Modrinth under a name we could match.', variant: 'info' })
      return
    }
    mods.value = await window.api.modrinth.swapMod(props.profile.id, mod.id, info.projectId, info.versionId)
    showToast({ title: `Updated "${mod.name}"`, body: `to ${info.versionNumber}`, variant: 'success' })
  } catch (e) {
    showToast({ title: `Failed to update "${mod.name}"`, body: String(e), variant: 'error' })
  } finally {
    updatingIds.value = new Set([...updatingIds.value].filter(id => id !== mod.id))
  }
}

async function updateAllMods(): Promise<void> {
  if (updatingAll.value) return
  updatingAll.value = true
  try {
    for (const mod of [...filteredMods.value]) {
      await updateOneMod(mod)
    }
  } finally {
    updatingAll.value = false
  }
}

function browseMods(): void {
  menuOpen.value = false
}

// ── Worlds tab ──────────────────────────────────────────────────────────────
const activeSection    = ref<'content' | 'worlds' | 'screenshots'>('content')
const worlds           = ref<WorldInfo[]>([])
const worldsLoading    = ref(false)
const worldsLoaded     = ref(false)
const worldSearch      = ref('')
const renamingId        = ref<string | null>(null)
const renameDraft       = ref('')
const renameInputRef    = ref<HTMLInputElement[] | HTMLInputElement | null>(null)
const duplicatingId     = ref<string | null>(null)
const backingUpId       = ref<string | null>(null)
const restoring         = ref(false)
const pendingDeleteWorldId = ref<string | null>(null)
let pendingDeleteWorldTimer: ReturnType<typeof setTimeout> | null = null

const filteredWorlds = computed(() => {
  const q = worldSearch.value.trim().toLowerCase()
  if (!q) return worlds.value
  return worlds.value.filter(w => w.name.toLowerCase().includes(q))
})

function formatLastPlayed(ms: number | null): string {
  if (!ms) return 'never opened'
  const days = Math.floor((Date.now() - ms) / 86400000)
  if (days <= 0) return 'today'
  if (days === 1) return 'yesterday'
  if (days < 30) return `${days}d ago`
  return new Date(ms).toLocaleDateString()
}

async function loadWorlds(): Promise<void> {
  worldsLoading.value = true
  try {
    worlds.value = await window.api.worlds.list(props.profile.id)
  } finally {
    worldsLoading.value = false
    worldsLoaded.value = true
  }
}

// Lazy: the Worlds tab's own content only needs to load the first time it's opened, and
// switching back to Content shouldn't re-fetch every time.
function loadWorldsOnce(): void {
  if (!worldsLoaded.value && !worldsLoading.value) loadWorlds()
}

function startRename(world: WorldInfo): void {
  renamingId.value = world.id
  renameDraft.value = world.name
  nextTick(() => {
    const el = renameInputRef.value
    const input = Array.isArray(el) ? el[0] : el
    input?.focus()
    input?.select()
  })
}

async function confirmRename(world: WorldInfo): Promise<void> {
  if (renamingId.value !== world.id) return
  const name = renameDraft.value.trim()
  renamingId.value = null
  if (!name || name === world.name) return
  const result = await window.api.worlds.rename(props.profile.id, world.id, name)
  if (result.ok && result.world) {
    worlds.value = worlds.value.map(w => (w.id === world.id ? result.world! : w))
  } else {
    showToast({ title: `Could not rename "${world.name}"`, body: result.error, variant: 'error' })
  }
}

async function duplicateWorld(world: WorldInfo): Promise<void> {
  if (duplicatingId.value) return
  duplicatingId.value = world.id
  try {
    const result = await window.api.worlds.duplicate(props.profile.id, world.id, `${world.name} (Copy)`)
    if (result.ok && result.world) {
      worlds.value = [result.world, ...worlds.value]
      showToast({ title: `Duplicated "${world.name}"`, variant: 'success' })
    } else {
      showToast({ title: `Could not duplicate "${world.name}"`, body: result.error, variant: 'error' })
    }
  } finally {
    duplicatingId.value = null
  }
}

async function backupWorld(world: WorldInfo): Promise<void> {
  if (backingUpId.value) return
  backingUpId.value = world.id
  try {
    const result = await window.api.worlds.backup(props.profile.id, world.id)
    if (result.ok) {
      showToast({ title: `Backed up "${world.name}"`, variant: 'success' })
    } else if (result.error !== 'cancelled') {
      showToast({ title: `Backup failed for "${world.name}"`, body: result.error, variant: 'error' })
    }
  } finally {
    backingUpId.value = null
  }
}

async function restoreWorld(): Promise<void> {
  if (restoring.value) return
  restoring.value = true
  try {
    const result = await window.api.worlds.restore(props.profile.id)
    if (result.ok && result.world) {
      worlds.value = [result.world, ...worlds.value]
      showToast({ title: `Restored "${result.world.name}"`, variant: 'success' })
    } else if (result.error !== 'cancelled') {
      showToast({ title: 'Restore failed', body: result.error, variant: 'error' })
    }
  } finally {
    restoring.value = false
  }
}

function openWorldFolder(world: WorldInfo): void {
  window.api.worlds.openFolder(props.profile.id, world.id)
}

// ── Auto-backups browser ─────────────────────────────────────────────────
const backupsMenuWorldId = ref<string | null>(null)
const autoBackupsList    = ref<string[]>([])

async function toggleBackupsMenu(world: WorldInfo): Promise<void> {
  if (backupsMenuWorldId.value === world.id) {
    backupsMenuWorldId.value = null
    return
  }
  backupsMenuWorldId.value = world.id
  autoBackupsList.value = await window.api.worlds.listAutoBackups(props.profile.id, world.id)
}

// Filenames are "YYYY-MM-DDTHH-MM-SS.zip" (see auto_backup_service) - reformat for display.
function formatBackupName(name: string): string {
  const stamp = name.replace('.zip', '')
  const [datePart, timePart] = stamp.split('T')
  if (!timePart) return stamp
  return `${datePart} ${timePart.replace(/-/g, ':')}`
}

async function restoreAutoBackup(world: WorldInfo, backupName: string): Promise<void> {
  backupsMenuWorldId.value = null
  const result = await window.api.worlds.restoreAutoBackup(props.profile.id, world.id, backupName)
  if (result.ok && result.world) {
    worlds.value = [result.world, ...worlds.value]
    showToast({ title: `Restored backup of "${world.name}"`, variant: 'success' })
  } else {
    showToast({ title: 'Could not restore backup', body: result.error, variant: 'error' })
  }
}

async function deleteWorld(world: WorldInfo): Promise<void> {
  if (pendingDeleteWorldId.value !== world.id) {
    pendingDeleteWorldId.value = world.id
    if (pendingDeleteWorldTimer) clearTimeout(pendingDeleteWorldTimer)
    pendingDeleteWorldTimer = setTimeout(() => { pendingDeleteWorldId.value = null }, 3000)
    return
  }
  if (pendingDeleteWorldTimer) clearTimeout(pendingDeleteWorldTimer)
  pendingDeleteWorldId.value = null
  const result = await window.api.worlds.delete(props.profile.id, world.id)
  if (result.ok) {
    worlds.value = worlds.value.filter(w => w.id !== world.id)
    showToast({ title: `Deleted "${world.name}"`, variant: 'info' })
  } else {
    showToast({ title: `Could not delete "${world.name}"`, body: result.error, variant: 'error' })
  }
}

// ── Screenshots tab ─────────────────────────────────────────────────────────
const screenshots         = ref<ScreenshotInfo[]>([])
const screenshotsLoading  = ref(false)
const screenshotsLoaded   = ref(false)
const screenshotSearch    = ref('')
const thumbnails          = ref<Record<string, string>>({})
const pendingDeleteShotId = ref<string | null>(null)
let pendingDeleteShotTimer: ReturnType<typeof setTimeout> | null = null

function formatShotDate(ms: number): string {
  return new Date(ms).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })
}

const filteredScreenshots = computed(() => {
  const q = screenshotSearch.value.trim().toLowerCase()
  if (!q) return screenshots.value
  return screenshots.value.filter(s => formatShotDate(s.takenAt).toLowerCase().includes(q))
})

// Thumbnails are decoded+resized on the Rust side per-request (full screenshots can be several
// MB) - fetch with a small concurrency cap instead of one-request-per-image-in-parallel so a
// gallery of a few hundred shots doesn't flood the IPC bridge all at once.
async function loadThumbnails(list: ScreenshotInfo[]): Promise<void> {
  const CONCURRENCY = 6
  let index = 0
  async function worker() {
    while (index < list.length) {
      const shot = list[index++]
      if (thumbnails.value[shot.id]) continue
      const dataUrl = await window.api.screenshots.thumbnail(props.profile.id, shot.id)
      if (dataUrl) thumbnails.value = { ...thumbnails.value, [shot.id]: dataUrl }
    }
  }
  await Promise.all(Array.from({ length: Math.min(CONCURRENCY, list.length) }, worker))
}

async function loadScreenshots(): Promise<void> {
  screenshotsLoading.value = true
  try {
    screenshots.value = await window.api.screenshots.list(props.profile.id)
    loadThumbnails(screenshots.value)
  } finally {
    screenshotsLoading.value = false
    screenshotsLoaded.value = true
  }
}

function loadScreenshotsOnce(): void {
  if (!screenshotsLoaded.value && !screenshotsLoading.value) loadScreenshots()
}

function openScreenshot(shot: ScreenshotInfo): void {
  window.api.screenshots.open(props.profile.id, shot.id)
}

function openScreenshotsFolder(): void {
  window.api.screenshots.openFolder(props.profile.id)
}

async function exportScreenshot(shot: ScreenshotInfo): Promise<void> {
  const result = await window.api.screenshots.export(props.profile.id, shot.id)
  if (result.ok) {
    showToast({ title: 'Screenshot saved', variant: 'success' })
  } else if (result.error !== 'cancelled') {
    showToast({ title: 'Could not save screenshot', body: result.error, variant: 'error' })
  }
}

async function deleteScreenshot(shot: ScreenshotInfo): Promise<void> {
  if (pendingDeleteShotId.value !== shot.id) {
    pendingDeleteShotId.value = shot.id
    if (pendingDeleteShotTimer) clearTimeout(pendingDeleteShotTimer)
    pendingDeleteShotTimer = setTimeout(() => { pendingDeleteShotId.value = null }, 2500)
    return
  }
  if (pendingDeleteShotTimer) clearTimeout(pendingDeleteShotTimer)
  pendingDeleteShotId.value = null
  const result = await window.api.screenshots.delete(props.profile.id, shot.id)
  if (result.ok) {
    screenshots.value = screenshots.value.filter(s => s.id !== shot.id)
  } else {
    showToast({ title: 'Could not delete screenshot', body: result.error, variant: 'error' })
  }
}

async function exportPack(): Promise<void> {
  menuOpen.value = false
  await window.api.profiles.exportPack(props.profile.id)
}

async function exportZip(): Promise<void> {
  menuOpen.value = false
  await window.api.profiles.exportZip(props.profile.id)
}

const healthCheckOpen = ref(false)

function openHealthCheck(): void {
  menuOpen.value = false
  healthCheckOpen.value = true
}

async function rollbackLastModUpdate(): Promise<void> {
  menuOpen.value = false
  const result = await window.api.mods.rollbackLastUpdate(props.profile.id)
  if (result.ok) {
    showToast({ title: `Rolled back "${result.modName}"`, variant: 'success' })
    if (activeSection.value === 'content') loadMods()
  } else {
    showToast({ title: 'Nothing to undo', body: result.error, variant: 'info' })
  }
}

async function shareProfile(): Promise<void> {
  menuOpen.value = false
  let result: Awaited<ReturnType<typeof window.api.profiles.share>>
  try {
    result = await window.api.profiles.share(props.profile.id)
  } catch (err) {
    showToast({ title: 'Could not share profile', body: err instanceof Error ? err.message : String(err), variant: 'error' })
    return
  }
  if ('error' in result) {
    showToast({ title: 'Could not share profile', body: result.error, variant: 'error' })
    return
  }
  showToast({
    title: 'Share link created',
    body: result.webUrl,
    variant: 'success',
    duration: 12000,
    action: {
      label: 'Copy link',
      onClick: () => {
        navigator.clipboard.writeText(result.webUrl)
        showToast({ title: 'Copied to clipboard', variant: 'success', duration: 2500 })
      },
    },
  })
}

function requestDelete(): void {
  menuOpen.value = false
  emit('deleted', props.profile.id)
}

function onClickOutside(e: MouseEvent): void {
  if (dotsWrapRef.value && !dotsWrapRef.value.contains(e.target as Node)) {
    menuOpen.value = false
  }
  if (backupsMenuWorldId.value && !(e.target as HTMLElement).closest('.backups-menu-wrap')) {
    backupsMenuWorldId.value = null
  }
}

onMounted(() => {
  loadMods()
  document.addEventListener('mousedown', onClickOutside)
})
onUnmounted(() => document.removeEventListener('mousedown', onClickOutside))
</script>

<style lang="scss" scoped>
@import '../../styles/motion';

.profile-detail {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

// ── Header: back + title + search + browse ─────────────────────────────────────
.detail-header-row {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-shrink: 0;
  // The outer .profiles-page wrapper only has 2px of top padding, so the pixel-font
  // title's tall glyphs were getting clipped by its overflow-y:auto - pad here instead
  // of relying on the parent.
  padding-top: 8px;
  padding-bottom: 22px;
}

.back-chevron {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  background: none;
  border: none;
  color: $text-primary;
  cursor: pointer;
  transition: opacity $transition;
  &:hover { opacity: 0.7; }
}

.detail-title {
  display: flex;
  flex-direction: column;
  line-height: 1;
  flex-shrink: 0;
  font-family: 'Minecrafter', 'Plus Jakarta Sans', sans-serif;
  letter-spacing: 0.03em;
  text-transform: uppercase;
}

.detail-title-name {
  font-size: 24px;
  // The Minecrafter pixel font's glyph bounding box is taller than its line-height:1
  // metrics suggest, so at line-height:1 the ascenders get clipped by .profile-detail's
  // overflow:hidden - give it breathing room.
  line-height: 1.4;
  color: #fff;
}

.detail-title-sub {
  font-size: 13px;
  color: #F6AE35;
  margin-top: 3px;
}

.detail-edit-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: $radius-sm;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: background $transition, color $transition;
  &:hover { background: rgba(255, 255, 255, 0.12); color: #fff; }
}

.dots-menu-wrap {
  position: relative;
  flex-shrink: 0;
}

.dots-dropdown {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  min-width: 170px;
  background: rgba(13, 13, 13, 0.95);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: $radius-sm;
  overflow: hidden;
  z-index: 50;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
}

.dots-item {
  display: block;
  width: 100%;
  padding: 9px 14px;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.75);
  font-size: 12px;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: background 100ms, color 100ms;

  &:hover { background: #1a1a1a; color: #fff; }
}

.dots-sep {
  height: 1px;
  background: rgba(255, 255, 255, 0.1);
  margin: 4px 0;
}

.dots-dd-enter-active, .dots-dd-leave-active { transition: opacity 120ms ease, transform 120ms ease; }
.dots-dd-enter-from, .dots-dd-leave-to { opacity: 0; transform: translateY(-6px); }

.toolbar-search {
  display: flex;
  align-items: center;
  background: #13141A;
  border-radius: $radius-btn;
  height: 38px;
  padding: 0 20px;
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

.detail-search {
  flex: 1;
  min-width: 200px;
  margin-left: 8px;
}

.detail-compass-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 42px;
  height: 42px;
  flex-shrink: 0;
  background: #16161a;
  border: none;
  border-radius: 50%;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: background $transition, color $transition;
  &:hover { background: #232328; color: #fff; }
}

// ── "update all" link ────────────────────────────────────────────────────────
.update-all-link {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.6);
  font-size: 12px;
  cursor: pointer;
  padding: 0;
  margin-bottom: 14px;
  transition: color $transition;

  &:hover:not(:disabled) { color: #fff; }
  &:disabled { opacity: 0.5; cursor: wait; }

  svg.spinning { animation: spin 0.8s linear infinite; }
}

// ── Mods area ─────────────────────────────────────────────────────────────────
.mods-area {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

// ── Empty state ───────────────────────────────────────────────────────────────
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 48px 24px;
  height: 100%;
}

.empty-icon {
  width: 60px;
  height: 60px;
  object-fit: contain;
  opacity: 0.55;
  filter: grayscale(1);
}

.empty-text {
  font-style: italic;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.75);
  font-family: 'Inter', system-ui, sans-serif;
}

.empty-actions {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.import-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 16px;
  background: rgba(102, 102, 102, 0.3);
  border: 2px solid rgba(255, 255, 255, 0.18);
  border-radius: 12px;
  color: #d6d6d6;
  font-size: 10px;
  cursor: pointer;
  transition: background $transition, border-color $transition;

  &:hover {
    background: rgba(102, 102, 102, 0.45);
    border-color: rgba(255, 255, 255, 0.32);
  }
}

.browse-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 16px;
  background: rgba(0, 0, 0, 0.6);
  border: 2px solid rgba(255, 255, 255, 0.18);
  border-radius: 12px;
  color: #46d66d;
  font-size: 10px;
  cursor: pointer;
  transition: background $transition, border-color $transition;

  &:hover {
    background: rgba(0, 0, 0, 0.8);
    border-color: rgba(70, 214, 109, 0.4);
  }
}

.btn-icon {
  width: 14px;
  height: 14px;
  object-fit: contain;
  filter: brightness(0) invert(0.85);

  &--modrinth {
    filter: none;
  }
}

// ── Mods list - same style as explore tab ─────────────────────────────────────
.mods-list {
  display: flex;
  flex-direction: column;
}

.mod-row {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 16px 18px;
  background: rgba(10, 10, 11, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.04);
  transition: background 80ms, border-color 80ms;
  flex-shrink: 0;

  &:hover { background: rgba(20, 20, 22, 0.85); border-color: rgba(255, 255, 255, 0.09); }
}

.mod-icon-wrap { width: 72px; height: 72px; flex-shrink: 0; }

.mod-icon {
  width: 72px;
  height: 72px;
  object-fit: cover;
  image-rendering: pixelated;
  display: block;
}

.mod-icon-fallback {
  width: 72px;
  height: 72px;
  background: #1a1a1a;
  border: 1px solid #333;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 26px;
  color: #555;
  text-transform: uppercase;
}

.mod-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.mod-name {
  font-size: 18px;
  color: #d9d9d9;
  letter-spacing: 0.02em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mod-desc {
  font-size: 11px;
  color: #666;
  letter-spacing: 0.02em;
  line-height: 1.6;
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mod-enabled-toggle {
  display: flex;
  align-items: center;
  gap: 9px;
  flex-shrink: 0;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.85);
  font-size: 13px;
  cursor: pointer;
  padding: 0;
}

.toggle-check {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  color: transparent;
  transition: background $transition, color $transition;

  .mod-enabled-toggle.on & {
    background: #46d66d;
    color: #06210f;
  }
}

.update-btn {
  flex-shrink: 0;
  padding: 10px 24px;
  font-size: 11px;
  color: #ccc;
  background: #13141A;
  border: none;
  border-radius: $radius-btn;
  cursor: pointer;
  letter-spacing: 0.04em;
  transition: background 80ms, color 80ms;

  &:hover:not(:disabled) { background: #232328; color: #fff; }
  &:disabled { opacity: 0.5; cursor: wait; }
}

.mod-delete-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px;
  font-size: 11px;
  letter-spacing: 0.04em;
  color: rgba(255, 255, 255, 0.4);
  background: #13141A;
  border: none;
  border-radius: $radius-btn;
  cursor: pointer;
  transition: background 80ms, color 80ms;

  &:hover { background: rgba(220, 60, 60, 0.14); color: #ff6b6b; }

  &.confirm {
    padding: 10px 14px;
    background: rgba(220, 60, 60, 0.18);
    color: #ff6b6b;
  }
}

// ── Content/Worlds tab switcher (replaces the old static "Content" subtitle) ──
.detail-title-tabs {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-top: 3px;
}

.detail-title-tab {
  display: flex;
  align-items: center;
  gap: 5px;
  background: none;
  border: none;
  padding: 0;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.3);
  cursor: pointer;
  transition: color $transition;

  &:hover { color: rgba(255, 255, 255, 0.6); }
  &.active { color: #F6AE35; }
}

.detail-title-tab-count {
  font-size: 10px;
  color: inherit;
  opacity: 0.7;
}

// ── World rows (reuse .mod-row/.mod-icon-wrap/.mod-icon/.mod-icon-fallback) ───
.world-badge {
  display: inline-block;
  margin-left: 8px;
  padding: 2px 7px;
  font-size: 9px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  border-radius: $radius-sm;
  vertical-align: middle;

  &--hardcore { background: rgba(220, 60, 60, 0.18); color: #ff6b6b; }
}

.world-rename-input {
  width: 100%;
  max-width: 320px;
  background: #13141A;
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: $radius-sm;
  padding: 6px 10px;
  font-size: 16px;
  color: #fff;
  outline: none;

  &:focus { border-color: #F6AE35; }
}

.backups-menu-wrap {
  position: relative;
  flex-shrink: 0;
}

.backups-dropdown {
  width: 220px;
  max-height: 260px;
  overflow-y: auto;
}

.backups-empty {
  padding: 12px 14px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  line-height: 1.5;
  margin: 0;
}

.world-action-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  color: rgba(255, 255, 255, 0.5);
  background: #13141A;
  border: none;
  border-radius: $radius-btn;
  cursor: pointer;
  transition: background 80ms, color 80ms;

  &:hover:not(:disabled) { background: #232328; color: #fff; }
  &:disabled { opacity: 0.5; cursor: wait; }
}

// ── Screenshot gallery ──────────────────────────────────────────────────────
.screenshots-area {
  flex: 1;
  overflow-y: auto;
}

.screenshot-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 14px;
}

.screenshot-tile {
  position: relative;
  aspect-ratio: 16 / 9;
  overflow: hidden;
  background: #101012;
  border: 1px solid rgba(255, 255, 255, 0.06);
  cursor: pointer;

  &:hover .screenshot-overlay { opacity: 1; }
  &:hover .screenshot-thumb:not(.screenshot-thumb--loading) { transform: scale(1.04); }
}

.screenshot-thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  transition: transform 200ms ease;

  &--loading {
    display: flex;
    align-items: center;
    justify-content: center;

    .spinner { width: 20px; height: 20px; border-width: 2px; }
  }
}

.screenshot-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  padding: 10px;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.75), transparent 55%);
  opacity: 0;
  transition: opacity 120ms ease;
}

.screenshot-date {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.85);
  margin-bottom: 6px;
}

.screenshot-overlay-actions {
  display: flex;
  gap: 6px;
}

.screenshot-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  background: rgba(15, 15, 17, 0.85);
  border: none;
  border-radius: $radius-sm;
  color: rgba(255, 255, 255, 0.75);
  cursor: pointer;
  transition: background 80ms, color 80ms;

  &:hover { background: rgba(30, 30, 34, 0.95); color: #fff; }

  &--danger {
    &:hover { background: rgba(220, 60, 60, 0.5); color: #ff8a8a; }
    &.confirm { background: rgba(220, 60, 60, 0.7); color: #fff; }
  }
}

// ── Spinner ───────────────────────────────────────────────────────────────────
.mods-spinner {
  display: flex;
  justify-content: center;
  padding: 40px;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-top-color: $primary;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

// Mod row enter/leave (remove) - mirrors ModsPage's row-stagger transition
.row-stagger-enter-active {
  transition: opacity 240ms ease, transform 240ms cubic-bezier(0.25, 1, 0.5, 1);
}
.row-stagger-enter-from { opacity: 0; transform: translateY(8px); }
.row-stagger-leave-active { transition: opacity 120ms ease; }
.row-stagger-leave-to { opacity: 0; }
.row-stagger-move { transition: transform 220ms cubic-bezier(0.25, 1, 0.5, 1); }
</style>
