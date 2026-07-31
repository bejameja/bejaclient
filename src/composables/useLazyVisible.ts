import { ref, onMounted, onUnmounted, type Ref } from 'vue'

/**
 * Tracks whether `target` is on/near screen. Used to gate expensive per-item
 * work (e.g. a live WebGL context per grid card) so a long list doesn't
 * create it all at once — see the callers for why that matters.
 */
export function useLazyVisible(target: Ref<HTMLElement | null>, rootMargin = '200px') {
  const isVisible = ref(false)
  let observer: IntersectionObserver | null = null

  onMounted(() => {
    if (!target.value) return
    observer = new IntersectionObserver(
      entries => { isVisible.value = entries[0]?.isIntersecting ?? false },
      { rootMargin },
    )
    observer.observe(target.value)
  })

  onUnmounted(() => {
    observer?.disconnect()
    observer = null
  })

  return isVisible
}
