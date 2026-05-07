<template>
  <Transition name="modal-fade">
    <div v-if="visible" class="modal-overlay" @click.self="dismiss">
      <div class="modal">

        <div class="modal-header">
          <div class="modal-title-group">
            <span class="modal-badge">What's New</span>
            <h2 class="modal-title">BejaClient v{{ pendingVersion }}</h2>
          </div>
          <button class="close-btn" @click="dismiss" title="Close">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        <div class="modal-body">
          <div v-if="notes" class="release-notes" v-html="notes" />
          <div v-else class="release-notes no-notes">
            <p>A new version of BejaClient has been installed.</p>
            <p>Check the GitHub releases page for details.</p>
          </div>
        </div>

        <div class="modal-footer">
          <label class="never-again">
            <input type="checkbox" v-model="neverAgain" />
            <span>Don't show this again for v{{ pendingVersion }}</span>
          </label>
          <button class="got-it-btn" @click="dismiss">Got it</button>
        </div>

      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

const PENDING_KEY = 'beja-whats-new-pending'
const seenKey = (v: string) => `beja-whats-new-seen-${v}`

const visible        = ref(false)
const pendingVersion = ref('')
const notes          = ref<string | null>(null)
const neverAgain     = ref(false)

onMounted(async () => {
  try {
    const raw = localStorage.getItem(PENDING_KEY)
    if (!raw) return
    const pending = JSON.parse(raw) as { version: string; notes: string | null }

    const currentVersion = await window.api.system.getVersion()
    if (pending.version !== currentVersion) return
    if (localStorage.getItem(seenKey(pending.version))) return

    pendingVersion.value = pending.version
    notes.value = pending.notes
    visible.value = true
  } catch {}
})

function dismiss() {
  if (neverAgain.value) {
    localStorage.setItem(seenKey(pendingVersion.value), '1')
    localStorage.removeItem(PENDING_KEY)
  }
  visible.value = false
}
</script>

<style lang="scss" scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background: rgba(0, 0, 0, 0.70);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.modal {
  width: 100%;
  max-width: 560px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 22px 22px 16px;
  border-bottom: 1px solid $border;
  flex-shrink: 0;
}

.modal-title-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.modal-badge {
  display: inline-block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: $text-secondary;
  background: $surface-elevated;
  border: 1px solid $border;
  border-radius: 4px;
  padding: 2px 7px;
  width: fit-content;
}

.modal-title {
  font-size: 18px;
  font-weight: 700;
  color: $text-primary;
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  color: $muted;
  cursor: pointer;
  padding: 4px;
  border-radius: $radius;
  line-height: 0;
  transition: color $transition, background $transition;
  &:hover { color: $text-primary; background: $surface-elevated; }
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px 22px;

  &::-webkit-scrollbar { width: 5px; }
  &::-webkit-scrollbar-track { background: transparent; }
  &::-webkit-scrollbar-thumb { background: $border-strong; border-radius: 4px; }
}

.release-notes {
  font-size: 13px;
  line-height: 1.7;
  color: $text-secondary;

  :deep(h1), :deep(h2), :deep(h3) {
    color: $text-primary;
    font-weight: 600;
    margin: 16px 0 8px;
    &:first-child { margin-top: 0; }
  }
  :deep(h1) { font-size: 15px; }
  :deep(h2) { font-size: 14px; }
  :deep(h3) { font-size: 13px; }

  :deep(ul), :deep(ol) {
    padding-left: 20px;
    margin: 8px 0;
  }

  :deep(li) { margin: 4px 0; }

  :deep(a) {
    color: $text-primary;
    text-decoration: none;
    &:hover { text-decoration: underline; }
  }

  :deep(code) {
    background: $surface-elevated;
    border: 1px solid $border;
    border-radius: 3px;
    padding: 1px 5px;
    font-size: 12px;
  }

  :deep(p) { margin: 8px 0; }

  &.no-notes {
    display: flex;
    flex-direction: column;
    gap: 6px;
    color: $muted;
    font-style: italic;
  }
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 22px;
  border-top: 1px solid $border;
  flex-shrink: 0;
}

.never-again {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: $muted;
  cursor: pointer;
  user-select: none;

  input[type='checkbox'] {
    width: 14px;
    height: 14px;
    cursor: pointer;
  }

  &:hover { color: $text-secondary; }
}

.got-it-btn {
  padding: 7px 20px;
  background: $text-primary;
  color: $bg;
  border: none;
  border-radius: $radius;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background $transition;
  &:hover { background: $text-secondary; }
}

.modal-fade-enter-active, .modal-fade-leave-active { transition: opacity 0.25s ease; }
.modal-fade-enter-active .modal, .modal-fade-leave-active .modal { transition: transform 0.25s ease; }
.modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }
.modal-fade-enter-from .modal, .modal-fade-leave-to .modal { transform: scale(0.96) translateY(8px); }
</style>
