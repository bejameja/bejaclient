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

    <!-- ── MODULES ──────────────────────────────────────────────────────── -->
    <template v-if="activeTab === 'modules'">
      <div class="module-groups">
        <div v-for="group in moduleGroups" :key="group.category" class="module-group">
          <div class="module-group-label">{{ group.category }}</div>
          <div class="module-items">
            <div v-for="mod in group.modules" :key="mod.id" class="module-wrap">

              <!-- Row -->
              <div class="module-row" :class="{ active: mod.enabled, editing: editingModuleId === mod.id }">
                <div class="module-left">
                  <div class="module-dot" :style="mod.hasColor && mod.enabled ? { background: mod.color } : {}" />
                  <div class="module-info">
                    <span class="module-name">{{ mod.name }}</span>
                    <span class="module-desc">{{ mod.description }}</span>
                  </div>
                </div>
                <div class="module-right">
                  <span v-if="mod.bind && mod.bind !== 'None'" class="module-bind-badge">{{ mod.bind }}</span>
                  <button
                    class="module-gear-btn"
                    :class="{ active: editingModuleId === mod.id }"
                    title="Settings"
                    @click="toggleModuleEdit(mod.id)"
                  >
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <circle cx="12" cy="12" r="3"/>
                      <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
                    </svg>
                  </button>
                  <Toggle :model-value="mod.enabled" @update:model-value="val => toggleModule(mod.id, val)" />
                </div>
              </div>

              <!-- Settings panel -->
              <Transition name="edit-expand">
                <div v-if="editingModuleId === mod.id" class="module-edit-panel">

                  <!-- Keybind -->
                  <div class="msetting-row">
                    <span class="msetting-label">Keybind</span>
                    <button
                      class="keybind-btn"
                      :class="{ listening: listeningKeyFor === mod.id }"
                      @click="startKeyCapture(mod.id)"
                    >
                      {{ listeningKeyFor === mod.id ? 'Press a key…' : (mod.bind || 'None') }}
                    </button>
                    <button v-if="mod.bind && mod.bind !== 'None'" class="keybind-clear" @click="clearBind(mod.id)">✕</button>
                  </div>

                  <!-- Color -->
                  <div v-if="mod.hasColor" class="msetting-row">
                    <span class="msetting-label">Color</span>
                    <div class="color-wrap">
                      <input type="color" :value="mod.color" class="color-input" @input="e => setModuleColor(mod.id, (e.target as HTMLInputElement).value)" />
                      <span class="color-hex">{{ mod.color }}</span>
                    </div>
                  </div>

                  <!-- Extra settings -->
                  <div v-for="s in mod.settings" :key="s.id" class="msetting-row">
                    <span class="msetting-label">{{ s.label }}</span>
                    <div v-if="s.type === 'number'" class="slider-wrap">
                      <input
                        type="range"
                        :min="s.min"
                        :max="s.max"
                        :step="s.step ?? 0.1"
                        :value="s.value"
                        class="msetting-slider"
                        @input="e => setModuleSetting(mod.id, s.id, parseFloat((e.target as HTMLInputElement).value))"
                      />
                      <span class="slider-val">{{ Number(s.value).toFixed(s.step && s.step >= 1 ? 0 : 1) }}</span>
                    </div>
                    <Toggle
                      v-else-if="s.type === 'toggle'"
                      :model-value="s.value"
                      @update:model-value="val => setModuleSetting(mod.id, s.id, val)"
                    />
                    <select
                      v-else-if="s.type === 'select'"
                      :value="s.value"
                      class="edit-select msetting-select"
                      @change="e => setModuleSetting(mod.id, s.id, (e.target as HTMLSelectElement).value)"
                    >
                      <option v-for="opt in s.options" :key="opt" :value="opt">{{ opt }}</option>
                    </select>
                  </div>

                </div>
              </Transition>
            </div>
          </div>
        </div>
      </div>
    </template>

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

    <!-- ── MODPACKS – coming soon ───────────────────────────────────────── -->
    <div v-else-if="activeTab === 'modpacks'" class="coming-soon">
      <svg width="32" height="32" viewBox="0 0 32 32" fill="none"><circle cx="16" cy="16" r="14" stroke="currentColor" stroke-width="1.5"/><path d="M16 9v8" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><circle cx="16" cy="22" r="1.5" fill="currentColor"/></svg>
      <span class="cs-title">Coming Soon</span>
      <span class="cs-text">Modpack browsing will be available in a future update.</span>
    </div>

    <!-- ── BROWSE (mods / resourcepacks / shaders) ───────────────────── -->
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
  { id: 'modules',      label: 'Modules'        },
  { id: 'installed',    label: 'Installed'      },
  { id: 'mods',         label: 'Browse Mods'    },
  { id: 'modpacks',     label: 'Modpacks'       },
  { id: 'resourcepacks',label: 'Resource Packs' },
  { id: 'shaders',      label: 'Shaders'        },
]
const activeTab = ref('modules')

// ── client modules ────────────────────────────────────────────────────────────
interface ModuleSetting {
  id: string
  label: string
  type: 'number' | 'toggle' | 'select'
  value: any
  min?: number
  max?: number
  step?: number
  options?: string[]
}

interface ClientModule {
  id: string
  name: string
  description: string
  enabled: boolean
  bind: string
  hasColor: boolean
  color: string
  settings: ModuleSetting[]
}

interface ModuleGroup {
  category: string
  modules: ClientModule[]
}

const MODULES_DEFAULT: ClientModule[] = [
  {
    id: 'last-hit-reach',
    name: 'Last Hit Reach',
    description: 'Extends your reach for the finishing hit',
    enabled: false, bind: 'None', hasColor: false, color: '#ffffff',
    settings: [
      { id: 'reach', label: 'Reach Distance', type: 'number', value: 3.5, min: 3.0, max: 6.0, step: 0.1 },
    ],
  },
  {
    id: 'reply-mod',
    name: 'Reply Mod',
    description: 'Quickly reply to the last private message',
    enabled: false, bind: 'None', hasColor: false, color: '#ffffff',
    settings: [],
  },
  {
    id: 'zoom',
    name: 'Zoom',
    description: 'Optifine-style zoom key',
    enabled: false, bind: 'None', hasColor: false, color: '#ffffff',
    settings: [
      { id: 'level', label: 'Zoom Level', type: 'number', value: 4, min: 1, max: 10, step: 0.5 },
      { id: 'smooth', label: 'Smooth Zoom', type: 'toggle', value: true },
    ],
  },
  {
    id: 'crosshair',
    name: 'Crosshair Indicator',
    description: 'Custom crosshair with hit indicator',
    enabled: false, bind: 'None', hasColor: true, color: '#ffffff',
    settings: [
      { id: 'style', label: 'Style', type: 'select', value: 'Cross', options: ['Cross', 'Dot', 'Circle', 'Plus'] },
      { id: 'size', label: 'Size', type: 'number', value: 5, min: 1, max: 20, step: 1 },
      { id: 'thickness', label: 'Thickness', type: 'number', value: 2, min: 1, max: 6, step: 1 },
    ],
  },
  {
    id: 'hit-color',
    name: 'Hit Color',
    description: 'Tints entities when you hit them',
    enabled: false, bind: 'None', hasColor: true, color: '#ff4444',
    settings: [
      { id: 'duration', label: 'Duration (ms)', type: 'number', value: 150, min: 50, max: 500, step: 10 },
    ],
  },
  {
    id: 'combat-hitboxes',
    name: 'Combat Hitboxes',
    description: 'Shows entity hitboxes during combat',
    enabled: false, bind: 'None', hasColor: true, color: '#ff0000',
    settings: [
      { id: 'opacity', label: 'Opacity', type: 'number', value: 0.4, min: 0.1, max: 1.0, step: 0.05 },
      { id: 'only-targets', label: 'Only Targeted', type: 'toggle', value: true },
    ],
  },
  {
    id: 'chat-custom',
    name: 'Chat Custom',
    description: 'Customize the chat appearance',
    enabled: false, bind: 'None', hasColor: true, color: '#27ade0',
    settings: [
      { id: 'opacity', label: 'Background Opacity', type: 'number', value: 0.6, min: 0.0, max: 1.0, step: 0.05 },
      { id: 'width', label: 'Width', type: 'number', value: 320, min: 160, max: 600, step: 10 },
      { id: 'shadow', label: 'Text Shadow', type: 'toggle', value: true },
    ],
  },
  {
    id: 'saturation',
    name: 'Saturation',
    description: 'Shows your saturation level on the HUD',
    enabled: false, bind: 'None', hasColor: true, color: '#ffaa00',
    settings: [
      { id: 'position', label: 'Position', type: 'select', value: 'Above hunger', options: ['Above hunger', 'Below hunger', 'Next to hunger'] },
    ],
  },
  {
    id: 'momentum',
    name: 'Momentum',
    description: 'Displays your current movement speed',
    enabled: false, bind: 'None', hasColor: true, color: '#aaffaa',
    settings: [
      { id: 'unit', label: 'Unit', type: 'select', value: 'BPS', options: ['BPS', 'KM/H', 'M/S'] },
      { id: 'show-vertical', label: 'Show Vertical', type: 'toggle', value: false },
    ],
  },
  {
    id: 'block-outline',
    name: 'Block Outline',
    description: 'Custom outline around targeted blocks',
    enabled: false, bind: 'None', hasColor: true, color: '#ffffff',
    settings: [
      { id: 'width', label: 'Line Width', type: 'number', value: 2.0, min: 0.5, max: 5.0, step: 0.5 },
      { id: 'fill', label: 'Fill', type: 'toggle', value: false },
      { id: 'fill-opacity', label: 'Fill Opacity', type: 'number', value: 0.15, min: 0.0, max: 0.5, step: 0.05 },
    ],
  },
  {
    id: 'toggle-sprint',
    name: 'Toggle Sprint',
    description: 'Hold or toggle sprint with one key',
    enabled: false, bind: 'None', hasColor: false, color: '#ffffff',
    settings: [
      { id: 'mode', label: 'Mode', type: 'select', value: 'Toggle', options: ['Toggle', 'Hold'] },
    ],
  },
  {
    id: 'toggle-sneak',
    name: 'Toggle Sneak',
    description: 'Hold or toggle sneak with one key',
    enabled: false, bind: 'None', hasColor: false, color: '#ffffff',
    settings: [
      { id: 'mode', label: 'Mode', type: 'select', value: 'Toggle', options: ['Toggle', 'Hold'] },
    ],
  },
]

const MODULE_GROUPS: { category: string; ids: string[] }[] = [
  { category: 'Combat',    ids: ['last-hit-reach', 'hit-color', 'combat-hitboxes'] },
  { category: 'Visual',    ids: ['crosshair', 'block-outline', 'saturation', 'momentum'] },
  { category: 'Movement',  ids: ['toggle-sprint', 'toggle-sneak', 'zoom'] },
  { category: 'Utility',   ids: ['reply-mod', 'chat-custom'] },
]

function loadModules(): ClientModule[] {
  try {
    const raw = localStorage.getItem('bc_modules')
    if (!raw) return MODULES_DEFAULT.map(m => ({ ...m, settings: m.settings.map(s => ({ ...s })) }))
    const saved = JSON.parse(raw) as Record<string, Partial<ClientModule>>
    return MODULES_DEFAULT.map(def => {
      const s = saved[def.id]
      if (!s) return { ...def, settings: def.settings.map(x => ({ ...x })) }
      return {
        ...def,
        enabled: s.enabled ?? def.enabled,
        bind:    s.bind    ?? def.bind,
        color:   s.color   ?? def.color,
        settings: def.settings.map(ds => {
          const sv = s.settings?.find(x => x.id === ds.id)
          return { ...ds, value: sv?.value ?? ds.value }
        }),
      }
    })
  } catch { return MODULES_DEFAULT.map(m => ({ ...m, settings: m.settings.map(s => ({ ...s })) })) }
}

function saveModules() {
  const data: Record<string, any> = {}
  for (const m of clientModules.value) {
    data[m.id] = { enabled: m.enabled, bind: m.bind, color: m.color, settings: m.settings.map(s => ({ id: s.id, value: s.value })) }
  }
  localStorage.setItem('bc_modules', JSON.stringify(data))
}

const clientModules = ref<ClientModule[]>(loadModules())

const moduleGroups = computed<ModuleGroup[]>(() =>
  MODULE_GROUPS.map(g => ({
    category: g.category,
    modules: g.ids.map(id => clientModules.value.find(m => m.id === id)!).filter(Boolean),
  }))
)

const editingModuleId = ref<string | null>(null)
const listeningKeyFor = ref<string | null>(null)
let keyCaptureCleanup: (() => void) | null = null

function toggleModuleEdit(id: string) {
  if (editingModuleId.value === id) {
    editingModuleId.value = null
    stopKeyCapture()
  } else {
    editingModuleId.value = id
    stopKeyCapture()
  }
}

function toggleModule(id: string, val: boolean) {
  const m = clientModules.value.find(x => x.id === id)
  if (m) { m.enabled = val; saveModules() }
}

function setModuleColor(id: string, color: string) {
  const m = clientModules.value.find(x => x.id === id)
  if (m) { m.color = color; saveModules() }
}

function setModuleSetting(moduleId: string, settingId: string, value: any) {
  const m = clientModules.value.find(x => x.id === moduleId)
  if (!m) return
  const s = m.settings.find(x => x.id === settingId)
  if (s) { s.value = value; saveModules() }
}

function startKeyCapture(id: string) {
  stopKeyCapture()
  listeningKeyFor.value = id
  const handler = (e: KeyboardEvent) => {
    e.preventDefault()
    const key = e.key === ' ' ? 'Space' : e.key.length === 1 ? e.key.toUpperCase() : e.key
    const m = clientModules.value.find(x => x.id === id)
    if (m) { m.bind = key; saveModules() }
    stopKeyCapture()
  }
  window.addEventListener('keydown', handler, { once: true })
  keyCaptureCleanup = () => window.removeEventListener('keydown', handler)
}

function stopKeyCapture() {
  listeningKeyFor.value = null
  if (keyCaptureCleanup) { keyCaptureCleanup(); keyCaptureCleanup = null }
}

function clearBind(id: string) {
  const m = clientModules.value.find(x => x.id === id)
  if (m) { m.bind = 'None'; saveModules() }
}

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
  return 'mod'
})

const searchPlaceholder = computed(() => {
  if (activeTab.value === 'modpacks')      return 'Search modpacks…'
  if (activeTab.value === 'resourcepacks') return 'Search resource packs…'
  if (activeTab.value === 'shaders')       return 'Search shaders…'
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

// Reset + auto-load when switching browse tabs (not modpacks — Coming Soon)
watch(activeTab, tab => {
  if (tab === 'installed' || tab === 'modpacks') return
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

.coming-soon {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: $sp-8; gap: $sp-3; text-align: center; color: $muted;
}
.cs-title { font-size: 16px; font-weight: 700; color: $text-secondary; }
.cs-text  { font-size: 12px; color: $muted; max-width: 280px; line-height: 1.5; }

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

// ── Modules tab ───────────────────────────────────────────────────────────────
.module-groups {
  display: flex;
  flex-direction: column;
  gap: $sp-5;
}

.module-group-label {
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: $muted;
  padding: 0 2px $sp-1;
  border-bottom: 1px solid $border;
  margin-bottom: $sp-1;
}

.module-items {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.module-wrap {
  display: flex;
  flex-direction: column;
}

.module-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px $sp-4;
  background: $surface;
  border-radius: $radius;
  transition: background $transition, border-radius $transition;

  &:hover { background: $surface-elevated; }

  &.active .module-dot {
    background: var(--accent, #{$accent});
  }

  &.editing {
    border-radius: $radius $radius 0 0;
    background: $surface-elevated;
  }
}

.module-left {
  display: flex;
  align-items: center;
  gap: $sp-3;
  min-width: 0;
}

.module-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: $border;
  flex-shrink: 0;
  transition: background $transition;
}

.module-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.module-name {
  font-size: 13px;
  font-weight: 700;
  color: $text-primary;
}

.module-desc {
  font-size: 11px;
  color: $muted;
}

.module-right {
  display: flex;
  align-items: center;
  gap: $sp-2;
  flex-shrink: 0;
}

.module-bind-badge {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.05em;
  padding: 2px 7px;
  border-radius: $radius-sm;
  background: $surface-elevated;
  border: 1px solid $border;
  color: $text-secondary;
}

.module-gear-btn {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: $radius;
  color: $muted;
  cursor: pointer;
  transition: background $transition, color $transition;

  &:hover { background: $border; color: $text-primary; }
  &.active { background: $border; color: var(--accent, #{$accent}); }
}

.module-edit-panel {
  background: $surface-elevated;
  border: 1px solid $border;
  border-top: none;
  border-radius: 0 0 $radius $radius;
  padding: $sp-3 $sp-4;
  display: flex;
  flex-direction: column;
  gap: $sp-3;
}

.msetting-row {
  display: flex;
  align-items: center;
  gap: $sp-3;
}

.msetting-label {
  font-size: 11px;
  color: $muted;
  min-width: 120px;
  flex-shrink: 0;
}

.keybind-btn {
  padding: 4px 12px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.04em;
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius;
  color: $text-primary;
  cursor: pointer;
  transition: border-color $transition, background $transition;
  font-family: inherit;

  &:hover { border-color: $border-strong; }

  &.listening {
    border-color: var(--accent, #{$accent});
    color: var(--accent, #{$accent});
    animation: pulse-border 0.8s ease infinite alternate;
  }
}

@keyframes pulse-border {
  from { box-shadow: 0 0 0 0 rgba(39, 173, 224, 0); }
  to   { box-shadow: 0 0 0 3px rgba(39, 173, 224, 0.2); }
}

.keybind-clear {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: $radius-sm;
  color: $muted;
  cursor: pointer;
  font-size: 11px;
  transition: color $transition;
  &:hover { color: $text-primary; }
}

.color-wrap {
  display: flex;
  align-items: center;
  gap: $sp-2;
}

.color-input {
  width: 32px;
  height: 24px;
  border: 1px solid $border;
  border-radius: $radius;
  background: transparent;
  cursor: pointer;
  padding: 1px;
}

.color-hex {
  font-size: 11px;
  color: $text-secondary;
  font-family: monospace;
}

.slider-wrap {
  display: flex;
  align-items: center;
  gap: $sp-2;
  flex: 1;
}

.msetting-slider {
  flex: 1;
  accent-color: var(--accent, #{$accent});
  cursor: pointer;
}

.slider-val {
  font-size: 11px;
  font-weight: 700;
  color: $text-secondary;
  min-width: 30px;
  text-align: right;
}

.msetting-select {
  flex: 1;
}
</style>
