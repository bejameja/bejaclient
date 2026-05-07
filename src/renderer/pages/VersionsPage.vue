<template>
  <div class="versions-page page-content">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">Versions</h1>
        <span class="page-subtitle">Manage Minecraft installations</span>
      </div>
      <div class="header-right">
        <div class="filter-group">
          <button
            v-for="f in typeFilters"
            :key="f.value"
            class="filter-btn"
            :class="{ active: activeFilter === f.value }"
            @click="activeFilter = f.value"
          >
            {{ f.label }}
          </button>
        </div>
        <Button variant="primary" size="sm" @click="showInstallModal = true">
          <template #icon>
            <svg width="12" height="12" viewBox="0 0 12 12">
              <line x1="6" y1="1" x2="6" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
              <line x1="1" y1="6" x2="11" y2="6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </template>
          Install Version
        </Button>
      </div>
    </div>

    <div v-if="installProgress" class="progress-bar-wrapper">
      <div class="progress-label">{{ installProgress.task }}</div>
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{ width: `${installProgress.total > 0 ? (installProgress.progress / installProgress.total) * 100 : 0}%` }"
        />
      </div>
    </div>

    <div class="version-list">
      <div
        v-for="v in filteredVersions"
        :key="v.id"
        class="version-item"
        :class="{ installed: v.installed }"
      >
        <div class="version-left">
          <div class="version-status" :class="{ installed: v.installed }">
            <svg v-if="v.installed" width="10" height="8" viewBox="0 0 10 8">
              <path d="M1 4L4 7L9 1" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </div>
          <div class="version-info">
            <span class="version-id">{{ v.id }}</span>
            <span class="version-date">{{ formatDate(v.releaseTime) }}</span>
          </div>
        </div>
        <div class="version-right">
          <span class="badge" :class="`badge-${v.type === 'release' ? 'release' : 'snapshot'}`">
            {{ v.type }}
          </span>
          <div class="version-actions">
            <Button
              v-if="!v.installed"
              variant="secondary"
              size="sm"
              :loading="installing === v.id"
              @click="installVersion(v.id)"
            >
              Install
            </Button>
            <Button
              v-if="v.installed"
              variant="ghost"
              size="sm"
              @click="createProfileFor(v.id)"
            >
              New Profile
            </Button>
            <Button
              v-if="v.installed"
              variant="danger"
              size="sm"
              @click="deleteVersion(v.id)"
            >
              Remove
            </Button>
          </div>
        </div>
      </div>

      <div v-if="filteredVersions.length === 0 && !loading" class="empty-state">
        <span class="empty-icon">📦</span>
        <span class="empty-title">No versions found</span>
        <span class="empty-text">Install a Minecraft version to get started</span>
      </div>

      <div v-if="loading" class="loading-state">
        <span class="loading-dot" />
        <span>Loading versions...</span>
      </div>
    </div>

    <!-- Install Modal -->
    <Transition name="fade">
      <div v-if="showInstallModal" class="modal-overlay" @click.self="showInstallModal = false">
        <div class="modal">
          <div class="modal-header">
            <h2>Install Minecraft</h2>
            <button class="modal-close" @click="showInstallModal = false">
              <svg width="12" height="12" viewBox="0 0 12 12">
                <line x1="1" y1="1" x2="11" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                <line x1="11" y1="1" x2="1" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
              </svg>
            </button>
          </div>

          <div class="modal-body">
            <Select
              v-model="selectedVersion"
              :options="releaseVersionOptions"
              label="Minecraft Version"
              placeholder="Select version..."
            />
            <Select
              v-model="selectedLoader"
              :options="loaderOptions"
              label="Mod Loader"
            />
            <Select
              v-if="selectedLoader === 'fabric' && fabricVersionOptions.length"
              v-model="selectedFabricVersion"
              :options="fabricVersionOptions"
              label="Fabric Loader Version"
            />
          </div>

          <div class="modal-footer">
            <Button variant="secondary" @click="showInstallModal = false">Cancel</Button>
            <Button
              variant="primary"
              :disabled="!selectedVersion"
              :loading="installing !== null"
              @click="confirmInstall"
            >
              Install
            </Button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import Button from '../components/common/Button.vue'
import Select from '../components/common/Select.vue'
import { useLauncherStore } from '../store/launcherStore'
import { useSettingsStore } from '../store/settingsStore'
import type { RemoteVersion, VersionManifest } from '../types'

const launcherStore = useLauncherStore()
const settingsStore = useSettingsStore()

const loading = ref(false)
const installing = ref<string | null>(null)
const manifest = ref<VersionManifest | null>(null)
const installedIds = ref<string[]>([])
const activeFilter = ref<'all' | 'release' | 'snapshot' | 'installed'>('all')
const showInstallModal = ref(false)
const selectedVersion = ref<string | null>(null)
const selectedLoader = ref<string>('vanilla')
const fabricVersions = ref<{ loader: { version: string; stable: boolean } }[]>([])
const selectedFabricVersion = ref<string | null>(null)

const installProgress = computed(() => launcherStore.installProgress)

const typeFilters: { label: string; value: 'all' | 'release' | 'snapshot' | 'installed' }[] = [
  { label: 'All', value: 'all' },
  { label: 'Release', value: 'release' },
  { label: 'Snapshot', value: 'snapshot' },
  { label: 'Installed', value: 'installed' },
]

const loaderOptions = [
  { label: 'Vanilla', value: 'vanilla' },
  { label: 'Fabric', value: 'fabric' },
  { label: 'Forge', value: 'forge' },
  { label: 'Quilt', value: 'quilt' },
]

const allVersions = computed<(RemoteVersion & { installed: boolean })[]>(() => {
  if (!manifest.value) return []
  return manifest.value.versions.map(v => ({
    ...v,
    installed: installedIds.value.includes(v.id),
  }))
})

const filteredVersions = computed(() => {
  return allVersions.value.filter(v => {
    if (activeFilter.value === 'all') return v.type === 'release' || v.type === 'snapshot'
    if (activeFilter.value === 'release') return v.type === 'release'
    if (activeFilter.value === 'snapshot') return v.type === 'snapshot'
    if (activeFilter.value === 'installed') return v.installed
    return true
  })
})

const releaseVersionOptions = computed(() =>
  (manifest.value?.versions ?? [])
    .filter(v => v.type === 'release' || v.type === 'snapshot')
    .slice(0, 80)
    .map(v => ({ label: v.id, value: v.id })),
)

const fabricVersionOptions = computed(() =>
  fabricVersions.value.slice(0, 20).map(f => ({
    label: f.loader.version + (f.loader.stable ? '' : ' (unstable)'),
    value: f.loader.version,
  })),
)

async function loadData() {
  loading.value = true
  try {
    const [m, installed] = await Promise.all([
      window.api.versions.listRemote(),
      window.api.versions.listInstalled(),
    ])
    manifest.value = m
    installedIds.value = installed
  } finally {
    loading.value = false
  }
}

watch(selectedVersion, async ver => {
  if (ver && selectedLoader.value === 'fabric') {
    fabricVersions.value = await window.api.versions.listFabricVersions(ver)
    selectedFabricVersion.value = fabricVersions.value[0]?.loader?.version ?? null
  }
})

watch(selectedLoader, async loader => {
  if (loader === 'fabric' && selectedVersion.value) {
    fabricVersions.value = await window.api.versions.listFabricVersions(selectedVersion.value)
    selectedFabricVersion.value = fabricVersions.value[0]?.loader?.version ?? null
  } else {
    fabricVersions.value = []
    selectedFabricVersion.value = null
  }
})

async function installVersion(versionId: string) {
  installing.value = versionId
  try {
    installedIds.value = await window.api.versions.install(versionId, 'vanilla')
  } finally {
    installing.value = null
  }
}

async function confirmInstall() {
  if (!selectedVersion.value) return
  installing.value = selectedVersion.value
  showInstallModal.value = false
  try {
    installedIds.value = await window.api.versions.install(
      selectedVersion.value,
      selectedLoader.value,
      selectedLoader.value === 'fabric' ? (selectedFabricVersion.value ?? undefined) : undefined,
    )
  } finally {
    installing.value = null
  }
}

async function deleteVersion(versionId: string) {
  installedIds.value = await window.api.versions.delete(versionId)
}

async function createProfileFor(versionId: string) {
  await launcherStore.createProfile({
    name: `Minecraft ${versionId}`,
    version: versionId,
    loader: 'vanilla',
    loaderVersion: '',
    gameDir: settingsStore.settings.game.defaultGameDir,
    minRam: 512,
    maxRam: settingsStore.settings.game.maxRam,
    javaPath: settingsStore.settings.game.defaultJavaPath,
    jvmArgs: '',
    resolution: settingsStore.settings.game.resolution,
    useBejaClient: false,
  })
}

function formatDate(iso: string) {
  try {
    return new Date(iso).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
  } catch {
    return iso
  }
}

onMounted(loadData)
</script>

<style lang="scss" scoped>
.versions-page {
  display: flex;
  flex-direction: column;
  gap: $sp-5;
}

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.header-left {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.page-title {
  font-size: 20px;
  font-weight: 800;
  color: $text-primary;
}

.page-subtitle {
  font-size: 12px;
  color: $muted;
}

.header-right {
  display: flex;
  align-items: center;
  gap: $sp-3;
}

.filter-group {
  display: flex;
  gap: 1px;
  background: $border;
  padding: 1px;
  border-radius: $radius;
}

.filter-btn {
  padding: 4px $sp-3;
  font-size: 12px;
  font-weight: 600;
  color: $text-secondary;
  background: transparent;
  border: none;
  border-radius: $radius-sm;
  cursor: pointer;
  transition: background $transition, color $transition;

  &:hover { color: $text-primary; }
  &.active {
    background: $surface-elevated;
    color: $text-primary;
  }
}

.progress-bar-wrapper {
  display: flex;
  flex-direction: column;
  gap: $sp-1;
  padding: $sp-3;
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius;
}

.progress-label {
  font-size: 12px;
  color: $text-secondary;
}

.progress-bar {
  height: 3px;
  background: $border;
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: $text-secondary;
  transition: width 0.15s ease;
}

.version-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.version-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: $sp-3 $sp-4;
  background: $surface;
  border-radius: $radius;
  transition: background $transition;

  &:hover {
    background: $surface-elevated;
  }

  &.installed {
    border-left: 2px solid $border-strong;
  }
}

.version-left {
  display: flex;
  align-items: center;
  gap: $sp-3;
}

.version-status {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid $border;
  border-radius: 50%;
  color: $muted;
  flex-shrink: 0;

  &.installed {
    background: $surface-elevated;
    border-color: $border-strong;
    color: $text-secondary;
  }
}

.version-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.version-id {
  font-size: 13px;
  font-weight: 600;
  color: $text-primary;
}

.version-date {
  font-size: 11px;
  color: $muted;
}

.version-right {
  display: flex;
  align-items: center;
  gap: $sp-3;
}

.version-actions {
  display: flex;
  gap: $sp-2;
}

.loading-state {
  display: flex;
  align-items: center;
  gap: $sp-3;
  padding: $sp-8;
  color: $muted;
  font-size: 13px;
}

.loading-dot {
  width: 8px;
  height: 8px;
  background: $border-strong;
  border-radius: 50%;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: $surface;
  border: 1px solid $border-strong;
  border-radius: $radius;
  width: 380px;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: $sp-4 $sp-5;
  border-bottom: 1px solid $border;

  h2 {
    font-size: 15px;
    font-weight: 700;
    color: $text-primary;
  }
}

.modal-close {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: $muted;
  background: transparent;
  border: none;
  border-radius: $radius;
  cursor: pointer;
  transition: background $transition, color $transition;

  &:hover {
    background: $surface-elevated;
    color: $text-primary;
  }
}

.modal-body {
  padding: $sp-5;
  display: flex;
  flex-direction: column;
  gap: $sp-4;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: $sp-2;
  padding: $sp-4 $sp-5;
  border-top: 1px solid $border;
}
</style>
