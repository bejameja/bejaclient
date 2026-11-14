<template>
  <div class="launch-wrap" :class="{ 'launch-wrap--text': props.textOnly }" ref="rootEl">
  <div class="launch-split" :class="[status, { ready: props.readyMode && lobbyStore.isReady }]">

    <!-- Main launch area -->
    <button
      class="launch-main"
      :class="status"
      :disabled="status === 'starting' || status === 'stopping' || (status === 'idle' && isMaintenance) || (status === 'idle' && isPartyLeader && !lobbyStore.canLaunch)"
      @click="onLaunch"
      @mouseenter="startScramble"
      @mouseleave="stopScramble"
    >
      <!-- Idle + BejaClient profile selected: blocked, maintenance clock -->
      <template v-if="status === 'idle' && isMaintenance">
        <svg class="launch-rocket launch-rocket--maint" width="27" height="27" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="9" />
          <polyline points="12 7 12 12 15.5 13.8" />
        </svg>
        <div class="launch-text">
          <span class="launch-label launch-label--title launch-label--maint" style="font-size: 14px;">MAINTENANCE</span>
          <span v-if="activeProfile" class="launch-version launch-version--maint">{{ versionLabel }}</span>
        </div>
      </template>

      <!-- Idle: flat centered label, no icon -->
      <template v-else-if="status === 'idle'">
        <span class="launch-label launch-label--title"><span class="lg" :class="{ scr: scrambled !== null }" :data-scr="scrambled ?? ''">{{ idleText.charAt(0) }}</span>{{ idleText.slice(1) }}</span>
      </template>

      <!-- Starting -->
      <template v-else-if="status === 'starting'">
        <div class="launch-state-overlay">
          <svg class="launch-icon spin" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
          </svg>
          <span class="launch-label launch-label--status">{{ store.statusMsg || $t('launch.starting') }}</span>
        </div>
      </template>

      <!-- Running -->
      <template v-else-if="status === 'running'">
        <div class="launch-state-overlay">
          <span class="launch-label">{{ $t('launch.running') }}</span>
        </div>
      </template>

      <!-- Error -->
      <template v-else-if="status === 'error'">
        <div class="launch-state-overlay">
          <span class="launch-label launch-label--error">{{ $t('launch.retry') }}</span>
        </div>
      </template>
    </button>

    <!-- Dropdown chevron - swaps to a cancel/close "X" while starting or running -->
    <button
      class="launch-chevron"
      :class="{ cancel: showCancelX }"
      :title="status === 'starting' ? 'Cancel launch' : status === 'running' ? 'Close Minecraft' : undefined"
      @click.stop="showCancelX ? store.killGame() : toggleDropdown()"
      :disabled="status === 'stopping'"
    >
      <svg v-if="showCancelX" xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
      <svg v-else xmlns="http://www.w3.org/2000/svg" width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
    </button>

  </div>

  <!-- Text variant: the selected profile, shown under LAUNCH (change it with the arrow -> Profiles tab) -->
  <div v-if="props.textOnly && activeProfile" class="launch-profile" :title="activeProfile.name">
    <span class="launch-profile-name">{{ activeProfile.name }}</span>
    <span v-if="versionLabel" class="launch-profile-version">{{ versionLabel }}</span>
  </div>

  <!-- Profile dropdown - sibling of .launch-split (not a child), since
       .launch-split has overflow:hidden for its conic-gradient border and
       would otherwise clip the dropdown, which sits entirely above it. -->
  <Transition name="dropdown-fade">
    <div v-if="dropdownOpen" class="profile-dropdown">
      <div class="dropdown-label">{{ $t('launch.profile') }}</div>
      <button
        v-for="profile in profiles"
        :key="profile.id"
        class="dropdown-item"
        :class="{ active: profile.id === activeProfile?.id }"
        @click="selectProfile(profile.id)"
      >
        <span class="dropdown-item-name">{{ profile.name }}</span>
        <span class="dropdown-item-version">{{ profile.version }}</span>
      </button>
      <div v-if="profiles.length === 0" class="dropdown-empty">{{ $t('launch.noProfiles') }}</div>
    </div>
  </Transition>

  <Transition name="err-fade">
    <div v-if="status === 'error' && store.lastError" class="launch-error">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><circle cx="6" cy="6" r="5" stroke="currentColor" stroke-width="1.2"/><line x1="6" y1="3.5" x2="6" y2="6.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/><circle cx="6" cy="8.5" r="0.6" fill="currentColor"/></svg>
      <span class="launch-error-text">{{ store.lastError }}</span>
    </div>
  </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useLauncherStore } from '../../store/launcherStore'
import { useLobbyStore } from '../../store/lobbyStore'

const props = defineProps<{ readyMode?: boolean; textOnly?: boolean }>()
// Hover effect: the first letter cycles through random letters for ~0.5 s (10 steps), then lands on the real one.
const SCRAMBLE_GLYPHS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789#%&?'
const SCRAMBLE_STEPS = 10
const SCRAMBLE_MS = 20
const scrambled = ref<string | null>(null)
let scrambleTimer: ReturnType<typeof setInterval> | null = null
function stopScramble() {
  if (scrambleTimer) { clearInterval(scrambleTimer); scrambleTimer = null }
  scrambled.value = null
}
function startScramble() {
  if (!props.textOnly) return   // no reduced-motion check on purpose: it is a letter swap, nothing moves
  stopScramble()
  let step = 0
  const tick = () => {
    if (++step > SCRAMBLE_STEPS) { stopScramble(); return }
    const real = idleText.value.charAt(0)
    let g = real
    while (g === real) g = SCRAMBLE_GLYPHS.charAt(Math.floor(Math.random() * SCRAMBLE_GLYPHS.length))
    scrambled.value = g
  }
  tick()
  scrambleTimer = setInterval(tick, SCRAMBLE_MS)
}
const idleText = computed(() => props.readyMode ? (lobbyStore.isReady ? 'READY' : 'READY UP') : 'LAUNCH')

const store = useLauncherStore()
const lobbyStore = useLobbyStore()
const status = computed(() => store.status)
const profiles = computed(() => store.profiles)
const activeProfile = computed(() => store.activeProfile)
const isMaintenance = computed(() => store.isBejaMaintenance)
const router = useRouter()
const dropdownOpen = ref(false)
const rootEl = ref<HTMLElement | null>(null)
const showCancelX = computed(() => status.value === 'starting' || status.value === 'running')
const isPartyLeader = computed(() => !!lobbyStore.party && lobbyStore.isLeader)

const LOADER_NAMES: Record<string, string> = {
  fabric: 'Fabric',
  forge: 'Forge',
  neoforge: 'NeoForge',
  quilt: 'Quilt',
  optifine: 'OptiFine',
  vanilla: '',
}

const versionLabel = computed(() => {
  const p = activeProfile.value
  if (!p) return ''
  if (p.useBejaClient) return `BJC ${p.version}`
  const name = LOADER_NAMES[p.loader] ?? p.loader
  return name ? `${name} ${p.version}` : p.version
})

function onLaunch() {
  if (status.value === 'running') {
    store.killGame()
  } else if (status.value === 'idle' && props.readyMode) {
    lobbyStore.toggleReady()
  } else if (status.value === 'idle' || status.value === 'error') {
    store.launch()
  }
}

// The arrow does not open a menu: it sends you to the Profiles tab in pick mode (?pick=1). Clicking a profile there makes it the
// active one and returns to the hub. Opening Profiles normally keeps the usual behaviour (click = detail/edit view).
function toggleDropdown() {
  void router.push({ name: 'profiles', query: { pick: '1' } })
}

async function selectProfile(id: string) {
  await store.setActiveProfile(id)
  dropdownOpen.value = false
}

function onClickOutside(e: MouseEvent) {
  if (dropdownOpen.value && rootEl.value && !rootEl.value.contains(e.target as Node)) {
    dropdownOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('mousedown', onClickOutside, true)
})
onUnmounted(() => { document.removeEventListener('mousedown', onClickOutside, true); stopScramble() })
</script>

<style lang="scss" scoped>
// LabyMod-style reference - blue/indigo gradient glass pill, play icon +
// left-aligned title/subtitle text block, swap icon on the right.
$btn-blue-1: #7d8ae6;
$btn-blue-2: #4a4fc4;
$btn-error:  #ff453a;

.launch-wrap {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.launch-error {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  padding: 6px 10px;
  background: $surface-elevated;
  border: 1px solid $border;
  border-radius: $radius;
  max-width: 340px;
  color: $text-secondary;
  cursor: pointer;
  user-select: text;

  &--maint {
    background: #0a0a0a;
    border-color: rgba(255, 255, 255, 0.08);
    border-radius: $radius-sm;

    .launch-error-text { color: $btn-error; font-weight: 600; }
  }
}

.launch-error-text {
  font-size: 11px;
  color: $text-secondary;
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.45;
}

.err-fade-enter-active, .err-fade-leave-active { transition: opacity 0.2s ease; }
.err-fade-enter-from, .err-fade-leave-to { opacity: 0; }

// Wooden sign: oak planks with seams and grain, a dark frame with the game's bevel, nail heads where the two ropes
// attach, the title in the Minecrafter pixel font with the dark drop shadow.
$wood-dark: #3b2a14;

.launch-split {
  --wood: #8c6a3c;
  position: relative;
  display: flex;
  align-items: stretch;
  width: 272px;
  height: 72px;
  border-radius: 0;
  overflow: hidden;
  border: 3px solid $wood-dark;
  background:
    repeating-linear-gradient(to bottom, transparent 0 21px, rgba(40, 26, 10, 0.55) 21px 23px),   // plank seams
    repeating-linear-gradient(90deg, rgba(0, 0, 0, 0.05) 0 3px, transparent 3px 8px),             // grain
    linear-gradient(to bottom, rgba(255, 230, 170, 0.14), transparent 45%),                        // light from above
    var(--wood);
  box-shadow:
    inset 2px 2px 0 rgba(255, 225, 160, 0.35),
    inset -2px -2px 0 rgba(0, 0, 0, 0.4),
    0 10px 26px rgba(0, 0, 0, 0.55);
  transform-origin: 50% -8px;
  transition: filter 120ms ease-out;

  // nail heads, in line with the ropes (HomePage .rope)
  &::before,
  &::after {
    content: '';
    position: absolute;
    top: 4px;
    width: 6px;
    height: 6px;
    background: #20160a;
    box-shadow: inset 1px 1px 0 rgba(255, 230, 170, 0.35);
    z-index: 2;
  }
  &::before { left: 25px; }
  &::after { right: 26px; }

  &:hover {
    filter: brightness(1.12);
    animation: sign-sway 1400ms ease-in-out;
  }

  &.ready { --wood: #5f7d3a; }

  &.error { --wood: #7a3d2e; }
}

@keyframes sign-sway {
  0%, 100% { transform: rotate(0deg); }
  25% { transform: rotate(0.9deg); }
  60% { transform: rotate(-0.6deg); }
}

.launch-main {
  flex: 1;
  position: relative;
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 0 16px;
  background: none;
  border: none;
  border-radius: 0;
  outline: none;
  cursor: pointer;
  overflow: hidden;
  transition: transform 100ms cubic-bezier(0.34, 1.56, 0.64, 1);

  &:active:not(:disabled) { transform: scale(0.98); }
  &:disabled { cursor: not-allowed; opacity: 0.7; }
}

.launch-icon {
  flex-shrink: 0;
  color: #fff;
  &.spin { animation: spin 0.9s linear infinite; }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@keyframes rocket-nudge {
  0%, 100% { transform: translateY(0) rotate(0deg); }
  50% { transform: translateY(-2px) rotate(-4deg); }
}

.launch-rocket {
  flex-shrink: 0;
  color: #fff;

  .launch-main:hover:not(:disabled) & {
    animation: rocket-nudge 900ms ease-in-out infinite;
  }
}

.launch-text {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 1px;
}

.launch-label {
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 17px;
  font-weight: 800;
  letter-spacing: 0.02em;
  line-height: 1.2;
  color: #fff;
  text-shadow: 2px 2px 0 $wood-dark;

  &--title {
    font-family: 'Minecrafter', 'Plus Jakarta Sans', sans-serif;
    font-weight: 400;
    font-size: 26px;
    letter-spacing: 0.12em;
    // the pixel font's glyph box is taller than its line box
    padding-top: 3px;
  }

  &--status {
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
    color: #fff;
  }

  &--error { color: #fff; }

  // Same font/weight as the top bar's Hub/Locker/etc. nav labels (TopBar.vue's
  // .nav-label, which just inherits the app's default $font-family/weight 500)
  // instead of the branded Plus Jakarta Sans 800 used for "LAUNCH".
  &--maint {
    font-family: $font-family;
    font-weight: 500;
  }
}

.launch-version {
  text-align: left;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 12px;
  font-weight: 500;
  color: #fff;
  line-height: 1.2;
  background: transparent;
  text-shadow: 1px 1px 0 $wood-dark;
}


.launch-state-overlay {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: flex-start;
  gap: 10px;
  width: 100%;
  height: 100%;
}

.launch-chevron {
  width: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.2);
  border: none;
  border-radius: 0;   // the global button reset rounds corners
  border-left: 3px solid $wood-dark;
  outline: none;
  color: #fff;
  cursor: pointer;
  transition: color 120ms, background 120ms;
  flex-shrink: 0;

  svg { filter: drop-shadow(2px 2px 0 #{$wood-dark}); }

  &:hover:not(:disabled) { background: rgba(255, 255, 255, 0.14); }
  &:disabled { cursor: not-allowed; opacity: 0.5; }

  &.cancel:hover { color: #ffb4ab; background: rgba(120, 20, 10, 0.35); }
}

// ── Text-only variant (hub corner): white pixel text, no sign, no frame ──────────────────────────────────────
.launch-wrap--text {
  align-items: flex-start;

  .launch-split {
    width: auto;
    height: auto;
    border: 0;
    background: none;
    box-shadow: none;
    overflow: visible;
    align-items: center;
    gap: 2px;
    animation: none;
    transform: none;
    &::before, &::after { display: none; }
    &:hover { filter: none; animation: none; }
  }

  .launch-main { flex: none; padding: 6px 10px; overflow: visible; }

  .launch-label { text-shadow: 2px 2px 0 rgba(0, 0, 0, 0.55); }
  .launch-label--title { font-size: 44px; letter-spacing: 0.14em; opacity: 0.92; transition: opacity 120ms, color 160ms; }
  // very light green tint on hover
  .launch-main:hover:not(:disabled) .launch-label--title { opacity: 1; color: #d2f7cc; }

  .launch-chevron {
    width: 60px;
    height: 60px;
    background: none;
    border: 0;
    opacity: 0.92;
    svg { width: 40px; height: 40px; stroke-width: 5.2; stroke-linecap: square; stroke-linejoin: miter; filter: drop-shadow(1px 1px 0 rgba(0, 0, 0, 0.55)); }
    align-self: center;
    margin-top: -24px;   // optical centre on the pixel-font caps (the font box sits low: shifts the arrow up ~12px)
    &:hover:not(:disabled) { background: none; opacity: 1; }
  }

  .profile-dropdown { left: 0; transform: none; bottom: calc(100% + 6px); }
}
// first letter scramble (see startScramble): the real letter keeps its space (transparent), a random one is drawn over it
.launch-wrap--text .lg {
  position: relative;
  display: inline-block;

  &.scr { color: transparent; text-shadow: none; }
  &.scr::before {
    content: attr(data-scr);
    position: absolute;
    left: 0;
    top: 0;
    color: #d2f7cc;
    text-shadow: 2px 2px 0 rgba(0, 0, 0, 0.55);
  }
}
.launch-profile {
  display: flex;
  align-items: baseline;
  gap: 8px;
  max-width: 360px;
  margin: -6px 0 0 10px;
  font-family: 'Plus Jakarta Sans', sans-serif;
  text-shadow: 1px 1px 0 rgba(0, 0, 0, 0.6);
}
.launch-profile-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 15px;
  font-weight: 700;
  color: #fff;
}
.launch-profile-version {
  flex: none;
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.6);
}
.launch-wrap--text :deep(.dropdown-fade-enter-from),
.launch-wrap--text :deep(.dropdown-fade-leave-to) { transform: translateY(6px); }

// ── Dropdown ──────────────────────────────────────────────────────────────────
.profile-dropdown {
  position: absolute;
  bottom: calc(100% + 10px);
  left: 50%;
  width: 300px;
  transform: translateX(-50%);
  background: #0a0a0a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: $radius-sm;
  overflow: hidden;
  z-index: 10;
}

.dropdown-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: $muted;
  padding: 10px 14px 6px;
}

.dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 9px 14px;
  background: none;
  border: none;
  color: $text-secondary;
  cursor: pointer;
  font-size: 13px;
  transition: background 120ms, color 120ms;

  &:hover {
    background: #1a1a1a;
    color: $text-primary;
  }

  &.active {
    color: $text-primary;
    .dropdown-item-name { font-weight: 600; }
  }
}

.dropdown-item-name {
  font-weight: 500;
}

.dropdown-item-version {
  font-size: 11px;
  color: $muted;
}

.dropdown-item-maint {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: $btn-error;
}

.dropdown-empty {
  padding: 12px 14px;
  font-size: 12px;
  color: $muted;
  font-style: italic;
}

.dropdown-fade-enter-active, .dropdown-fade-leave-active { transition: opacity 0.15s ease, transform 0.15s ease; }
.dropdown-fade-enter-from, .dropdown-fade-leave-to { opacity: 0; transform: translateX(-50%) translateY(6px); }
</style>
