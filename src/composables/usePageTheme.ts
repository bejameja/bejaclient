import { ref } from 'vue'

const DEFAULT = {
  background:     'rgba(7, 9, 8, 0.45)',
  borderBottom:   '1px solid rgba(255,255,255,0.07)',
  backdropFilter: 'blur(6px)',
}

const topbarTheme = ref({ ...DEFAULT })

export function usePageTheme() {
  function setTopbar(style: Partial<typeof DEFAULT>) {
    topbarTheme.value = { ...DEFAULT, ...style }
  }
  function resetTopbar() {
    topbarTheme.value = { ...DEFAULT }
  }
  return { topbarTheme, setTopbar, resetTopbar }
}
