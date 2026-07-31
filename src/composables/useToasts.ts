import { reactive } from 'vue'

export type ToastVariant = 'info' | 'success' | 'warning' | 'error'

export interface ToastAction {
  label: string
  onClick: () => void
}

export interface ToastOptions {
  title: string
  body?: string
  variant?: ToastVariant
  duration?: number
  action?: ToastAction
}

export interface Toast extends ToastOptions {
  id: string
  variant: ToastVariant
  duration: number
  timerId: ReturnType<typeof setTimeout>
}

const MAX_VISIBLE = 4
const DEFAULT_DURATION = 5000

const toasts = reactive<Toast[]>([])

export function showToast(opts: ToastOptions): string {
  const id = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
  const duration = opts.duration ?? DEFAULT_DURATION

  if (toasts.length >= MAX_VISIBLE) {
    const oldest = toasts[0]
    clearTimeout(oldest.timerId)
    toasts.shift()
  }

  const timerId = setTimeout(() => dismissToast(id), duration)
  toasts.push({
    id,
    title: opts.title,
    body: opts.body,
    variant: opts.variant ?? 'info',
    duration,
    action: opts.action,
    timerId,
  })
  return id
}

export function dismissToast(id: string): void {
  const idx = toasts.findIndex(t => t.id === id)
  if (idx === -1) return
  clearTimeout(toasts[idx].timerId)
  toasts.splice(idx, 1)
}

export function useToasts() {
  return { toasts, showToast, dismissToast }
}
