<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="modelValue" class="modal-overlay" @click.self="close">
        <div class="modal">

          <button class="close-btn" @click="close">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>

          <div class="modal-body">
            <div v-if="skinUrl" class="skin-column">
              <HeroSkinViewer
                :skin-url="skinUrl"
                :cape-url="capeUrl"
                :model="skinModel"
                animation="custom-idle"
                :zoom="0.8"
                :initial-rotation-y="2.7"
                class="skin-viewer"
              />
              <span class="drag-hint">Drag to rotate</span>
            </div>
            <div v-else class="skin-placeholder">
              <span>Sign in to preview capes on your skin.</span>
            </div>

            <span class="preview-cape-name">{{ capeName }}</span>
          </div>

        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAccountStore } from '../../store/accountStore'
import HeroSkinViewer from '../skin/HeroSkinViewer.vue'

const props = defineProps<{
  modelValue: boolean
  capeUrl: string | null
  capeName: string
}>()

const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>()

const accountStore = useAccountStore()
const skinUrl   = computed(() => accountStore.selectedAccount?.skinUrl ?? null)
const skinModel = computed(() => accountStore.selectedAccount?.skinModel ?? 'default')

function close() { emit('update:modelValue', false) }
</script>

<style lang="scss" scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.modal {
  position: relative;
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius-lg;
  width: 320px;
  max-width: 92vw;
  overflow: hidden;
}

.close-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  background: $surface-elevated;
  border: 1px solid $border;
  border-radius: $radius;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: $text-secondary;
  transition: color $transition, background $transition;
  z-index: 1;
  &:hover { color: $text-primary; background: $border; }
}

.modal-body {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.skin-column {
  width: 100%;
  background: #0d0d0d;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 16px 8px 10px;
  min-height: 380px;
}

.skin-viewer {
  width: 100%;
  height: 340px;
}

.drag-hint {
  font-size: 10px;
  color: $muted;
  letter-spacing: 0.05em;
  padding-bottom: 2px;
}

.skin-placeholder {
  min-height: 380px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 24px;
  text-align: center;
  font-size: 12px;
  color: $text-secondary;
}

.preview-cape-name {
  width: 100%;
  padding: 12px 16px 16px;
  text-align: center;
  font-size: 13px;
  color: $text-primary;
  border-top: 1px solid $border;
}

</style>
