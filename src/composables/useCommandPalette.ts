import { ref } from 'vue'

const open = ref(false)

export function openPalette(): void {
  open.value = true
}

export function closePalette(): void {
  open.value = false
}

export function togglePalette(): void {
  open.value = !open.value
}

export function useCommandPalette() {
  return { open, openPalette, closePalette, togglePalette }
}
