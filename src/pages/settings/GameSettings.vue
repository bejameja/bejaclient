<template>
  <div class="settings-section">
    <h2 class="section-heading">Game Settings</h2>
    <p class="section-desc">Configure Minecraft directories, Java, and memory allocation.</p>

    <div class="settings-groups">
      <div class="settings-group">
        <h3 class="group-title">Directories</h3>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Game Directory</span>
            <span class="setting-desc">Where Minecraft files are stored</span>
          </div>
          <div class="setting-control dir-control">
            <span class="dir-path">{{ truncatePath(settings.game.defaultGameDir) }}</span>
            <Button variant="secondary" size="sm" @click="chooseDir">Browse</Button>
          </div>
        </div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Java Executable</span>
            <span class="setting-desc">{{ detectedJavas.length ? `${detectedJavas.length} version(s) detected on this system` : 'Leave empty to auto-detect' }}</span>
          </div>
          <div class="setting-control dir-control">
            <Select
              v-if="detectedJavas.length"
              :model-value="settings.game.defaultJavaPath"
              :options="javaOptions"
              placeholder="Auto-detect"
              class="java-select"
              @update:model-value="onSelectJava"
            />
            <span v-else class="dir-path">{{ settings.game.defaultJavaPath || 'Auto-detect' }}</span>
            <Button variant="secondary" size="sm" @click="chooseJava">Browse</Button>
          </div>
        </div>
      </div>

      <div class="settings-group">
        <h3 class="group-title">Managed Java runtimes</h3>
        <p class="group-subtext">Downloaded from Mojang and picked automatically per Minecraft version. Remove ones you no longer need; they are re-downloaded on demand.</p>
        <p v-if="runtimesError" class="rt-error">{{ runtimesError }}</p>
        <div v-for="rt in runtimes" :key="rt.component" class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Java {{ rt.majorVersion }}</span>
            <span class="setting-desc">{{ rt.label }} &middot; {{ rt.installed ? `installed, ${formatSize(rt.sizeBytes)}` : 'not installed' }}</span>
          </div>
          <div class="setting-control">
            <Button v-if="!rt.installed" variant="secondary" size="sm" :disabled="busyRuntime !== ''" @click="installRuntime(rt)">
              {{ busyRuntime === rt.component ? 'Downloading…' : 'Install' }}
            </Button>
            <Button v-else variant="secondary" size="sm" :disabled="busyRuntime !== ''" @click="removeRuntime(rt)">
              {{ busyRuntime === rt.component ? 'Removing…' : 'Remove' }}
            </Button>
          </div>
        </div>
      </div>

      <div class="settings-group">
        <div class="group-title-row">
          <h3 class="group-title">Memory</h3>
          <button class="autotune-btn" :disabled="autotuning" @click="autoTune">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" :class="{ spinning: autotuning }">
              <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/>
            </svg>
            Auto-tune
          </button>
        </div>
        <p v-if="hardwareLabel" class="group-subtext">{{ hardwareLabel }}</p>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Minimum RAM</span>
            <span class="setting-desc">Initial memory allocation</span>
          </div>
          <div class="setting-control">
            <span class="ram-value">{{ formatRam(settings.game.minRam) }}</span>
            <Slider
              v-model="settings.game.minRam"
              :min="256"
              :max="2048"
              :step="256"
              :format="formatRam"
              class="ram-slider"
              @change="save"
            />
          </div>
        </div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Maximum RAM</span>
            <span class="setting-desc">Maximum memory allocation</span>
          </div>
          <div class="setting-control">
            <span class="ram-value">{{ formatRam(settings.game.maxRam) }}</span>
            <Slider
              v-model="settings.game.maxRam"
              :min="512"
              :max="16384"
              :step="256"
              :format="formatRam"
              class="ram-slider"
              @change="save"
            />
          </div>
        </div>
      </div>

      <div class="settings-group">
        <h3 class="group-title">Resolution</h3>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Window Size</span>
            <span class="setting-desc">Default game window resolution</span>
          </div>
          <div class="setting-control res-control">
            <Input
              v-model="settings.game.resolution.width"
              type="number"
              style="width: 80px"
              @blur="save"
            />
            <span class="res-sep">×</span>
            <Input
              v-model="settings.game.resolution.height"
              type="number"
              style="width: 80px"
              @blur="save"
            />
          </div>
        </div>
      </div>

      <div class="settings-group">
        <h3 class="group-title">JVM Arguments</h3>
        <div class="setting-row vertical">
          <span class="setting-label">Additional JVM flags</span>
          <Input
            v-model="settings.game.jvmArgs"
            placeholder="-XX:+UseG1GC -XX:MaxGCPauseMillis=200"
            @blur="save"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import Button from '../../components/common/Button.vue'
import Input from '../../components/common/Input.vue'
import Select from '../../components/common/Select.vue'
import Slider from '../../components/common/Slider.vue'
import { useSettingsStore } from '../../store/settingsStore'
import { showToast } from '../../composables/useToasts'
import type { JavaInfo, ManagedJavaRuntime } from '../../types'

const settingsStore = useSettingsStore()
const settings = computed(() => settingsStore.settings)

// ── Detected Java installs ───────────────────────────────────────────────────
const detectedJavas = ref<JavaInfo[]>([])

const javaOptions = computed(() => {
  const opts = [
    { label: 'Auto-detect', value: '' },
    ...detectedJavas.value.map(j => ({
      label: `Java ${j.majorVersion}${j.arch ? ` (${j.arch})` : ''} - ${j.path}`,
      value: j.path,
    })),
  ]
  const current = settings.value.game.defaultJavaPath
  if (current && !opts.some(o => o.value === current)) {
    opts.push({ label: `Custom - ${current}`, value: current })
  }
  return opts
})

async function onSelectJava(path: string | number): Promise<void> {
  settings.value.game.defaultJavaPath = String(path)
  await save()
}

// ── Managed Java runtimes ────────────────────────────────────────────────────
const runtimes = ref<ManagedJavaRuntime[]>([])
const runtimesError = ref('')
const busyRuntime = ref('')

async function refreshRuntimes(): Promise<void> {
  try {
    runtimes.value = await window.api.system.listJavaRuntimes()
    runtimesError.value = ''
  } catch (e) {
    runtimesError.value = `Could not read runtimes: ${e instanceof Error ? e.message : String(e)}`
  }
}

function formatSize(bytes: number): string {
  if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(1)} GB`
  return `${Math.max(1, Math.round(bytes / 1024 ** 2))} MB`
}

async function installRuntime(rt: ManagedJavaRuntime): Promise<void> {
  busyRuntime.value = rt.component
  try {
    await window.api.system.installJavaRuntime(rt.component)
    showToast({ title: `Java ${rt.majorVersion} installed`, body: rt.label, variant: 'success', duration: 2500 })
  } catch (e) {
    showToast({ title: `Java ${rt.majorVersion} install failed`, body: String(e), variant: 'error', duration: 5000 })
  } finally {
    busyRuntime.value = ''
    await refreshRuntimes()
  }
}

async function removeRuntime(rt: ManagedJavaRuntime): Promise<void> {
  busyRuntime.value = rt.component
  try {
    await window.api.system.removeJavaRuntime(rt.component)
    showToast({ title: `Java ${rt.majorVersion} removed`, body: rt.label, variant: 'info', duration: 2200 })
  } catch (e) {
    showToast({ title: `Could not remove Java ${rt.majorVersion}`, body: String(e), variant: 'error', duration: 5000 })
  } finally {
    busyRuntime.value = ''
    await refreshRuntimes()
  }
}

onMounted(async () => {
  detectedJavas.value = await window.api.system.getJavaVersions()
  await refreshRuntimes()
})

// ── Auto-tune ─────────────────────────────────────────────────────────────────
const autotuning    = ref(false)
const hardwareLabel = ref('')

// Community-standard "Aikar's flags" G1GC tuning set - widely published/public-domain best
// practice for modded Minecraft, not tied to any specific heap size.
const AUTOTUNE_JVM_ARGS =
  '-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions ' +
  '-XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 ' +
  '-XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 ' +
  '-XX:InitiatingHeapOccupancyPercent=15 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 ' +
  '-XX:SurvivorRatio=32 -XX:MaxTenuringThreshold=1'

function roundToStep(value: number, step: number): number {
  return Math.round(value / step) * step
}

async function autoTune(): Promise<void> {
  if (autotuning.value) return
  autotuning.value = true
  try {
    const { totalRamMb, cpuCores } = await window.api.system.hardwareInfo()
    // Leave at least a quarter of system RAM (min 2 GB) for the OS and everything else running.
    const reserveMb = Math.max(2048, Math.round(totalRamMb * 0.25))
    const recommendedMax = Math.min(16384, Math.max(1024, roundToStep(totalRamMb - reserveMb, 256)))
    const recommendedMin = Math.min(recommendedMax, Math.max(256, roundToStep(recommendedMax / 4, 256)))

    settings.value.game.maxRam = recommendedMax
    settings.value.game.minRam = recommendedMin
    settings.value.game.jvmArgs = AUTOTUNE_JVM_ARGS
    await save()

    hardwareLabel.value = `Detected ${formatRam(totalRamMb)} RAM, ${cpuCores} CPU threads - allocated ${formatRam(recommendedMax)}`
    showToast({ title: 'Memory & JVM flags auto-tuned', body: `Max RAM set to ${formatRam(recommendedMax)}`, variant: 'success' })
  } catch (e) {
    showToast({ title: 'Could not detect hardware', body: String(e), variant: 'error' })
  } finally {
    autotuning.value = false
  }
}

async function save() {
  await settingsStore.save()
}

async function chooseDir() {
  await settingsStore.chooseGameDir()
}

async function chooseJava() {
  await settingsStore.chooseJava()
}

function formatRam(mb: number) {
  return mb >= 1024 ? `${(mb / 1024).toFixed(mb % 1024 === 0 ? 0 : 1)} GB` : `${mb} MB`
}

function truncatePath(path: string) {
  if (path.length <= 42) return path
  const parts = path.split(/[/\\]/)
  return '...' + [parts[parts.length - 2], parts[parts.length - 1]].join('/')
}
</script>

<style lang="scss" scoped>
.settings-section {
  display: flex;
  flex-direction: column;
  gap: $sp-6;
  max-width: 640px;
}

.section-heading {
  font-size: 18px;
  font-weight: 800;
  color: $text-primary;
}

.section-desc {
  font-size: 13px;
  color: $muted;
  margin-top: -$sp-4;
}

.settings-groups {
  display: flex;
  flex-direction: column;
  gap: $sp-6;
}

.rt-error {
  font-size: 12px;
  color: #ff6b6b;
}

.group-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.group-subtext {
  font-size: 11px;
  color: $muted;
  margin-top: -4px;
}

.autotune-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.03em;
  color: $primary;
  background: rgba(49, 89, 163, 0.12);
  border: 1px solid rgba(49, 89, 163, 0.3);
  border-radius: $radius-sm;
  padding: 5px 10px;
  cursor: pointer;
  transition: background $transition, opacity $transition;

  &:hover:not(:disabled) { background: rgba(49, 89, 163, 0.2); }
  &:disabled { opacity: 0.6; cursor: wait; }

  svg.spinning { animation: autotune-spin 0.8s linear infinite; }
}

@keyframes autotune-spin { to { transform: rotate(360deg); } }

.settings-group {
  display: flex;
  flex-direction: column;
  gap: $sp-1;
}

.group-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: $muted;
  padding-bottom: $sp-2;
  border-bottom: 1px solid $border;
  margin-bottom: $sp-2;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $sp-4;
  padding: $sp-3 0;
  border-bottom: 1px solid $border;

  &:last-child { border-bottom: none; }
  &.vertical { flex-direction: column; align-items: flex-start; gap: $sp-2; }
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.setting-label {
  font-size: 13px;
  font-weight: 600;
  color: $text-primary;
}

.setting-desc {
  font-size: 11px;
  color: $muted;
}

.setting-control {
  display: flex;
  align-items: center;
  gap: $sp-2;
  flex-shrink: 0;

  &.dir-control {
    max-width: 280px;
    min-width: 0;
  }

  &.res-control {
    align-items: center;
  }
}

.dir-path {
  font-size: 11px;
  color: $text-secondary;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}

.java-select {
  flex: 1;
  min-width: 0;
}

.ram-value {
  font-size: 12px;
  font-weight: 600;
  color: $text-primary;
  min-width: 52px;
  text-align: right;
}

.ram-slider {
  width: 120px;
  accent-color: $primary;
  cursor: pointer;
}

.res-sep {
  font-size: 14px;
  color: $muted;
}
</style>
