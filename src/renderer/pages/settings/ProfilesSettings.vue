<template>
  <div class="profiles-page">
  <div class="settings-section">
    <h2 class="section-heading">Profiles</h2>
    <p class="section-desc">Manage launch configurations for different Minecraft versions.</p>

    <!-- Profile list -->
    <div class="profiles-list">
      <div
        v-for="profile in launcherStore.profiles"
        :key="profile.id"
        class="profile-row"
        :class="{ active: profile.id === launcherStore.activeProfile?.id }"
      >
        <div class="profile-info">
          <span class="profile-name">{{ profile.name }}</span>
          <span class="profile-meta">
            {{ profile.version }}
            <span v-if="profile.loader !== 'vanilla'" class="loader-tag">{{ profile.loader }}</span>
            <span v-if="profile.useBejaClient" class="beja-tag">BejaClient</span>
          </span>
        </div>
        <div class="profile-actions">
          <button
            v-if="profile.id !== launcherStore.activeProfile?.id"
            class="action-btn"
            @click="launcherStore.setActiveProfile(profile.id)"
          >Select</button>
          <span v-else class="active-badge">Active</span>
          <button class="action-btn edit-btn" @click="startEdit(profile)">Edit</button>
          <button class="action-btn folder-btn" :title="`Open ${profile.name} folder`" @click="openProfileFolder(profile)">
            <svg width="11" height="11" viewBox="0 0 12 12" fill="none"><path d="M1 3h3l1.5-1.5H10a1 1 0 0 1 1 1V9a1 1 0 0 1-1 1H1a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/></svg>
          </button>
          <button class="action-btn delete-btn" @click="deleteProfile(profile.id)">Delete</button>
        </div>
      </div>

      <div v-if="launcherStore.profiles.length === 0" class="empty-state">
        No profiles yet. Create one to get started.
      </div>
    </div>

    <!-- Create / Edit form -->
    <div class="profile-form">
      <h3 class="form-title">{{ editing ? 'Edit Profile' : 'New Profile' }}</h3>

      <div class="form-grid">
        <div class="form-field">
          <label class="field-label">Name</label>
          <Input v-model="form.name" placeholder="My Profile" />
        </div>

        <div class="form-field">
          <label class="field-label">Minecraft Version</label>
          <select v-model="form.version" class="native-select">
            <option value="">Select version...</option>
            <option v-for="v in installedVersions" :key="v" :value="v">{{ v }}</option>
          </select>
        </div>

        <div class="form-field">
          <label class="field-label">Mod Loader</label>
          <select v-model="form.loader" class="native-select">
            <option value="vanilla">Vanilla</option>
            <option value="fabric">Fabric</option>
            <option value="forge">Forge</option>
            <option value="quilt">Quilt</option>
            <option value="neoforge">NeoForge</option>
          </select>
        </div>

        <div class="form-field">
          <label class="field-label">Min RAM</label>
          <div class="ram-row">
            <span class="ram-val">{{ formatRam(form.minRam) }}</span>
            <input v-model.number="form.minRam" type="range" min="256" :max="form.maxRam" step="256" class="ram-slider" />
          </div>
        </div>

        <div class="form-field">
          <label class="field-label">Max RAM</label>
          <div class="ram-row">
            <span class="ram-val">{{ formatRam(form.maxRam) }}</span>
            <input v-model.number="form.maxRam" type="range" :min="form.minRam" :max="maxRamLimit" step="256" class="ram-slider" />
          </div>
        </div>

        <div class="form-field full-width">
          <label class="field-label">Game Directory <span class="field-optional">(leave empty for default)</span></label>
          <div class="dir-row">
            <Input v-model="form.gameDir" placeholder="Uses global default…" class="dir-input" />
            <button type="button" class="browse-btn" @click="chooseGameDir">Browse</button>
          </div>
        </div>

        <div class="form-field full-width">
          <label class="field-label">Java Executable <span class="field-optional">(leave empty for default)</span></label>
          <div class="dir-row">
            <Input v-model="form.javaPath" placeholder="Uses global default…" class="dir-input" />
            <button type="button" class="browse-btn" @click="chooseJavaPath">Browse</button>
          </div>
        </div>

        <div class="form-field full-width">
          <label class="field-label">JVM Arguments <span class="field-optional">(optional)</span></label>
          <Input v-model="form.jvmArgs" placeholder="-XX:+UseG1GC -XX:MaxGCPauseMillis=200" />
        </div>

        <div class="form-field full-width">
          <div class="beja-toggle-row">
            <div class="beja-toggle-info">
              <span class="field-label">BejaClient</span>
              <span class="beja-toggle-desc">Inject BejaClient enhancements (Zoom, NoFog, HUD modules)</span>
            </div>
            <Toggle v-model="form.useBejaClient" />
          </div>
        </div>
      </div>

      <div v-if="formError" class="form-error">{{ formError }}</div>

      <div class="form-actions">
        <Button variant="primary" :loading="saving" @click="submit">
          {{ editing ? 'Save Changes' : 'Create Profile' }}
        </Button>
        <Button v-if="editing" variant="secondary" @click="cancelEdit">Cancel</Button>
      </div>
    </div>
  </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import Button from '../../components/common/Button.vue'
import Input from '../../components/common/Input.vue'
import Toggle from '../../components/common/Toggle.vue'
import { useLauncherStore } from '../../store/launcherStore'
import { useSettingsStore } from '../../store/settingsStore'
import type { LaunchProfile, LoaderType } from '../../types'

const launcherStore = useLauncherStore()
const settingsStore = useSettingsStore()

const installedVersions = ref<string[]>([])
const saving = ref(false)
const formError = ref('')
const editing = ref<string | null>(null)

const maxRamLimit = computed(() => {
  const mem = (window.navigator as any)?.deviceMemory ?? 8
  return Math.max(mem * 1024, 8192)
})

const defaultForm = () => ({
  name: '',
  version: '',
  loader: 'vanilla' as LoaderType,
  loaderVersion: '',
  minRam: settingsStore.settings.game.minRam,
  maxRam: settingsStore.settings.game.maxRam,
  jvmArgs: '',
  gameDir: '',
  javaPath: '',
  resolution: { width: 854, height: 480 },
  useBejaClient: false,
})

const form = reactive(defaultForm())

onMounted(async () => {
  installedVersions.value = await window.api.versions.listInstalled()
})

function formatRam(mb: number) {
  return mb >= 1024 ? `${(mb / 1024).toFixed(mb % 1024 === 0 ? 0 : 1)} GB` : `${mb} MB`
}

function startEdit(profile: LaunchProfile) {
  editing.value = profile.id
  Object.assign(form, {
    name: profile.name,
    version: profile.version,
    loader: profile.loader,
    loaderVersion: profile.loaderVersion,
    minRam: profile.minRam,
    maxRam: profile.maxRam,
    jvmArgs: profile.jvmArgs,
    gameDir: profile.gameDir,
    javaPath: profile.javaPath,
    resolution: { ...profile.resolution },
    useBejaClient: profile.useBejaClient ?? false,
  })
  formError.value = ''
}

function cancelEdit() {
  editing.value = null
  Object.assign(form, defaultForm())
  formError.value = ''
}

async function submit() {
  formError.value = ''
  if (!form.name.trim()) { formError.value = 'Name is required.'; return }
  if (!form.version) { formError.value = 'Select a Minecraft version.'; return }

  saving.value = true
  try {
    if (editing.value) {
      await launcherStore.updateProfile(editing.value, { ...form })
      cancelEdit()
    } else {
      await launcherStore.createProfile({ ...form })
      Object.assign(form, defaultForm())
    }
  } catch (e) {
    formError.value = String(e)
  } finally {
    saving.value = false
  }
}

async function openProfileFolder(profile: LaunchProfile) {
  const dir = profile.gameDir || settingsStore.settings.game.defaultGameDir
  if (dir) await window.api.system.openExternal(`file:///${dir}`)
}

async function chooseGameDir() {
  const dir = await window.api.settings.chooseDir()
  if (dir) form.gameDir = dir
}

async function chooseJavaPath() {
  const path = await window.api.settings.chooseJava()
  if (path) form.javaPath = path
}

async function deleteProfile(id: string) {
  const profile = launcherStore.profiles.find(p => p.id === id)
  if (!confirm(`Delete profile "${profile?.name ?? id}"? This cannot be undone.`)) return
  if (editing.value === id) cancelEdit()
  await launcherStore.deleteProfile(id)
}
</script>

<style lang="scss" scoped>
.profiles-page { padding: $sp-8; overflow-y: auto; flex: 1; }
.settings-section { display: flex; flex-direction: column; gap: $sp-6; max-width: 640px; }
.section-heading { font-size: 18px; font-weight: 800; color: $text-primary; }
.section-desc { font-size: 13px; color: $muted; margin-top: -$sp-4; }

// Profile list
.profiles-list { display: flex; flex-direction: column; gap: $sp-2; }

.profile-row {
  display: flex; align-items: center; gap: $sp-3; padding: $sp-3 $sp-4;
  background: $surface; border: 1px solid $border; border-radius: $radius;
  &.active { border-color: $border-strong; background: $surface-elevated; }
}

.profile-info { flex: 1; display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.profile-name { font-size: 13px; font-weight: 700; color: $text-primary; }
.profile-meta { font-size: 11px; color: $muted; display: flex; align-items: center; gap: $sp-2; }

.loader-tag {
  font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.07em;
  color: $text-secondary; background: $surface-elevated; padding: 1px 5px;
  border-radius: $radius-sm; border: 1px solid $border;
}

.beja-tag {
  font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.07em;
  color: $text-secondary; background: $surface-elevated; padding: 1px 5px;
  border-radius: $radius-sm; border: 1px solid $border;
}

.profile-actions { display: flex; align-items: center; gap: $sp-2; flex-shrink: 0; }

.action-btn {
  padding: 4px $sp-3; font-size: 11px; font-weight: 600; border-radius: $radius;
  cursor: pointer; border: 1px solid $border; background: $surface-elevated;
  color: $text-secondary; transition: all $transition;
  &:hover { border-color: $border-strong; color: $text-primary; }
  &.edit-btn:hover { border-color: $border-strong; color: $text-primary; }
  &.delete-btn { color: $text-secondary; border-color: $border; background: $surface-elevated;
    &:hover { background: $border; color: $text-primary; } }
}

.active-badge { font-size: 11px; font-weight: 700; color: $primary; padding: 4px $sp-3; }
.empty-state { padding: $sp-8; text-align: center; font-size: 13px; color: $muted; }

.folder-btn {
  padding: 4px 6px; font-size: 11px; font-weight: 600; border-radius: $radius;
  cursor: pointer; border: 1px solid $border; background: $surface-elevated;
  color: $muted; transition: all $transition;
  &:hover { border-color: $border-strong; color: $text-primary; }
}

.dir-row { display: flex; gap: $sp-2; }
.dir-input { flex: 1; min-width: 0; }
.browse-btn {
  padding: 6px $sp-3; font-size: 12px; font-weight: 600; color: $text-secondary;
  background: $surface-elevated; border: 1px solid $border; border-radius: $radius;
  cursor: pointer; white-space: nowrap; transition: all $transition; flex-shrink: 0;
  &:hover { border-color: $border-strong; color: $text-primary; }
}

// Form
.profile-form {
  display: flex; flex-direction: column; gap: $sp-4;
  padding: $sp-5; background: $surface; border: 1px solid $border; border-radius: $radius;
}

.form-title { font-size: 14px; font-weight: 700; color: $text-primary; }

.form-grid {
  display: grid; grid-template-columns: 1fr 1fr; gap: $sp-4;
}

.form-field {
  display: flex; flex-direction: column; gap: $sp-1;
  &.full-width { grid-column: 1 / -1; }
}

.field-label { font-size: 11px; font-weight: 600; color: $muted; text-transform: uppercase; letter-spacing: 0.06em; }
.field-optional { text-transform: none; font-weight: 400; }

.native-select {
  width: 100%; padding: 7px $sp-3; font-size: 13px; color: $text-primary;
  background: $surface-elevated; border: 1px solid $border; border-radius: $radius;
  cursor: pointer; outline: none; font-family: inherit;
  &:focus { border-color: $primary; }
  option { background: $surface-elevated; }
}

.ram-row { display: flex; align-items: center; gap: $sp-2; }
.ram-val { font-size: 12px; font-weight: 600; color: $text-primary; width: 52px; flex-shrink: 0; }
.ram-slider { flex: 1; accent-color: $primary; cursor: pointer; height: 4px; }

.form-error {
  font-size: 12px; color: $error; padding: $sp-2 $sp-3;
  background: $surface-elevated; border: 1px solid $border;
  border-radius: $radius;
}

.form-actions { display: flex; gap: $sp-3; }

.beja-toggle-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: $sp-3 $sp-4; background: $surface-elevated;
  border: 1px solid $border; border-radius: $radius;
}

.beja-toggle-info { display: flex; flex-direction: column; gap: 2px; }

.beja-toggle-desc { font-size: 11px; color: $muted; }
</style>
