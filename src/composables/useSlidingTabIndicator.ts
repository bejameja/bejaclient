import { ref, watch, onMounted, onUnmounted, nextTick, type Ref } from 'vue'

/**
 * Tracks the DOM rect of whichever tab button matches `activeKey` and exposes
 * a left/width style so a shared `.tab-indicator` element can slide under it
 * instead of each button drawing its own static active-state underline.
 */
export function useSlidingTabIndicator<T extends string>(activeKey: Ref<T>) {
  const tabBtnRefs = new Map<string, HTMLElement>()
  const indicatorStyle = ref({ left: '0px', width: '0px' })

  function setTabBtnRef(key: string, el: HTMLElement | null) {
    if (el) tabBtnRefs.set(key, el)
    else tabBtnRefs.delete(key)
  }

  function updateIndicator() {
    const btn = tabBtnRefs.get(activeKey.value)
    if (!btn) return
    indicatorStyle.value = { left: `${btn.offsetLeft}px`, width: `${btn.offsetWidth}px` }
  }

  watch(activeKey, () => nextTick(updateIndicator))
  onMounted(() => {
    nextTick(updateIndicator)
    window.addEventListener('resize', updateIndicator)
  })
  onUnmounted(() => window.removeEventListener('resize', updateIndicator))

  return { setTabBtnRef, indicatorStyle, updateIndicator }
}
