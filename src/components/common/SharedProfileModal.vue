<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="pending" class="shared-overlay" @click.self="dismiss">
        <div class="shared-modal">

          <div class="shared-header">
            <div class="shared-icon-wrap">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/>
                <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>
              </svg>
            </div>
            <div>
              <h2 class="shared-title">Shared profile</h2>
              <p class="shared-subtitle">{{ pending.ownerUsername }} sent you a profile</p>
            </div>
          </div>

          <div class="shared-body">
            <div class="shared-profile-name">{{ pending.profileName }}</div>
            <div class="shared-meta">
              <span v-if="pending.loader">{{ pending.loader }}</span>
              <span v-if="pending.loader && pending.version" class="sep">&middot;</span>
              <span v-if="pending.version">{{ pending.version }}</span>
              <span v-if="pending.modCount" class="sep">&middot;</span>
              <span v-if="pending.modCount">{{ pending.modCount }} mods referenced</span>
            </div>
            <p class="shared-note">Installing creates a new local profile with these settings. Mods referenced in the pack still need to be installed separately if you don't already have them.</p>
          </div>

          <div class="shared-footer">
            <button class="btn-cancel" @click="dismiss">Cancel</button>
            <button class="btn-install" :disabled="installing" @click="install">
              {{ installing ? 'Installing…' : 'Install profile' }}
            </button>
          </div>

        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { showToast } from '../../composables/useToasts'
import { useLauncherStore } from '../../store/launcherStore'

interface PendingShare {
  shareId: string
  ownerUsername: string
  profileName: string
  version?: string
  loader?: string
  modCount: number
}

const store = useLauncherStore()
const pending = ref<PendingShare | null>(null)
const installing = ref(false)

function dismiss(): void {
  pending.value = null
}

async function install(): Promise<void> {
  if (!pending.value) return
  installing.value = true
  try {
    const result = await window.api.profiles.importShared(pending.value.shareId)
    if ('error' in result) {
      showToast({ title: 'Could not install profile', body: result.error, variant: 'error' })
      return
    }
    await store.loadProfiles()
    showToast({ title: `Installed "${result.profile.name}"`, body: `Shared by ${result.ownerUsername}`, variant: 'success' })
    pending.value = null
  } finally {
    installing.value = false
  }
}

onMounted(() => {
  window.api.profiles.onSharedLink(async (shareId: string) => {
    const peek = await window.api.profiles.peekShared(shareId)
    if ('error' in peek) {
      showToast({ title: 'Could not open shared profile', body: peek.error, variant: 'error' })
      return
    }
    pending.value = { shareId, ...peek }
  })
})
</script>

<style lang="scss" scoped>
.shared-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.72);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  backdrop-filter: blur(4px);
}

.shared-modal {
  width: 420px;
  max-width: 92vw;
  background: $surface;
  border: 1px solid $border;
  border-radius: 4px;
  overflow: hidden;
  box-shadow: $shadow-inset, $shadow-xl;
}

.shared-header {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px 20px 16px;
  border-bottom: 1px solid $border;
  background: $surface-panel;
}

.shared-icon-wrap {
  width: 40px;
  height: 40px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.07);
  border: 1px solid $border-strong;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent, #{$primary});
  flex-shrink: 0;
}

.shared-title {
  font-size: 14px;
  font-weight: 700;
  color: $text-primary;
  margin: 0 0 2px;
}

.shared-subtitle {
  font-size: 11px;
  color: $text-muted;
  margin: 0;
}

.shared-body {
  padding: 18px 20px;
}

.shared-profile-name {
  font-size: 15px;
  font-weight: 700;
  color: $text-primary;
  margin-bottom: 4px;
}

.shared-meta {
  font-size: 12px;
  color: $text-secondary;
  margin-bottom: 14px;

  .sep { margin: 0 6px; opacity: 0.5; }
}

.shared-note {
  font-size: 11.5px;
  color: $text-muted;
  line-height: 1.6;
  margin: 0;
}

.shared-footer {
  display: flex;
  gap: 8px;
  padding: 14px 16px;
  border-top: 1px solid $border;
  background: $surface-panel;
}

.btn-cancel {
  flex: 1;
  padding: 8px 16px;
  background: $surface-elevated;
  color: $text-secondary;
  border: 1px solid $border;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all $transition;
  &:hover { background: $border; color: $text-primary; }
}

.btn-install {
  flex: 2;
  padding: 8px 16px;
  background: var(--accent, #{$primary});
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: opacity $transition;
  &:hover { opacity: 0.85; }
  &:disabled { opacity: 0.5; cursor: not-allowed; }
}
</style>
