<template>
  <Teleport to="body">
    <Transition name="wizard-fade">
      <div v-if="store.wizardOpen" class="wizard-overlay" @click.self="maybeClose">
        <div class="wizard-modal">
          <div class="wizard-bg" />

          <button class="wizard-close-btn" :disabled="creating" @click="maybeClose">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>

          <!-- Step bar -->
          <div class="wizard-stepbar">
            <template v-for="(s, i) in STEPS" :key="i">
              <div class="step-node" :class="{ active: step === i, done: step > i }">
                <img v-if="s.icon" :src="s.icon" class="step-node-icon" />
                <span v-else class="step-node-text">{{ s.label[0] }}</span>
              </div>
              <div v-if="i < STEPS.length - 1" class="step-connector" :class="{ filled: step > i }" />
            </template>
          </div>

          <!-- ── Step 0: Identity ── -->
          <div v-if="step === 0 && !done" class="wizard-content">
            <div class="wiz-step-title">Profile Identity</div>

            <!-- Image picker -->
            <div class="identity-img-wrap" @click="pickImage">
              <img v-if="profileImage" :src="profileImage" class="identity-img" />
              <div v-else class="identity-img-empty">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/>
                  <circle cx="12" cy="13" r="4"/>
                </svg>
                <span>Add Image</span>
              </div>
              <div class="identity-img-overlay">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/>
                  <circle cx="12" cy="13" r="4"/>
                </svg>
              </div>
            </div>
            <input ref="imageInputRef" type="file" accept="image/png,image/jpeg,image/webp,image/gif" style="display:none" @change="onImageSelected" />

            <div class="wiz-fields">
              <div class="wiz-field">
                <label class="wiz-label">Name</label>
                <div class="wiz-input-wrap">
                  <input v-model="name" class="wiz-input" placeholder="My Instance" maxlength="32" autofocus />
                  <span class="input-counter">{{ name.length }}/32</span>
                </div>
              </div>
              <div class="wiz-field">
                <label class="wiz-label">Description <span class="wiz-label-opt">optional</span></label>
                <input v-model="description" class="wiz-input" placeholder="Short description…" maxlength="120" />
              </div>
            </div>

            <div class="wiz-nav">
              <button class="wiz-btn-primary" @click="step++">Next →</button>
            </div>
          </div>

          <!-- ── Step 1: Version ── -->
          <div v-if="step === 1 && !done" class="wizard-content">
            <div class="wiz-step-title">Minecraft Version</div>

            <div class="version-search-row">
              <input v-model="versionSearch" class="wiz-input wiz-input--sm" placeholder="Search…" />
              <label class="snapshot-toggle">
                <input type="checkbox" v-model="showSnapshots" />
                <span>Snapshots</span>
              </label>
            </div>

            <div v-if="loadingVersions" class="wiz-state">Loading versions…</div>
            <div v-else class="version-list">
              <button
                v-for="v in filteredVersions"
                :key="v.id"
                class="version-item"
                :class="{ selected: selectedVersion === v.id }"
                @click="selectedVersion = v.id"
              >
                <span class="version-id">{{ v.id }}</span>
                <span class="version-badge" :class="v.type === 'release' ? 'badge-release' : 'badge-snap'">{{ v.type === 'release' ? 'release' : 'snapshot' }}</span>
              </button>
              <div v-if="!filteredVersions.length" class="wiz-state">No versions match.</div>
            </div>

            <div class="wiz-nav">
              <button class="wiz-btn-secondary" @click="step--">← Back</button>
              <button class="wiz-btn-primary" :disabled="!selectedVersion" @click="step++">Next →</button>
            </div>
          </div>

          <!-- ── Step 2: Loader ── -->
          <div v-if="step === 2 && !done" class="wizard-content">
            <div class="wiz-step-title">Mod Loader</div>
            <p class="wiz-step-sub">{{ selectedVersion }}</p>

            <div class="loader-grid">
              <button
                v-for="l in LOADERS"
                :key="l.id"
                class="loader-card"
                :class="{ selected: selectedLoader === l.id }"
                @click="selectedLoader = l.id as LoaderType"
              >
                <img v-if="l.icon" :src="l.icon" class="loader-card-icon" />
                <span v-else class="loader-card-icon loader-card-icon--text">◈</span>
                <span class="loader-card-name">{{ l.name }}</span>
                <span v-if="selectedLoader === l.id" class="loader-card-check">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                </span>
              </button>
            </div>

            <!-- BejaClient toggle (only for supported versions + non-vanilla) -->
            <div v-if="selectedLoader !== 'vanilla' && isBejaSupported(selectedVersion)" class="beja-row">
              <div class="beja-switch" :class="{ on: useBejaClient }" @click="useBejaClient = !useBejaClient">
                <div class="beja-knob" />
              </div>
              <span class="beja-label">BejaClient</span>
              <span class="beja-hint">Performance client for {{ selectedVersion }}</span>
            </div>

            <div class="wiz-nav">
              <button class="wiz-btn-secondary" @click="step--">← Back</button>
              <button class="wiz-btn-primary" @click="step++">Next →</button>
            </div>
          </div>

          <!-- ── Step 3: Explore / Finalize ── -->
          <div v-if="step === 3 && !done" class="wizard-content wizard-explore">
            <!-- Profile summary -->
            <div class="explore-summary">
              <div class="summary-img-wrap" :style="{ background: loaderColor(selectedLoader) + '18' }">
                <img v-if="profileImage" :src="profileImage" class="summary-img" />
                <img v-else-if="loaderIconSrc(selectedLoader)" :src="loaderIconSrc(selectedLoader)!" class="summary-loader-icon" />
                <span v-else class="summary-loader-text">◈</span>
              </div>
              <div class="summary-info">
                <span class="summary-name">{{ resolvedName }}</span>
                <span v-if="description" class="summary-desc">{{ description }}</span>
                <div class="summary-meta">
                  <span class="summary-version">{{ selectedVersion }}</span>
                  <span class="summary-dot" />
                  <span class="summary-loader" :style="{ color: loaderColor(selectedLoader) }">{{ loaderDisplayName }}</span>
                  <template v-if="useBejaClient && selectedLoader !== 'vanilla' && isBejaSupported(selectedVersion)">
                    <span class="summary-dot" />
                    <span class="summary-bjc">BJC</span>
                  </template>
                </div>
              </div>
            </div>

            <!-- Explore button -->
            <button class="modrinth-explore-btn" @click="exploreAndCreate" :disabled="creating">
              <img :src="modrinthLogo" class="modrinth-logo" alt="Modrinth" />
              <div class="modrinth-btn-text">
                <span class="modrinth-btn-title">Explore Mods on Modrinth</span>
                <span class="modrinth-btn-sub">Browse thousands of mods, shaders &amp; resource packs</span>
              </div>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="opacity:0.4;flex-shrink:0"><polyline points="9 18 15 12 9 6"/></svg>
            </button>

            <div v-if="createError" class="wiz-error">{{ createError }}</div>

            <div class="wiz-nav">
              <button class="wiz-btn-secondary" @click="step--">← Back</button>
              <button class="wiz-btn-primary" :disabled="creating" @click="createAndDone">
                {{ creating ? 'Creating…' : 'Create' }}
              </button>
            </div>
          </div>

          <!-- ── Done ── -->
          <div v-if="done" class="wizard-content wizard-done">
            <div class="done-icon">
              <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
            </div>
            <h2>Profile Created!</h2>
            <p>{{ resolvedName }} is ready. Version will download on first launch.</p>
            <button class="wiz-btn-primary" @click="finish">Close</button>
          </div>

        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useLauncherStore } from '../../store/launcherStore'
import { useSettingsStore } from '../../store/settingsStore'
import type { LoaderType } from '../../types'

import iconName   from '../../assets/wizard/icons8-name-50.png'
import iconBook   from '../../assets/wizard/icons8-buch-50.png'
import iconPuzzle from '../../assets/wizard/icons8-puzzle-64.png'
import modrinthLogo from '../../assets/modrinth.png'
import loaderFabric   from '../../assets/loaders/fabric.png'
import loaderForge    from '../../assets/loaders/forge.png'
import loaderQuilt    from '../../assets/loaders/quilt.webp'
import loaderNeoforge from '../../assets/loaders/neoforge.png'

const store    = useLauncherStore()
const settings = useSettingsStore()
const router   = useRouter()

const STEPS = [
  { label: 'Identity', icon: iconName   },
  { label: 'Version',  icon: iconBook   },
  { label: 'Loader',   icon: iconPuzzle },
  { label: 'Explore',  icon: modrinthLogo },
]

const LOADERS = [
  { id: 'vanilla',  name: 'Vanilla',  icon: null },
  { id: 'fabric',   name: 'Fabric',   icon: loaderFabric },
  { id: 'forge',    name: 'Forge',    icon: loaderForge },
  { id: 'quilt',    name: 'Quilt',    icon: loaderQuilt },
  { id: 'neoforge', name: 'NeoForge', icon: loaderNeoforge },
]

const LOADER_COLORS: Record<string, string> = {
  vanilla: '#4ade80', fabric: '#c9a96e', forge: '#f97316',
  neoforge: '#e879f9', quilt: '#818cf8',
}
const LOADER_ICON_MAP: Record<string, string> = {
  fabric: loaderFabric, forge: loaderForge,
  quilt: loaderQuilt, neoforge: loaderNeoforge,
}

function loaderColor(l: string) { return LOADER_COLORS[l] ?? '#888' }
function loaderIconSrc(l: string): string | null { return LOADER_ICON_MAP[l] ?? null }

const BEJA_EXACT = new Set(['1.16.5', '1.18.2', '1.19.4'])
function isBejaSupported(id: string): boolean {
  if (!id) return false
  if (BEJA_EXACT.has(id)) return true
  const parts = id.split('.').map(Number)
  if (parts[0] === 1 && parts[1] === 20) return (parts[2] ?? 0) >= 1 && (parts[2] ?? 0) <= 6
  if (parts[0] === 1 && parts[1] >= 21) return true
  return false
}

// ── State ─────────────────────────────────────────────────────────────────────
const step           = ref(0)
const done           = ref(false)
const creating       = ref(false)
const createError    = ref('')

const name           = ref('')
const description    = ref('')
const profileImage   = ref<string | null>(null)
const imageInputRef  = ref<HTMLInputElement | null>(null)

const allVersions    = ref<{ id: string; type: string }[]>([])
const loadingVersions = ref(false)
const versionSearch  = ref('')
const showSnapshots  = ref(false)
const selectedVersion = ref('')

const selectedLoader  = ref<LoaderType>('fabric')
const useBejaClient   = ref(true)

const loaderDisplayName = computed(() => LOADERS.find(l => l.id === selectedLoader.value)?.name ?? selectedLoader.value)
const resolvedName = computed(() => name.value.trim() || (selectedVersion.value ? `${selectedVersion.value} ${loaderDisplayName.value}` : 'My Instance'))

const filteredVersions = computed(() => {
  return allVersions.value
    .filter(v => showSnapshots.value ? true : v.type === 'release')
    .filter(v => !versionSearch.value || v.id.includes(versionSearch.value))
})

watch(() => store.wizardOpen, async open => {
  if (!open) return
  reset()
  loadingVersions.value = true
  try {
    const manifest = await window.api.versions.listRemote()
    allVersions.value = manifest.versions
    const latest = manifest.versions.find((v: { id: string; type: string }) => v.type === 'release')
    if (latest) selectedVersion.value = latest.id
  } catch { /* non-fatal */ }
  finally { loadingVersions.value = false }
})

watch(selectedVersion, (ver) => {
  if (ver) useBejaClient.value = isBejaSupported(ver)
})

// ── Image ──────────────────────────────────────────────────────────────────────
function pickImage() { imageInputRef.value?.click() }

async function onImageSelected(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  profileImage.value = await resizeImage(file, 128)
  ;(e.target as HTMLInputElement).value = ''
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

// ── Create ────────────────────────────────────────────────────────────────────
function buildProfileData() {
  return {
    name:          resolvedName.value,
    description:   description.value || undefined,
    version:       selectedVersion.value,
    loader:        selectedLoader.value,
    loaderVersion: '',
    minRam:        settings.settings.game.minRam,
    maxRam:        settings.settings.game.maxRam,
    gameDir:       '',
    javaPath:      '',
    jvmArgs:       '',
    resolution:    { width: 854, height: 480 },
    useBejaClient: useBejaClient.value && selectedLoader.value !== 'vanilla' && isBejaSupported(selectedVersion.value),
    imageUrl:      profileImage.value ?? null,
  }
}

async function createAndDone() {
  createError.value = ''
  creating.value = true
  try {
    await store.createProfile(buildProfileData())
    done.value = true
  } catch (e) {
    createError.value = String(e)
  } finally {
    creating.value = false
  }
}

async function exploreAndCreate() {
  createError.value = ''
  creating.value = true
  try {
    const profile = await store.createProfile(buildProfileData())
    await store.setActiveProfile(profile.id)
    store.wizardOpen = false
    router.push('/mods')
  } catch (e) {
    createError.value = String(e)
    creating.value = false
  }
}

function finish() { store.wizardOpen = false }

function maybeClose() {
  if (creating.value) return
  store.wizardOpen = false
}

function reset() {
  step.value           = 0
  done.value           = false
  creating.value       = false
  createError.value    = ''
  name.value           = ''
  description.value    = ''
  profileImage.value   = null
  versionSearch.value  = ''
  showSnapshots.value  = false
  selectedVersion.value = ''
  selectedLoader.value = 'fabric'
  useBejaClient.value  = true
}
</script>

<style lang="scss" scoped>
.wizard-overlay {
  position: fixed;
  inset: 0;
  z-index: 1500;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0,0,0,0.75);
  backdrop-filter: blur(8px);
}

.wizard-bg {
  position: absolute;
  inset: 0;
  background: url('../../assets/wizard/wp8990100-minecraft-logo-4k-wallpapers.jpg') center/cover no-repeat;
  opacity: 0.12;
  pointer-events: none;
}

.wizard-modal {
  position: relative;
  width: 760px;
  max-width: 96vw;
  max-height: 90vh;
  background: #0a0a0c;
  border: 1px solid rgba(255,255,255,0.08);
  display: flex;
  flex-direction: column;
  align-items: center;
  overflow: hidden;
}

.wizard-close-btn {
  position: absolute;
  top: 14px;
  right: 14px;
  z-index: 10;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 50%;
  color: rgba(255,255,255,0.5);
  cursor: pointer;
  transition: all 150ms;
  &:hover:not(:disabled) { background: rgba(255,255,255,0.1); color: #fff; }
  &:disabled { opacity: 0.3; cursor: not-allowed; }
}

// ── Step bar ──────────────────────────────────────────────────────────────────
.wizard-stepbar {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  padding: 24px 0 20px;
}

.step-node {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  border: 2px solid rgba(255,255,255,0.12);
  background: rgba(255,255,255,0.04);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 180ms;

  .step-node-icon {
    width: 20px;
    height: 20px;
    object-fit: contain;
    opacity: 0.3;
    transition: opacity 180ms, filter 180ms;
  }
  .step-node-text {
    font-size: 14px;
    font-weight: 700;
    color: rgba(255,255,255,0.3);
    transition: color 180ms;
  }

  &.active {
    background: #fff;
    border-color: #fff;
    .step-node-icon { filter: invert(1); opacity: 1; }
    .step-node-text { color: #000; }
  }
  &.done {
    background: rgba(52,199,89,0.12);
    border-color: rgba(52,199,89,0.5);
    .step-node-icon { opacity: 0.9; }
    .step-node-text { color: #34c759; }
  }
}

.step-connector {
  width: 56px;
  height: 2px;
  background: rgba(255,255,255,0.1);
  transition: background 200ms;
  &.filled { background: rgba(52,199,89,0.4); }
}

// ── Content area ──────────────────────────────────────────────────────────────
.wizard-content {
  position: relative;
  z-index: 2;
  width: 100%;
  max-width: 520px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 0 28px 28px;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.08); border-radius: 4px; }
}

.wiz-step-title {
  font-size: 20px;
  font-weight: 700;
  color: #fff;
  letter-spacing: -0.01em;
  flex-shrink: 0;
}

.wiz-step-sub {
  font-size: 12px;
  color: rgba(255,255,255,0.35);
  margin: -10px 0 0;
  flex-shrink: 0;
}

// ── Step 0 — Identity ─────────────────────────────────────────────────────────
.identity-img-wrap {
  align-self: center;
  width: 100px;
  height: 100px;
  flex-shrink: 0;
  position: relative;
  cursor: pointer;
  border: 2px dashed rgba(255,255,255,0.12);
  transition: border-color 150ms;
  &:hover {
    border-color: rgba(255,255,255,0.3);
    .identity-img-overlay { opacity: 1; }
  }
}

.identity-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  image-rendering: pixelated;
  display: block;
}

.identity-img-empty {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: rgba(255,255,255,0.2);
  font-size: 10px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.identity-img-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0,0,0,0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 150ms;
  color: rgba(255,255,255,0.8);
}

.wiz-fields {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.wiz-field {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.wiz-label {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255,255,255,0.55);
  letter-spacing: 0.03em;
  text-transform: uppercase;
}

.wiz-label-opt {
  font-weight: 400;
  color: rgba(255,255,255,0.2);
  text-transform: none;
  letter-spacing: 0;
}

.wiz-input-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.wiz-input {
  width: 100%;
  padding: 10px 44px 10px 14px;
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 8px;
  color: #fff;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  box-sizing: border-box;
  transition: border-color 150ms;
  &:focus { border-color: rgba(255,255,255,0.3); }
  &::placeholder { color: rgba(255,255,255,0.2); }
  &--sm { padding: 8px 14px; font-size: 13px; }
}

.input-counter {
  position: absolute;
  right: 12px;
  font-size: 10px;
  color: rgba(255,255,255,0.2);
  pointer-events: none;
  white-space: nowrap;
}

// ── Step 1 — Version ──────────────────────────────────────────────────────────
.version-search-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.snapshot-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: rgba(255,255,255,0.4);
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  input { accent-color: #34c759; cursor: pointer; }
  &:hover { color: rgba(255,255,255,0.65); }
}

.version-list {
  flex: 1;
  min-height: 200px;
  max-height: 320px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 3px;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.08); border-radius: 4px; }
}

.version-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  background: rgba(255,255,255,0.03);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 7px;
  cursor: pointer;
  text-align: left;
  flex-shrink: 0;
  transition: all 120ms;
  &:hover { background: rgba(255,255,255,0.06); border-color: rgba(255,255,255,0.14); }
  &.selected { background: rgba(255,255,255,0.08); border-color: rgba(255,255,255,0.5); }
}

.version-id { flex: 1; font-size: 13px; font-weight: 600; color: #fff; }

.version-badge {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.06em;
  padding: 2px 6px;
  border-radius: 4px;
  text-transform: uppercase;
  &.badge-release { color: #34c759; background: rgba(52,199,89,0.1); }
  &.badge-snap    { color: #f97316; background: rgba(249,115,22,0.1); }
}

// ── Step 2 — Loader ───────────────────────────────────────────────────────────
.loader-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 8px;
}

.loader-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 14px 10px 12px;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.07);
  border-radius: 10px;
  cursor: pointer;
  position: relative;
  transition: all 150ms;

  &:hover {
    background: rgba(255,255,255,0.08);
    border-color: rgba(255,255,255,0.18);
  }
  &.selected {
    background: rgba(255,255,255,0.1);
    border-color: rgba(255,255,255,0.45);
  }
}

.loader-card-icon {
  width: 32px;
  height: 32px;
  object-fit: contain;
  &--text { font-size: 22px; line-height: 1; color: #4ade80; }
}

.loader-card-name {
  font-size: 11px;
  font-weight: 600;
  color: rgba(255,255,255,0.6);
  letter-spacing: 0.02em;
  .selected & { color: #fff; }
}

.loader-card-check {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: rgba(52,199,89,0.2);
  border: 1px solid rgba(52,199,89,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #34c759;
}

.beja-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: rgba(249,115,22,0.04);
  border: 1px solid rgba(249,115,22,0.14);
  border-radius: 8px;
  flex-shrink: 0;
}

.beja-switch {
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background: rgba(255,255,255,0.1);
  border: 1px solid rgba(255,255,255,0.15);
  position: relative;
  cursor: pointer;
  flex-shrink: 0;
  transition: background 150ms, border-color 150ms;
  &.on { background: rgba(249,115,22,0.7); border-color: rgba(249,115,22,0.9); }
}

.beja-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: rgba(255,255,255,0.5);
  transition: transform 150ms, background 150ms;
  .on & { transform: translateX(16px); background: #fff; }
}

.beja-label { font-size: 13px; font-weight: 700; color: rgba(249,115,22,0.9); }
.beja-hint  { font-size: 11px; color: rgba(255,255,255,0.25); margin-left: auto; }

// ── Step 3 — Explore ──────────────────────────────────────────────────────────
.wizard-explore { gap: 14px; }

.explore-summary {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 14px;
  background: rgba(255,255,255,0.03);
  border: 1px solid rgba(255,255,255,0.07);
  border-radius: 10px;
  flex-shrink: 0;
}

.summary-img-wrap {
  width: 52px;
  height: 52px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  overflow: hidden;
}

.summary-img { width: 100%; height: 100%; object-fit: cover; image-rendering: pixelated; }
.summary-loader-icon { width: 44px; height: 44px; object-fit: contain; }
.summary-loader-text { font-size: 26px; color: #4ade80; }

.summary-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.summary-name {
  font-size: 15px;
  font-weight: 700;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.summary-desc {
  font-size: 11px;
  color: rgba(255,255,255,0.35);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.summary-meta {
  display: flex;
  align-items: center;
  gap: 6px;
}

.summary-version { font-size: 11px; color: rgba(255,255,255,0.35); }
.summary-dot { width: 2px; height: 2px; border-radius: 50%; background: rgba(255,255,255,0.2); flex-shrink: 0; }
.summary-loader { font-size: 11px; font-weight: 600; }
.summary-bjc { font-size: 10px; font-weight: 700; color: #f97316; letter-spacing: 0.05em; }

.modrinth-explore-btn {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px 20px;
  background: rgba(27, 217, 106, 0.05);
  border: 1px solid rgba(27, 217, 106, 0.2);
  border-radius: 12px;
  cursor: pointer;
  text-align: left;
  flex-shrink: 0;
  transition: all 150ms;
  width: 100%;

  &:hover:not(:disabled) {
    background: rgba(27, 217, 106, 0.09);
    border-color: rgba(27, 217, 106, 0.38);
    transform: translateY(-1px);
  }
  &:disabled { opacity: 0.4; cursor: not-allowed; }
}

.modrinth-logo {
  width: 40px;
  height: 40px;
  object-fit: contain;
  flex-shrink: 0;
  border-radius: 8px;
}

.modrinth-btn-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex: 1;
  min-width: 0;
}

.modrinth-btn-title {
  font-size: 14px;
  font-weight: 700;
  color: #fff;
}

.modrinth-btn-sub {
  font-size: 12px;
  color: rgba(255,255,255,0.4);
}

// ── Nav ───────────────────────────────────────────────────────────────────────
.wiz-nav {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: auto;
  padding-top: 4px;
  flex-shrink: 0;
}

.wiz-btn-primary {
  padding: 10px 26px;
  background: #fff;
  color: #000;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: background 150ms;
  &:hover:not(:disabled) { background: rgba(255,255,255,0.85); }
  &:disabled { opacity: 0.35; cursor: not-allowed; }
}

.wiz-btn-secondary {
  padding: 10px 20px;
  background: rgba(255,255,255,0.06);
  color: rgba(255,255,255,0.6);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 150ms;
  &:hover { background: rgba(255,255,255,0.1); color: #fff; }
}

// ── Shared ────────────────────────────────────────────────────────────────────
.wiz-state {
  font-size: 12px;
  color: rgba(255,255,255,0.3);
  text-align: center;
  padding: 16px 0;
}

.wiz-error {
  font-size: 12px;
  color: #e05050;
  padding: 8px 12px;
  background: rgba(224,80,80,0.07);
  border: 1px solid rgba(224,80,80,0.2);
  border-radius: 8px;
  flex-shrink: 0;
}

// ── Done ──────────────────────────────────────────────────────────────────────
.wizard-done {
  align-items: center;
  justify-content: center;
  text-align: center;
}

.done-icon {
  width: 60px;
  height: 60px;
  border-radius: 50%;
  background: rgba(52,199,89,0.1);
  border: 1px solid rgba(52,199,89,0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #34c759;
}

.wizard-done h2 { font-size: 20px; font-weight: 700; color: #fff; margin: 0; }
.wizard-done p  { font-size: 12px; color: rgba(255,255,255,0.4); margin: 0; max-width: 320px; line-height: 1.5; }

// ── Transition ────────────────────────────────────────────────────────────────
.wizard-fade-enter-active, .wizard-fade-leave-active { transition: opacity 180ms ease; }
.wizard-fade-enter-from, .wizard-fade-leave-to { opacity: 0; }
</style>
