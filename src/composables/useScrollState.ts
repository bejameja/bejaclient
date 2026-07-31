import { ref, readonly } from 'vue'

const scrollPercent = ref(0)
const hasScroll = ref(false)

export function useScrollState() {
  function updateScroll(el: HTMLElement) {
    const max = el.scrollHeight - el.clientHeight
    hasScroll.value = max > 0
    scrollPercent.value = max > 0 ? el.scrollTop / max : 0
  }

  return { scrollPercent: readonly(scrollPercent), hasScroll: readonly(hasScroll), updateScroll }
}
