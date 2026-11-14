<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="modelValue" class="of-overlay" @click.self="close">
        <div class="of-modal">
          <div class="of-header">
            <span class="of-title">OptiFine</span>
            <button class="of-close" title="Close" @click="close">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
          <!-- Step 1: pick version + build ─────────────────────────────────── -->
          <template v-if="!installGameDir">
            <p class="of-sub">Pick a Minecraft version, then a build. Runs standalone - no bundled Sodium/Lithium, since OptiFine's renderer conflicts with it.</p>

            <div class="of-body">
              <!-- MC version column -->
              <div class="of-col of-col--versions">
                <span class="of-col-label">Minecraft version</span>
                <div v-if="loadingVersions" class="of-loading"><div class="of-spinner" /></div>
                <div v-else-if="loadError" class="of-error">{{ loadError }}</div>
                <div v-else class="of-list">
                  <button
                    v-for="v in mcVersions"
                    :key="v"
                    class="of-row"
                    :class="{ active: v === selectedVersion }"
                    @click="selectVersion(v)"
                  >{{ v }}</button>
                </div>
              </div>

              <!-- Build column -->
              <div class="of-col of-col--builds">
                <span class="of-col-label">OptiFine build</span>
                <div v-if="!selectedVersion" class="of-empty">Pick a version first</div>
                <div v-else-if="loadingBuilds" class="of-loading"><div class="of-spinner" /></div>
                <div v-else-if="buildError" class="of-error">{{ buildError }}</div>
                <div v-else-if="!builds.length" class="of-empty">No OptiFine build for this version</div>
                <div v-else class="of-list">
                  <button
                    v-for="b in builds"
                    :key="b.filename"
                    class="of-row"
                    :class="{ active: b.filename === selectedBuild?.filename }"
                    @click="selectedBuild = b"
                  >OptiFine {{ b.buildTag }}<span v-if="b.preview" class="of-preview-tag">Preview</span></button>
                </div>
              </div>
            </div>

            <p v-if="createError" class="of-create-error">{{ createError }}</p>

            <div class="of-actions">
              <button class="of-btn of-btn--cancel" @click="close">Cancel</button>
              <button class="of-btn of-btn--create" :disabled="!selectedBuild || creating" @click="create">
                {{ creating ? 'Downloading…' : 'Create profile' }}
              </button>
            </div>
          </template>

          <!-- Step 2: real OptiFine installer is open, walk the user through it ── -->
          <template v-else>
            <p class="of-sub">
              The real OptiFine installer just opened in its own window. Click <strong>Install</strong> in it - if it
              doesn't already point at the folder below, paste it in via its "Edit" field first.
            </p>
            <div class="of-path-row">
              <code class="of-path">{{ installGameDir }}</code>
              <button class="of-copy-btn" @click="copyPath">{{ copied ? 'Copied' : 'Copy' }}</button>
            </div>
            <p class="of-sub of-sub--muted">
              Once it's done, close the installer and hit Play on the "{{ createdProfileName }}" profile - it launches straight away.
            </p>
            <div class="of-actions">
              <button class="of-btn of-btn--create" @click="close">Done</button>
            </div>
          </template>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useLauncherStore } from '../../store/launcherStore'
import { useSettingsStore } from '../../store/settingsStore'
import type { OptifineBuild, LaunchProfile } from '../../types'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [boolean]; created: [LaunchProfile] }>()

const store    = useLauncherStore()
const settings = useSettingsStore()

const mcVersions      = ref<string[]>([])
const loadingVersions  = ref(false)
const loadError        = ref('')

const selectedVersion  = ref<string | null>(null)
const builds           = ref<OptifineBuild[]>([])
const loadingBuilds     = ref(false)
const buildError        = ref('')
const selectedBuild     = ref<OptifineBuild | null>(null)

const creating     = ref(false)
const createError  = ref('')

const installGameDir     = ref('')
const createdProfileName = ref('')
const copied              = ref(false)

async function copyPath(): Promise<void> {
  await navigator.clipboard.writeText(installGameDir.value)
  copied.value = true
  setTimeout(() => { copied.value = false }, 1500)
}

async function loadVersions(): Promise<void> {
  loadingVersions.value = true
  loadError.value = ''
  try {
    mcVersions.value = await window.api.optifine.listMcVersions()
  } catch (e) {
    loadError.value = 'Could not reach optifine.net - try again later.'
  } finally {
    loadingVersions.value = false
  }
}

async function selectVersion(v: string): Promise<void> {
  selectedVersion.value = v
  selectedBuild.value = null
  builds.value = []
  loadingBuilds.value = true
  buildError.value = ''
  try {
    builds.value = await window.api.optifine.listBuilds(v)
    selectedBuild.value = builds.value[0] ?? null
  } catch (e) {
    buildError.value = 'Could not load builds for this version.'
  } finally {
    loadingBuilds.value = false
  }
}

async function create(): Promise<void> {
  if (!selectedVersion.value || !selectedBuild.value) return
  creating.value = true
  createError.value = ''
  try {
    const profile = await store.createProfile({
      name:           `OptiFine ${selectedVersion.value}`,
      version:        selectedVersion.value,
      loader:         'optifine',
      loaderVersion:  selectedBuild.value.filename,
      minRam:         settings.settings.game.minRam,
      maxRam:         settings.settings.game.maxRam,
      gameDir:        '',
      javaPath:       '',
      jvmArgs:        '',
      resolution:     { width: 854, height: 480 },
      useBejaClient:  false,
      isolateProfile: false,
    })
    emit('created', profile)
    createdProfileName.value = profile.name
    installGameDir.value = await window.api.optifine.launchInstaller(selectedVersion.value, selectedBuild.value.filename)
  } catch (e) {
    createError.value = 'Failed to create the profile - please try again.'
  } finally {
    creating.value = false
  }
}

function close(): void {
  emit('update:modelValue', false)
}

function reset(): void {
  selectedVersion.value = null
  selectedBuild.value = null
  builds.value = []
  createError.value = ''
  installGameDir.value = ''
  createdProfileName.value = ''
  copied.value = false
}

watch(() => props.modelValue, open => {
  if (open) {
    reset()
    if (!mcVersions.value.length) loadVersions()
  }
})
</script>

<style lang="scss" scoped>
@import '../../styles/motion';

.of-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.of-modal {
  @extend %glass-panel;
  background: rgba(17, 17, 17, 0.9);
  width: 560px;
  max-width: 92vw;
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.of-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.of-title {
  font-size: 16px;
  font-weight: 700;
  color: $text-primary;
}

.of-close {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: $muted;
  cursor: pointer;
  padding: 4px;
  transition: color $transition;
  &:hover { color: #fff; }
}

.of-sub {
  font-size: 12px;
  color: $muted;
  line-height: 1.5;
  margin-top: -6px;

  &--muted { opacity: 0.75; }
}

.of-path-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.of-path {
  flex: 1;
  min-width: 0;
  padding: 8px 10px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: $text-primary;
  font-size: 11.5px;
  white-space: nowrap;
  overflow-x: auto;
}

.of-copy-btn {
  flex-shrink: 0;
  padding: 8px 14px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: 12px;
  color: $text-primary;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: background $transition;
  &:hover { background: rgba(255, 255, 255, 0.16); }
}

.of-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  min-height: 260px;
}

.of-col {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.of-col-label {
  font-size: 10.5px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: $muted;
}

.of-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 260px;
  overflow-y: auto;
  padding-right: 2px;
}

.of-row {
  text-align: left;
  padding: 8px 10px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  color: $text-primary;
  font-size: 12.5px;
  cursor: pointer;
  transition: background $transition, border-color $transition;

  &:hover { background: rgba(255, 255, 255, 0.07); }

  &.active {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.4);
  }
}

.of-preview-tag {
  margin-left: 6px;
  padding: 1px 5px;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: #ffb454;
  background: rgba(255, 180, 84, 0.14);
  border: 1px solid rgba(255, 180, 84, 0.3);
}

.of-empty, .of-error {
  font-size: 12px;
  color: $muted;
  padding: 10px 2px;
}

.of-error { color: #ff8a8a; }

.of-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px 0;
}

.of-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid rgba(255, 255, 255, 0.15);
  border-top-color: #fff;
  border-radius: 50%;
  animation: of-spin 0.8s linear infinite;
}

@keyframes of-spin {
  to { transform: rotate(360deg); }
}

.of-create-error {
  font-size: 12px;
  color: #ff8a8a;
}

.of-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.of-btn {
  padding: 8px 18px;
  border: none;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity $transition;

  &:hover { opacity: 0.85; }
  &:disabled { opacity: 0.4; cursor: not-allowed; }

  &--cancel {
    background: rgba(255, 255, 255, 0.1);
    color: $text-primary;
  }

  &--create {
    background: var(--accent, #{$primary});
    color: #fff;
  }
}

.modal-fade-enter-active .of-modal,
.modal-fade-leave-active .of-modal {
  transition: transform 180ms $ease-out;
}
.modal-fade-enter-from .of-modal,
.modal-fade-leave-to .of-modal {
  transform: scale(0.95) translateY(6px);
}
</style>
