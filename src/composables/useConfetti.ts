import { useReducedMotion } from './useReducedMotion'

interface ConfettiOrigin {
  x: number
  y: number
}

const COLORS = ['#3eb8ff', '#30d158', '#f0f0f0', '#e8a030']

export function burstConfetti(origin: ConfettiOrigin): void {
  const { motionOK } = useReducedMotion()
  if (!motionOK.value) return

  const canvas = document.createElement('canvas')
  canvas.style.position = 'fixed'
  canvas.style.inset = '0'
  canvas.style.pointerEvents = 'none'
  canvas.style.zIndex = '9999'
  canvas.width = window.innerWidth
  canvas.height = window.innerHeight
  document.body.appendChild(canvas)
  const ctx = canvas.getContext('2d')
  if (!ctx) { canvas.remove(); return }

  const particles = Array.from({ length: 50 }, () => ({
    x: origin.x,
    y: origin.y,
    vx: (Math.random() - 0.5) * 9,
    vy: -Math.random() * 9 - 4,
    size: 4 + Math.random() * 4,
    color: COLORS[Math.floor(Math.random() * COLORS.length)],
    rotation: Math.random() * Math.PI * 2,
    vr: (Math.random() - 0.5) * 0.3,
  }))

  const gravity = 0.35
  const drag = 0.985
  const duration = 1100
  let start: number | null = null

  function frame(t: number) {
    if (start === null) start = t
    const elapsed = t - start
    ctx!.clearRect(0, 0, canvas.width, canvas.height)
    for (const p of particles) {
      p.vx *= drag
      p.vy = p.vy * drag + gravity
      p.x += p.vx
      p.y += p.vy
      p.rotation += p.vr
      ctx!.save()
      ctx!.translate(p.x, p.y)
      ctx!.rotate(p.rotation)
      ctx!.globalAlpha = Math.max(0, 1 - elapsed / duration)
      ctx!.fillStyle = p.color
      ctx!.fillRect(-p.size / 2, -p.size / 2, p.size, p.size * 0.6)
      ctx!.restore()
    }
    if (elapsed < duration) {
      requestAnimationFrame(frame)
    } else {
      canvas.remove()
    }
  }

  requestAnimationFrame(frame)
}
