import { ref, watch, type Ref } from 'vue'
import { useReducedMotion } from './useReducedMotion'

export function useCountUp(source: Ref<number>, duration = 500) {
  const { motionOK } = useReducedMotion()
  const display = ref(source.value)
  let raf: number | null = null

  watch(source, (newVal, oldVal) => {
    if (raf !== null) cancelAnimationFrame(raf)

    if (!motionOK.value || oldVal === undefined) {
      display.value = newVal
      return
    }

    const from = oldVal
    const start = performance.now()

    function tick(now: number) {
      const t = Math.min(1, (now - start) / duration)
      const eased = 1 - Math.pow(1 - t, 3)
      display.value = Math.round(from + (newVal - from) * eased)
      raf = t < 1 ? requestAnimationFrame(tick) : null
    }

    raf = requestAnimationFrame(tick)
  })

  return display
}
