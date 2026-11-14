<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="modelValue" class="health-overlay" @click.self="close">
        <div class="health-modal">
          <div class="health-header">
            <div class="health-icon-wrap" :class="overallClass">
              <svg v-if="overallStatus === 'pass'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"/><polyline points="16 9 10.5 15 8 12.5"/>
              </svg>
              <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
                <line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
              </svg>
            </div>
            <div>
              <h2 class="health-title">Instance Health Doctor</h2>
              <p class="health-subtitle">{{ statusSummary }}</p>
            </div>
          </div>

          <div class="health-body">
            <div v-if="loading" class="health-loading">
              <div class="spinner" />
              <span>Running checks…</span>
            </div>

            <div v-else v-for="c in checks" :key="c.id" class="health-item" :class="c.status">
              <span class="health-dot" />
              <div class="health-item-text">
                <span class="health-item-label">{{ c.label }}</span>
                <span class="health-item-msg">{{ c.message }}</span>
              </div>
            </div>
          </div>

          <div v-if="repairing" class="health-progress">
            <div class="health-progress-track">
              <div class="health-progress-fill" :style="{ width: repairPct + '%' }" />
            </div>
            <span class="health-progress-label">{{ repairTask }}</span>
          </div>

          <div class="health-footer">
            <button class="btn-cancel" @click="close">Close</button>
            <button class="btn-refresh" :disabled="loading || repairing" @click="run">Re-check</button>
            <button v-if="hasFixable" class="btn-repair" :disabled="repairing" @click="repair">
              {{ repairing ? 'Repairing…' : 'Repair' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { HealthCheck } from '../../types'
import { showToast } from '../../composables/useToasts'

const props = defineProps<{ modelValue: boolean; profileId: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()

const checks    = ref<HealthCheck[]>([])
const loading   = ref(false)
const repairing = ref(false)
const repairPct = ref(0)
const repairTask = ref('')

const overallStatus = computed<'pass' | 'warn' | 'fail'>(() => {
  if (checks.value.some(c => c.status === 'fail')) return 'fail'
  if (checks.value.some(c => c.status === 'warn')) return 'warn'
  return 'pass'
})
const overallClass = computed(() => overallStatus.value)
const hasFixable = computed(() => checks.value.some(c => c.status !== 'pass' && c.fixable))

const statusSummary = computed(() => {
  if (loading.value) return 'Checking your install…'
  const fails = checks.value.filter(c => c.status === 'fail').length
  const warns = checks.value.filter(c => c.status === 'warn').length
  if (fails === 0 && warns === 0) return 'Everything looks healthy'
  const parts = []
  if (fails) parts.push(`${fails} issue${fails > 1 ? 's' : ''}`)
  if (warns) parts.push(`${warns} warning${warns > 1 ? 's' : ''}`)
  return parts.join(', ') + ' found'
})

async function run(): Promise<void> {
  loading.value = true
  try {
    checks.value = await window.api.doctor.run(props.profileId)
  } catch (e) {
    showToast({ title: 'Health check failed', body: String(e), variant: 'error' })
    checks.value = []
  } finally {
    loading.value = false
  }
}

async function repair(): Promise<void> {
  repairing.value = true
  repairPct.value = 0
  repairTask.value = 'Starting repair…'
  try {
    const result = await window.api.doctor.repair(props.profileId)
    if (result.ok) {
      showToast({ title: 'Repair finished', variant: 'success' })
      await run()
    } else {
      showToast({ title: 'Repair failed', body: result.error, variant: 'error' })
    }
  } finally {
    repairing.value = false
  }
}

window.api.doctor.onProgress(({ task, progress, total }) => {
  repairTask.value = task
  repairPct.value = total > 0 ? Math.round((progress / total) * 100) : 0
})

function close(): void {
  emit('update:modelValue', false)
}

watch(() => props.modelValue, open => { if (open) run() })
</script>

<style lang="scss" scoped>
.health-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.72);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  backdrop-filter: blur(4px);
}

.health-modal {
  width: 440px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  background: #16171c;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: $radius;
  overflow: hidden;
}

.health-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 18px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.health-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  flex-shrink: 0;

  &.pass { background: rgba(48, 209, 88, 0.15); color: #30d158; }
  &.warn { background: rgba(232, 160, 48, 0.15); color: #e8a030; }
  &.fail { background: rgba(224, 80, 80, 0.15); color: #e05050; }
}

.health-title {
  font-size: 15px;
  font-weight: 700;
  color: #fff;
  margin: 0;
}

.health-subtitle {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  margin: 2px 0 0;
}

.health-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
}

.health-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 40px 0;
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-top-color: $primary;
  border-radius: 50%;
  animation: hc-spin 0.7s linear infinite;
}

@keyframes hc-spin { to { transform: rotate(360deg); } }

.health-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);

  &:last-child { border-bottom: none; }
}

.health-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-top: 5px;
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.2);

  .health-item.pass & { background: #30d158; }
  .health-item.warn & { background: #e8a030; }
  .health-item.fail & { background: #e05050; }
}

.health-item-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.health-item-label {
  font-size: 12px;
  font-weight: 600;
  color: #fff;
}

.health-item-msg {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.5);
  line-height: 1.4;
}

.health-progress {
  padding: 0 20px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.health-progress-track {
  height: 4px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 2px;
  overflow: hidden;
}

.health-progress-fill {
  height: 100%;
  background: $primary;
  transition: width 200ms ease;
}

.health-progress-label {
  font-size: 10px;
  color: rgba(255, 255, 255, 0.4);
}

.health-footer {
  display: flex;
  gap: 8px;
  padding: 14px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.btn-cancel, .btn-refresh, .btn-repair {
  flex: 1;
  padding: 9px 0;
  font-size: 12px;
  font-weight: 600;
  border: none;
  border-radius: $radius-sm;
  cursor: pointer;
  transition: opacity 100ms;

  &:disabled { opacity: 0.5; cursor: not-allowed; }
}

.btn-cancel { background: rgba(255, 255, 255, 0.08); color: rgba(255, 255, 255, 0.7); }
.btn-refresh { background: rgba(255, 255, 255, 0.08); color: rgba(255, 255, 255, 0.7); }
.btn-repair { background: $primary; color: #fff; }

.modal-fade-enter-active, .modal-fade-leave-active { transition: opacity 150ms ease; }
.modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }
</style>
