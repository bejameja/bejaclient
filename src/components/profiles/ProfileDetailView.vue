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
        <span class="detail-title-sub">Content</span>
      </div>

      <button class="detail-edit-btn" title="Profile settings" @click="$emit('settings', profile.id)">
        <Icon name="pencil" :size="16" />
      </button>

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
    </div>

    <!-- Mods area ───────────────────────────────────────────── -->
    <div class="mods-area">
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
          </div>
        </TransitionGroup>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useLauncherStore } from '../../store/launcherStore'
import type { LaunchProfile, ModInfo } from '../../types'
import Icon from '../common/Icon.vue'
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
  mods.value = await window.api.mods.delete(props.profile.id, modId)
}

async function importMod(): Promise<void> {
  mods.value = await window.api.mods.install(props.profile.id)
}

// Manual fallback for mods whose own auto-updater didn't run — best-effort: guesses a
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

async function exportPack(): Promise<void> {
  menuOpen.value = false
  await window.api.profiles.exportPack(props.profile.id)
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
  // title's tall glyphs were getting clipped by its overflow-y:auto — pad here instead
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
  // overflow:hidden — give it breathing room.
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
  border-radius: 4px;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: background $transition, color $transition;
  &:hover { background: rgba(255, 255, 255, 0.12); color: #fff; }
}

.toolbar-search {
  display: flex;
  align-items: center;
  background: #16161a;
  border-radius: 999px;
  height: 42px;
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
  border-radius: 4px;
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
  border-radius: 4px;
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

// ── Mods list — same style as explore tab ─────────────────────────────────────
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
  background: #1c1c20;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 999px;
  cursor: pointer;
  letter-spacing: 0.04em;
  transition: background 80ms, border-color 80ms, color 80ms;

  &:hover:not(:disabled) { background: #26262c; border-color: rgba(255, 255, 255, 0.28); color: #fff; }
  &:disabled { opacity: 0.5; cursor: wait; }
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

// Mod row enter/leave (remove) — mirrors ModsPage's row-stagger transition
.row-stagger-enter-active {
  transition: opacity 240ms ease, transform 240ms cubic-bezier(0.25, 1, 0.5, 1);
}
.row-stagger-enter-from { opacity: 0; transform: translateY(8px); }
.row-stagger-leave-active { transition: opacity 120ms ease; }
.row-stagger-leave-to { opacity: 0; }
.row-stagger-move { transition: transform 220ms cubic-bezier(0.25, 1, 0.5, 1); }
</style>
