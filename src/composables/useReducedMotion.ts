import { computed, ref, watchEffect } from 'vue'
import { useSettingsStore } from '../store/settingsStore'

const osReduceMotion = ref(false)
let initialized = false

export function useReducedMotion() {
  const settingsStore = useSettingsStore()

  if (!initialized) {
    initialized = true
    const mq = window.matchMedia('(prefers-reduced-motion: reduce)')
    osReduceMotion.value = mq.matches
    mq.addEventListener('change', (e) => { osReduceMotion.value = e.matches })

    watchEffect(() => {
      document.documentElement.classList.toggle('reduce-motion', effectiveReduce())
    })
  }

  function effectiveReduce(): boolean {
    const setting = settingsStore.settings.appearance.reduceMotion
    if (setting === 'on') return true
    if (setting === 'off') return false
    return osReduceMotion.value // 'system'
  }

  const motionOK = computed(() => !effectiveReduce())

  return { motionOK, osReduceMotion: computed(() => osReduceMotion.value) }
}
