<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="modelValue" class="muc-overlay" @click.self="close">
        <div class="muc-modal">
          <div class="muc-header">
            <div>
              <h2 class="muc-title">Mod Update Center</h2>
              <p class="muc-subtitle">{{ summary }}</p>
            </div>
            <label v-if="updates.length" class="muc-select-all">
              <input type="checkbox" :checked="allSelected" @change="toggleAll" />
              Select all
            </label>
          </div>

          <div class="muc-body">
            <div v-if="loading" class="muc-loading">
              <div class="spinner" />
              <span>Checking every mod across every profile…</span>
            </div>

            <p v-else-if="!updates.length" class="muc-empty">Everything is up to date.</p>

            <div v-else v-for="u in updates" :key="`${u.profileId}-${u.modId}`" class="muc-row">
              <input type="checkbox" v-model="selected" :value="rowKey(u)" />
              <div class="muc-row-info">
                <span class="muc-mod-name">{{ u.modName }}</span>
                <span class="muc-mod-sub">{{ u.profileName }} · updates to {{ u.update.versionNumber }}</span>
              </div>
              <span v-if="applyResults[rowKey(u)] === 'done'" class="muc-row-status muc-row-status--done">Updated</span>
              <span v-else-if="applyResults[rowKey(u)] === 'failed'" class="muc-row-status muc-row-status--failed">Failed</span>
            </div>
          </div>

          <div class="muc-footer">
            <button class="btn-cancel" @click="close">Close</button>
            <button class="btn-refresh" :disabled="loading || applying" @click="load">Re-check</button>
            <button class="btn-update" :disabled="!selected.length || applying" @click="applySelected">
              {{ applying ? `Updating ${appliedCount}/${selected.length}…` : `Update Selected (${selected.length})` }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { CrossProfileUpdate } from '../../types'
import { showToast } from '../../composables/useToasts'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()

const updates  = ref<CrossProfileUpdate[]>([])
const loading  = ref(false)
const selected = ref<string[]>([])
const applying = ref(false)
const appliedCount = ref(0)
const applyResults = ref<Record<string, 'done' | 'failed'>>({})

function rowKey(u: CrossProfileUpdate): string {
  return `${u.profileId}-${u.modId}`
}

const allSelected = computed(() => updates.value.length > 0 && selected.value.length === updates.value.length)

function toggleAll(): void {
  selected.value = allSelected.value ? [] : updates.value.map(rowKey)
}

const summary = computed(() => {
  if (loading.value) return 'Scanning…'
  if (!updates.value.length) return ''
  return `${updates.value.length} update${updates.value.length > 1 ? 's' : ''} available across your profiles`
})

async function load(): Promise<void> {
  loading.value = true
  applyResults.value = {}
  try {
    updates.value = await window.api.mods.checkAllProfilesUpdates()
    selected.value = updates.value.map(rowKey)
  } finally {
    loading.value = false
  }
}

async function applySelected(): Promise<void> {
  if (applying.value) return
  applying.value = true
  appliedCount.value = 0
  let failCount = 0
  try {
    for (const u of updates.value) {
      const key = rowKey(u)
      if (!selected.value.includes(key)) continue
      try {
        await window.api.modrinth.swapMod(u.profileId, u.modId, u.update.projectId, u.update.versionId)
        applyResults.value = { ...applyResults.value, [key]: 'done' }
      } catch {
        applyResults.value = { ...applyResults.value, [key]: 'failed' }
        failCount++
      }
      appliedCount.value++
    }
    showToast({
      title: failCount ? `Updated with ${failCount} failure(s)` : 'All selected mods updated',
      variant: failCount ? 'error' : 'success',
    })
    // Leave successfully-updated rows visible with their "Updated" badge rather than
    // re-scanning immediately - re-checking wipes the just-applied status the user wants to see.
  } finally {
    applying.value = false
  }
}

function close(): void {
  emit('update:modelValue', false)
}

watch(() => props.modelValue, open => { if (open) load() })
</script>

<style lang="scss" scoped>
.muc-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.72);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  backdrop-filter: blur(4px);
}

.muc-modal {
  width: 520px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  background: #16171c;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: $radius;
  overflow: hidden;
}

.muc-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 18px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.muc-title {
  font-size: 15px;
  font-weight: 700;
  color: #fff;
  margin: 0;
}

.muc-subtitle {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  margin: 2px 0 0;
}

.muc-select-all {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  white-space: nowrap;
}

.muc-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
}

.muc-loading {
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
  animation: muc-spin 0.7s linear infinite;
}

@keyframes muc-spin { to { transform: rotate(360deg); } }

.muc-empty {
  text-align: center;
  padding: 40px 0;
  color: rgba(255, 255, 255, 0.4);
  font-size: 12px;
}

.muc-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);

  &:last-child { border-bottom: none; }
}

.muc-row-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.muc-mod-name {
  font-size: 12px;
  font-weight: 600;
  color: #fff;
}

.muc-mod-sub {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.45);
}

.muc-row-status {
  font-size: 10px;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: $radius-sm;
  flex-shrink: 0;

  &--done { color: #30d158; background: rgba(48, 209, 88, 0.14); }
  &--failed { color: #e05050; background: rgba(224, 80, 80, 0.14); }
}

.muc-footer {
  display: flex;
  gap: 8px;
  padding: 14px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.btn-cancel, .btn-refresh, .btn-update {
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

.btn-cancel, .btn-refresh { background: rgba(255, 255, 255, 0.08); color: rgba(255, 255, 255, 0.7); }
.btn-update { background: $primary; color: #fff; flex: 1.4; }

.modal-fade-enter-active, .modal-fade-leave-active { transition: opacity 150ms ease; }
.modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }
</style>
