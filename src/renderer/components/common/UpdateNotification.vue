<template>
  <Transition name="update-slide">
    <div v-if="state !== 'idle'" class="update-bar" :class="state">

      <!-- Downloading (auto-started) -->
      <template v-if="state === 'downloading'">
        <div class="update-progress-track">
          <div class="update-progress-fill" :style="{ width: progress + '%' }" />
        </div>
        <span class="update-label">
          Updating to <strong>v{{ version }}</strong> — {{ progress }}%
        </span>
      </template>

      <!-- Countdown before restart -->
      <template v-else-if="state === 'restarting'">
        <div class="update-progress-track full" />
        <span class="update-label">
          <strong>v{{ version }}</strong> installed — restarting in {{ countdown }}s…
        </span>
      </template>

      <!-- Error -->
      <template v-else-if="state === 'error'">
        <span class="update-label">Update failed: {{ errorMsg }}</span>
        <button class="update-dismiss" @click="state = 'idle'">✕</button>
      </template>

    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

type State = 'idle' | 'downloading' | 'restarting' | 'error'

const PENDING_KEY = 'beja-whats-new-pending'

const state    = ref<State>('idle')
const version  = ref('')
const progress = ref(0)
const countdown = ref(3)
const errorMsg = ref('')

let countdownTimer: ReturnType<typeof setInterval>

function startCountdown() {
  state.value = 'restarting'
  countdown.value = 3
  countdownTimer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) {
      clearInterval(countdownTimer)
      window.api.updater.install()
    }
  }, 1000)
}

onMounted(() => {
  window.api.updater.onAvailable((info) => {
    version.value = info.version
    progress.value = 0
    state.value = 'downloading'
    const rawNotes = info.releaseNotes as string | Array<{ note?: string }> | null | undefined
    const notes = typeof rawNotes === 'string'
      ? rawNotes
      : Array.isArray(rawNotes)
        ? rawNotes.map(n => n.note ?? '').join('\n')
        : null
    localStorage.setItem(PENDING_KEY, JSON.stringify({ version: info.version, notes }))
  })

  window.api.updater.onProgress((p) => {
    progress.value = Math.round(p.percent)
    state.value = 'downloading'
  })

  window.api.updater.onDownloaded((info) => {
    version.value = info.version
    startCountdown()
  })

  window.api.updater.onError((msg) => {
    errorMsg.value = msg
    state.value = 'error'
  })
})
</script>

<style lang="scss" scoped>
.update-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 9999;
  display: flex;
  align-items: center;
  gap: $sp-3;
  background: $bg;
  border-top: 1px solid $border;
  padding: 8px 20px;
  height: 36px;
}

.update-progress-track {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: $border;

  &.full::after {
    content: '';
    position: absolute;
    inset: 0;
    background: $text-secondary;
  }
}

.update-progress-fill {
  height: 100%;
  background: $text-primary;
  transition: width 0.4s ease;
}

.update-label {
  font-size: 12px;
  color: $text-secondary;
  flex: 1;
  text-align: center;

  strong { color: $text-primary; font-weight: 600; }
}

.update-dismiss {
  background: none;
  border: none;
  color: $muted;
  cursor: pointer;
  font-size: 12px;
  padding: 2px 6px;
  border-radius: 4px;
  &:hover { color: $text-primary; }
}

.update-slide-enter-active,
.update-slide-leave-active { transition: opacity 0.2s ease, transform 0.2s ease; }
.update-slide-enter-from,
.update-slide-leave-to     { opacity: 0; transform: translateY(100%); }
</style>
