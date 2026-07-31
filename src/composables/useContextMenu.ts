import { reactive } from 'vue'
import type { IconName } from '../components/common/icons'

export interface ContextMenuItem {
  label: string
  icon?: IconName
  danger?: boolean
  disabled?: boolean
  separator?: boolean
  onClick?: () => void
}

interface ContextMenuState {
  open: boolean
  x: number
  y: number
  items: ContextMenuItem[]
}

const state = reactive<ContextMenuState>({ open: false, x: 0, y: 0, items: [] })

export function openContextMenu(event: MouseEvent, items: ContextMenuItem[]): void {
  event.preventDefault()
  state.items = items
  state.x = event.clientX
  state.y = event.clientY
  state.open = true
}

export function closeContextMenu(): void {
  state.open = false
}

export function useContextMenu() {
  return { state, openContextMenu, closeContextMenu }
}
