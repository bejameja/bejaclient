import { ref } from 'vue'
import { router } from '../router'
import { togglePalette } from './useCommandPalette'
import { useSettingsStore } from '../store/settingsStore'

// Controller support: d-pad / left stick moves a focus ring between whatever is clickable on screen,
// A clicks, B goes back (closes the top overlay first), LB/RB switch tabs, right stick scrolls.
// Plain Gamepad API, nothing per page - the targets are found from the DOM each time you press a direction.

/** True while the controller was the last thing used. Drives the focus ring and the hint bar. */
export const padActive = ref(false)
/** Which pad is plugged in, for the hint bar. */
export const padConnected = ref(false)

const NATIVE = 'button, a[href], input:not([type="hidden"]), select, textarea, summary, [role="button"], [role="tab"], [role="menuitem"], [tabindex]:not([tabindex="-1"]), [data-pad]'
const GENERIC = 'div, li, span, img, article, label, section, td, tr'
const FOCUS_CLASS = 'pad-focus'

const DEADZONE = 0.5
const REPEAT_FIRST_MS = 380
const REPEAT_MS = 120
const SCROLL_SPEED = 16

type Dir = 'up' | 'down' | 'left' | 'right'

let current: HTMLElement | null = null
let started = false
let raf = 0

// per-button state so a press fires once, and a held direction repeats
const down: Record<string, boolean> = {}
const nextRepeat: Record<string, number> = {}

// ── Haptics ──────────────────────────────────────────────────────────────────
// Short, different pulses so you can feel what happened without looking: a tick when the ring moves,
// a press when something is selected, a soft one for back, a dull thud when there is nothing further that way.
export type Rumble = 'move' | 'confirm' | 'back' | 'bump' | 'tab'

// [weak motor (fine buzz), strong motor (deep), duration in ms] at full strength. The numbers are the shape of the pulse;
// Settings > Controller > Strength scales them (see rumble()).
const PATTERNS: Record<Rumble, [number, number, number]> = {
  move:    [1.0, 0.0, 22],
  confirm: [0.8, 0.8, 60],
  back:    [0.6, 0.3, 40],
  bump:    [0.0, 1.0, 70],
  tab:     [0.7, 0.4, 45],
}

let lastRumble = 0

/**
 * Rumbles the connected controller. Strength 10 to 100 maps to 0.235 to 1.0 of the pattern: the motors in most
 * controllers do not move at all below about 0.2, so a plain linear scale made the lower half of the slider feel like "off".
 * The pulse also gets a bit longer the stronger it is, so the steps are felt as well as measured.
 */
export function rumble(kind: Rumble): void {
  const cfg = useSettingsStore().settings.launcher
  if (!cfg.controllerHaptics) return
  const now = performance.now()
  if (kind === 'move' && now - lastRumble < 30) return // holding a direction should not blur into one long buzz
  const pad = (navigator.getGamepads?.() ?? []).find(p => !!p && p.connected && !!(p as any).vibrationActuator)
  const actuator = pad && (pad as any).vibrationActuator
  if (!actuator?.playEffect) return
  const [weak, strong, ms] = PATTERNS[kind]
  const k = Math.max(0, Math.min(100, cfg.hapticsStrength)) / 100
  const intensity = 0.15 + 0.85 * k
  lastRumble = now
  Promise.resolve(actuator.playEffect('dual-rumble', {
    startDelay: 0,
    duration: Math.round(ms * (0.7 + 0.6 * k)),
    weakMagnitude: weak * intensity,
    strongMagnitude: strong * intensity,
  })).catch(() => { /* the controller does not do rumble: nothing to do */ })
}

function isVisible(el: HTMLElement): boolean {
  const r = el.getBoundingClientRect()
  return r.width > 2 && r.height > 2 && r.bottom > 0 && r.right > 0 && r.top < innerHeight && r.left < innerWidth
}

/** Is any part of it really on top (not under a modal, drawer, hidden page or clipped away)? */
function isReachable(el: HTMLElement): boolean {
  const r = el.getBoundingClientRect()
  const l = Math.max(r.left, 0), t = Math.max(r.top, 0)
  const rt = Math.min(r.right, innerWidth - 1), b = Math.min(r.bottom, innerHeight - 1)
  if (rt - l < 2 || b - t < 2) return false
  const w = rt - l, h = b - t
  const pts: [number, number][] = [
    [l + w / 2, t + h / 2],
    [l + w * 0.2, t + h * 0.5],
    [l + w * 0.8, t + h * 0.5],
    [l + w * 0.5, t + h * 0.2],
    [l + w * 0.5, t + h * 0.8],
  ]
  return pts.some(([x, y]) => {
    const hit = document.elementFromPoint(x, y)
    return !!hit && el.contains(hit)
  })
}

function isDisabled(el: HTMLElement): boolean {
  return (el as HTMLButtonElement).disabled === true || el.getAttribute('aria-disabled') === 'true'
}

function hasBox(el: HTMLElement): boolean {
  const r = el.getBoundingClientRect()
  return r.width > 2 && r.height > 2
}

/**
 * Everything the controller can land on. `onScreen` (default) keeps what is really visible and not covered;
 * with false it lists everything rendered inside `root`, including rows scrolled out of view.
 */
function collectTargets(root: ParentNode = document, onScreen = true): HTMLElement[] {
  const shown = onScreen ? isVisible : hasBox
  const found = new Set<HTMLElement>()
  root.querySelectorAll<HTMLElement>(NATIVE).forEach(el => found.add(el))

  // Vue pages are full of clickable divs/cards without a role: take the outermost ones that show a pointer cursor.
  // While the controller is in use the mouse cursor is hidden (cursor: none on everything), which would hide this clue
  // too: show it again just for the moment of reading it. Nothing is painted in between.
  const html = document.documentElement
  const hidden = html.classList.contains('pad-nocursor')
  if (hidden) html.classList.remove('pad-nocursor')
  try {
    root.querySelectorAll<HTMLElement>(GENERIC).forEach(el => {
      if (found.has(el) || !shown(el)) return
      if (getComputedStyle(el).cursor !== 'pointer') return
      const parent = el.parentElement
      // cursor is inherited, so a pointer parent means this is just its content
      if (parent && getComputedStyle(parent).cursor === 'pointer') return
      found.add(el)
    })
  } finally {
    if (hidden) html.classList.add('pad-nocursor')
  }

  // the window's close button stays out of reach: one overshoot with the d-pad must not quit the launcher
  return [...found].filter(el => !el.matches('.win-close') && !isDisabled(el) && shown(el) && (!onScreen || isReachable(el)))
}

function rectOf(el: HTMLElement) { return el.getBoundingClientRect() }

/** Best neighbour of `from` in `dir`: close along the direction, lined up across it. */
function neighbour(from: HTMLElement, dir: Dir, targets: HTMLElement[]): HTMLElement | null {
  const c = rectOf(from)
  const cx = c.left + c.width / 2, cy = c.top + c.height / 2
  const horiz = dir === 'left' || dir === 'right'
  let best: HTMLElement | null = null
  let bestScore = Infinity

  for (const el of targets) {
    if (el === from || el.contains(from)) continue
    const r = rectOf(el)
    const rx = r.left + r.width / 2, ry = r.top + r.height / 2

    // how far along the direction (edge to edge, centre as fallback) and how far off to the side
    let along: number, side: number, overlap: number
    if (horiz) {
      const gap = dir === 'right' ? r.left - c.right : c.left - r.right
      const centre = dir === 'right' ? rx - cx : cx - rx
      if (centre <= 1) continue
      along = Math.max(gap, 0) + Math.max(0, centre - Math.abs(gap)) * 0.1
      side = Math.abs(ry - cy)
      overlap = Math.max(0, Math.min(c.bottom, r.bottom) - Math.max(c.top, r.top))
    } else {
      const gap = dir === 'down' ? r.top - c.bottom : c.top - r.bottom
      const centre = dir === 'down' ? ry - cy : cy - ry
      if (centre <= 1) continue
      along = Math.max(gap, 0) + Math.max(0, centre - Math.abs(gap)) * 0.1
      side = Math.abs(rx - cx)
      overlap = Math.max(0, Math.min(c.right, r.right) - Math.max(c.left, r.left))
    }

    // up and down: the next row first (lists and settings pages must not skip rows), then the one lined up best.
    // left and right: stay in the row, prefer what is lined up.
    const score = horiz
      ? along + side * 2 - Math.min(overlap, 200) * 0.5
      : along * 2 + side * 0.6 - Math.min(overlap, 200) * 0.2
    if (score < bestScore) { bestScore = score; best = el }
  }
  return best
}

function setCurrent(el: HTMLElement | null): void {
  if (current) current.classList.remove(FOCUS_CLASS)
  current = el
  if (!el) return
  el.classList.add(FOCUS_CLASS)
  el.scrollIntoView({ block: 'nearest', inline: 'nearest' })
  if (el.matches('input:not([type="range"]), textarea')) return // typing fields only take focus on A
  el.focus({ preventScroll: true })
}

function scrollParent(el: HTMLElement | null): HTMLElement {
  let p = el?.parentElement ?? null
  while (p) {
    const o = getComputedStyle(p).overflowY
    if ((o === 'auto' || o === 'scroll') && p.scrollHeight > p.clientHeight + 1) return p
    p = p.parentElement
  }
  return (document.querySelector('.main-content') as HTMLElement) ?? document.documentElement
}

/** Start point when nothing is focused yet: the target nearest the middle of the page content. */
function pickInitial(targets: HTMLElement[]): HTMLElement | null {
  const main = document.querySelector('.main-content')?.getBoundingClientRect()
  const mx = main ? main.left + main.width / 2 : innerWidth / 2
  const my = main ? main.top + main.height / 2 : innerHeight / 2
  let best: HTMLElement | null = null
  let bestD = Infinity
  for (const el of targets) {
    const r = rectOf(el)
    const d = Math.hypot(r.left + r.width / 2 - mx, r.top + r.height / 2 - my)
    if (d < bestD) { bestD = d; best = el }
  }
  return best
}

function move(dir: Dir): void {
  const targets = collectTargets()
  if (!targets.length) return

  if (!current || !current.isConnected) { setCurrent(pickInitial(targets)); rumble('move'); return }

  const el = current as HTMLElement
  if (document.activeElement && el.matches('input, textarea') && document.activeElement === el) el.blur()

  // sliders and plain <select>s eat left/right
  if (dir === 'left' || dir === 'right') {
    const step = dir === 'right' ? 1 : -1
    if (el instanceof HTMLInputElement && el.type === 'range') { stepRange(el, step); return }
    if (el instanceof HTMLSelectElement) { stepSelect(el, step); return }
  }

  // up and down stay inside the list you are in first, including rows scrolled out of view: setCurrent() scrolls the
  // next row in, so the ring never loses its place (and never leaps to the top bar or the middle of the page).
  if (dir === 'up' || dir === 'down') {
    const list = scrollParent(el)
    if (list.contains(el)) {
      const inList = neighbour(el, dir, collectTargets(list, false))
      if (inList) { setCurrent(inList); rumble('move'); return }
    }
  }

  const next = neighbour(el, dir, targets)
  if (next) { setCurrent(next); rumble('move'); return }
  rumble('bump')
}

function stepRange(el: HTMLInputElement, dir: number): void {
  const step = Number(el.step) || 1
  const min = Number(el.min) || 0
  const max = el.max === '' ? 100 : Number(el.max)
  const v = Math.min(max, Math.max(min, Number(el.value) + dir * step))
  el.value = String(v)
  rumble('move')
  el.dispatchEvent(new Event('input', { bubbles: true }))
  el.dispatchEvent(new Event('change', { bubbles: true }))
}

function stepSelect(el: HTMLSelectElement, dir: number): void {
  const i = Math.min(el.options.length - 1, Math.max(0, el.selectedIndex + dir))
  if (i === el.selectedIndex) return
  el.selectedIndex = i
  rumble('move')
  el.dispatchEvent(new Event('change', { bubbles: true }))
}

function confirm(): void {
  const el = current
  if (!el || !el.isConnected) { move('down'); return }
  if (el.matches('input:not([type="range"]):not([type="checkbox"]):not([type="radio"]), textarea')) {
    el.focus()
    return
  }
  rumble('confirm')
  el.click()
  // the click may have swapped the screen: keep the ring on something that still exists
  requestAnimationFrame(() => {
    if (current && (!current.isConnected || !isVisible(current))) setCurrent(null)
  })
}

/** Topmost fixed overlay (modal, drawer) at the middle of the window, if any. */
function topOverlay(): HTMLElement | null {
  let el: HTMLElement | null = document.elementFromPoint(innerWidth / 2, innerHeight / 2) as HTMLElement | null
  while (el && el !== document.body) {
    if (getComputedStyle(el).position === 'fixed' && !el.matches('#toast-dock, .topbar, .pad-hints')) return el
    el = el.parentElement
  }
  return null
}

const nextFrames = (n = 2) => new Promise<void>(res => {
  const tick = () => (--n <= 0 ? res() : requestAnimationFrame(tick))
  requestAnimationFrame(tick)
})
const domSize = () => document.body.innerHTML.length

async function back(): Promise<void> {
  rumble('back')
  const active = document.activeElement as HTMLElement | null
  if (active && active.matches('input, textarea')) { active.blur(); return }

  // 1) plain Escape: context menu, fullscreen views and select mode already listen for it
  let before = domSize()
  document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
  document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Escape', bubbles: true }))
  await nextFrames()
  if (domSize() !== before) return

  // 2) a modal or drawer: its close button, else a click on its backdrop
  const overlay = topOverlay()
  if (overlay) {
    const close = overlay.querySelector<HTMLElement>('[aria-label*="lose" i], [title*="lose" i], [class*="close" i], [class*="cancel" i], [class*="dismiss" i]')
    if (close && isVisible(close)) { close.click(); return }
    if (overlay.getBoundingClientRect().width >= innerWidth * 0.9) { overlay.click(); return }
  }

  // 3) dropdowns and popovers close on a click outside
  before = domSize()
  for (const type of ['mousedown', 'mouseup', 'click']) {
    document.body.dispatchEvent(new MouseEvent(type, { bubbles: true, button: 0 }))
  }
  await nextFrames()
  if (domSize() !== before) return

  // 4) nothing open: go up to the Hub
  if (router.currentRoute.value.path !== '/') { router.push('/'); setCurrent(null) }
}

function switchTab(dir: number): void {
  const links = [...document.querySelectorAll<HTMLAnchorElement>('.topnav .nav-item')]
  if (!links.length) return
  const i = links.findIndex(a => a.classList.contains('active'))
  const next = links[(i + dir + links.length) % links.length]
  rumble('tab')
  const href = next.getAttribute('href')
  if (href) router.push(href.replace(/^#/, ''))
  setCurrent(null)
}

/** Hold-to-accept on friend requests and party invites listens for a held Esc; X holds it for you. */
function holdAccept(pressed: boolean): void {
  if (!document.querySelector('#toast-dock .toast-accept')) return
  document.dispatchEvent(new KeyboardEvent(pressed ? 'keydown' : 'keyup', { key: 'Escape', bubbles: true }))
  window.dispatchEvent(new KeyboardEvent(pressed ? 'keydown' : 'keyup', { key: 'Escape' }))
}

function press(name: string, pressedNow: boolean, now: number, repeat = false): boolean {
  const was = !!down[name]
  down[name] = pressedNow
  if (!pressedNow) return false
  if (!was) {
    if (repeat) nextRepeat[name] = now + REPEAT_FIRST_MS
    return true
  }
  if (repeat && now >= (nextRepeat[name] ?? Infinity)) {
    nextRepeat[name] = now + REPEAT_MS
    return true
  }
  return false
}

function frame(now: number): void {
  raf = requestAnimationFrame(frame)
  const pads = (navigator.getGamepads?.() ?? []).filter((p): p is Gamepad => !!p && p.connected)
  padConnected.value = pads.length > 0
  if (!pads.length) return

  // merge all connected pads: whichever you pick up works
  const b = (i: number) => pads.some(p => p.buttons[i]?.pressed)
  const ax = (i: number) => pads.reduce((m, p) => Math.abs(p.axes[i] ?? 0) > Math.abs(m) ? (p.axes[i] ?? 0) : m, 0)

  const lx = ax(0), ly = ax(1), rY = ax(3)
  const dirs: Record<Dir, boolean> = {
    up: b(12) || ly < -DEADZONE && Math.abs(ly) > Math.abs(lx),
    down: b(13) || ly > DEADZONE && Math.abs(ly) > Math.abs(lx),
    left: b(14) || lx < -DEADZONE && Math.abs(lx) >= Math.abs(ly),
    right: b(15) || lx > DEADZONE && Math.abs(lx) >= Math.abs(ly),
  }

  let used = false
  const wake = () => { used = true }

  for (const d of ['up', 'down', 'left', 'right'] as Dir[]) {
    const pressedNow = dirs[d]
    if (pressedNow) wake()
    // first press only wakes the ring up (shows where you are); after that every press moves
    if (press(d, pressedNow, now, true)) {
      if (!padActive.value) { padActive.value = true; if (current?.isConnected && isVisible(current)) { current.classList.add(FOCUS_CLASS); continue } }
      move(d)
    }
  }

  if (press('a', b(0), now)) { wake(); if (padActive.value) confirm() }
  if (press('b', b(1), now)) { wake(); void back() }
  if (press('lb', b(4), now)) { wake(); switchTab(-1) }
  if (press('rb', b(5), now)) { wake(); switchTab(1) }
  if (press('y', b(3), now)) { wake(); rumble('confirm'); togglePalette() }
  if (press('start', b(9), now)) { wake(); router.push('/'); setCurrent(null) }

  const xNow = b(2)
  const xWas = !!down.x
  if (xNow !== xWas) { holdAccept(xNow); wake(); if (xNow && document.querySelector('#toast-dock .toast-accept')) rumble('confirm') }
  down.x = xNow

  if (Math.abs(rY) > 0.15) {
    wake()
    const sp = scrollParent(current)
    sp.scrollTop += rY * SCROLL_SPEED
  }

  if (used && !padActive.value) padActive.value = true
  if (used) document.documentElement.classList.add('pad-mode', 'pad-nocursor')
}

function sleep(): void {
  if (!padActive.value) return
  padActive.value = false
  document.documentElement.classList.remove('pad-mode', 'pad-nocursor')
  current?.classList.remove(FOCUS_CLASS)
}

/** Call once at startup. Safe to call twice. */
export function startGamepad(): void {
  if (started) return
  started = true
  raf = requestAnimationFrame(frame)

  // mouse or keyboard takes over again (synthetic events from this file don't count)
  window.addEventListener('mousemove', e => { if (e.isTrusted && (e.movementX || e.movementY)) sleep() }, true)
  window.addEventListener('mousedown', e => { if (e.isTrusted) sleep() }, true)
  window.addEventListener('keydown', e => { if (e.isTrusted) sleep() }, true)

  window.addEventListener('gamepadconnected', () => { padConnected.value = true })
  window.addEventListener('gamepaddisconnected', () => { padConnected.value = !!navigator.getGamepads?.().some(p => p?.connected) })
}

export function stopGamepad(): void {
  cancelAnimationFrame(raf)
  started = false
}
