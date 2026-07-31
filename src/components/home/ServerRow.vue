<template>
  <div class="server-row">
    <button
      class="collapse-toggle"
      :class="{ collapsed }"
      :title="collapsed ? 'Show servers' : 'Hide servers'"
      @click="toggleCollapsed"
    >
      <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
        <path d="M1 1L5 4L1 7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>
    <div class="server-icons-wrap" :class="{ collapsed }">
      <div class="server-icons">
        <Tooltip
          v-for="i in 10"
          :key="i"
          :text="`${current(i - 1).name} — ${ipLabel(current(i - 1))}`"
          placement="top"
        >
          <div class="server-slot" :class="{ joining: joiningId === current(i - 1).id }" @click="joinServer(current(i - 1))">
            <div class="flip-card" :class="{ flipped: flipped[i - 1] }">
              <div class="flip-face front" :class="{ offline: !front(i - 1).online }">
                <img v-if="front(i - 1).favicon" :src="front(i - 1).favicon!" :alt="front(i - 1).name" />
                <span v-else class="server-icon-fallback">{{ front(i - 1).name.charAt(0) }}</span>
              </div>
              <div class="flip-face back" :class="{ offline: !back(i - 1).online }">
                <img v-if="back(i - 1).favicon" :src="back(i - 1).favicon!" :alt="back(i - 1).name" />
                <span v-else class="server-icon-fallback">{{ back(i - 1).name.charAt(0) }}</span>
              </div>
            </div>
          </div>
        </Tooltip>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import Tooltip from '../common/Tooltip.vue'
import { showToast } from '../../composables/useToasts'
import { useLauncherStore } from '../../store/launcherStore'

const launcherStore = useLauncherStore()

interface ServerEntry { id: string; name: string; host: string; port: number }

// 20 of the real entries from serverPingService.ts's FEATURED_SERVERS list —
// kept in sync with those host/port values rather than invented separately.
// Slot i shows SERVERS[i] on the front face and SERVERS[i + 10] on the back.
const SERVERS: ServerEntry[] = [
  { id: 'hypixel',        name: 'Hypixel',         host: 'mc.hypixel.net',         port: 25565 },
  { id: '2b2t',           name: '2b2t',            host: 'connect.2b2t.org',       port: 25565 },
  { id: 'wynncraft',      name: 'Wynncraft',       host: 'play.wynncraft.com',     port: 25565 },
  { id: 'cubecraft',      name: 'CubeCraft',       host: 'play.cubecraft.net',     port: 25565 },
  { id: 'gommehd',        name: 'GommeHD',         host: 'gommehd.net',            port: 25565 },
  { id: 'griefergames',   name: 'GrieferGames',    host: 'griefergames.net',       port: 25565 },
  { id: 'pikanetwork',    name: 'PikaNetwork',     host: 'play.pika-network.net',  port: 25565 },
  { id: 'donutsmp',       name: 'DonutSMP',        host: 'donutsmp.net',           port: 25565 },
  { id: 'earthmc',        name: 'EarthMC',         host: 'play.earthmc.net',       port: 25565 },
  { id: 'taswell',        name: 'Taswell Network', host: 'taswell.io',             port: 25565 },
  { id: 'hugosmp',        name: 'HugoSMP',         host: 'hugosmp.net',            port: 25565 },
  { id: 'empireminecraft',name: 'Empire Minecraft',host: 'play.emc.gs',            port: 25565 },
  { id: 'minemenclub',    name: 'Minemen Club',    host: 'minemen.club',           port: 25565 },
  { id: 'purpleprison',   name: 'Purple Prison',   host: 'purpleprison.org',       port: 25565 },
  { id: 'jartexnetwork',  name: 'JartexNetwork',   host: 'top.jartex.fun',         port: 25565 },
  { id: 'opblocks',       name: 'OPBlocks',        host: 'play.opblocks.com',      port: 25565 },
  { id: 'manacube',       name: 'ManaCube',        host: 'buzz.manacube.com',      port: 25565 },
  { id: 'complexgaming',  name: 'Complex Gaming',  host: 'hub.mc-complex.com',     port: 25565 },
  { id: 'minesuperior',   name: 'MineSuperior',    host: 'hub.minesuperior.com',   port: 25565 },
  { id: 'akumamc',        name: 'AkumaMC',         host: 'akumamc.net',           port: 25565 },
]

interface ServerState extends ServerEntry {
  favicon: string | null
  online: boolean
}

const states = ref<ServerState[]>(SERVERS.map(s => ({ ...s, favicon: null, online: false })))
const flipped = ref<boolean[]>(Array(10).fill(false))

const COLLAPSE_KEY = 'beja_server_row_collapsed'
const storedCollapse = localStorage.getItem(COLLAPSE_KEY)
const collapsed = ref(storedCollapse === null ? true : storedCollapse === '1')

function toggleCollapsed() {
  collapsed.value = !collapsed.value
  localStorage.setItem(COLLAPSE_KEY, collapsed.value ? '1' : '0')
}

function front(i: number): ServerState { return states.value[i] }
function back(i: number): ServerState { return states.value[i + 10] }
function current(i: number): ServerState { return flipped.value[i] ? back(i) : front(i) }

function ipLabel(s: ServerEntry): string {
  return s.port === 25565 ? s.host : `${s.host}:${s.port}`
}

const joiningId = ref<string | null>(null)

async function joinServer(s: ServerEntry) {
  if (joiningId.value) return

  const profile = launcherStore.activeProfile
  if (!profile) {
    showToast({ title: 'No profile selected', body: 'Pick a profile before joining a server.', variant: 'error', duration: 3500 })
    return
  }

  joiningId.value = s.id
  try {
    await window.api.lobby.startWithServer(profile.id, s.host, s.port)
  } catch (e) {
    showToast({ title: 'Failed to join', body: e instanceof Error ? e.message : String(e), variant: 'error', duration: 4000 })
  } finally {
    joiningId.value = null
  }
}

let rotateTimer: ReturnType<typeof setInterval> | null = null
const slotTimers: ReturnType<typeof setTimeout>[] = []

onMounted(() => {
  SERVERS.forEach(async (s, i) => {
    try {
      const status = await window.api.servers.ping(s.host, s.port)
      if (status) states.value[i] = { ...states.value[i], favicon: status.favicon, online: true }
    } catch {
      // stays offline — favicon fallback shows the server's initial instead
    }
  })

  // Every cycle, flip each of the 10 slots one after another (a ripple across
  // the row) instead of swapping all ten at once.
  rotateTimer = setInterval(() => {
    for (let i = 0; i < 10; i++) {
      slotTimers.push(setTimeout(() => { flipped.value[i] = !flipped.value[i] }, i * 110))
    }
  }, 6000)
})

onUnmounted(() => {
  if (rotateTimer) clearInterval(rotateTimer)
  slotTimers.forEach(clearTimeout)
})
</script>

<style lang="scss" scoped>
.server-row {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  margin-left: 16px;
  -webkit-app-region: no-drag;
}

.collapse-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  padding: 0;
  border: none;
  background: transparent;
  color: $text-muted;
  cursor: pointer;
  transition: color 150ms ease;
  -webkit-app-region: no-drag;

  svg { transition: transform 220ms cubic-bezier(0.4, 0.1, 0.2, 1); }

  &.collapsed svg { transform: rotate(180deg); }

  &:hover { color: $text-primary; }
}

.server-icons-wrap {
  overflow: hidden;
  max-width: 420px;
  margin-left: 6px;
  transition: max-width 480ms cubic-bezier(0.34, 1.56, 0.64, 1), margin-left 480ms cubic-bezier(0.34, 1.56, 0.64, 1);

  &.collapsed {
    max-width: 0;
    margin-left: 0;
  }
}

.server-icons {
  display: flex;
  justify-content: center;
  gap: 26px;
}

.server-slot {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  perspective: 240px;
  cursor: pointer;
  opacity: 1;
  transform: scale(1) translateX(0);
  transition: opacity 280ms ease, transform 380ms cubic-bezier(0.34, 1.56, 0.64, 1);

  &:hover .flip-card { transform: translateY(-2px) rotateX(var(--flip, 0deg)); }
  &:active .flip-card { transform: scale(0.95) rotateX(var(--flip, 0deg)); }

  &.joining {
    cursor: default;
    pointer-events: none;
    animation: server-slot-pulse 900ms ease-in-out infinite;
  }
}

@keyframes server-slot-pulse {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.4; }
}

// Staggered reveal — each icon pops in slightly after the previous one when
// expanding, and collapses in reverse order (last icon retracts first) since
// the delay is keyed to distance from the toggle arrow either way.
@for $i from 1 through 10 {
  .server-icons > .tooltip-trigger:nth-child(#{$i}) .server-slot {
    transition-delay: #{$i * 28}ms;
  }
}

.server-icons-wrap.collapsed .server-slot {
  opacity: 0;
  transform: scale(0.3) translateX(-14px);
  transition-delay: 0ms;
}

@for $i from 1 through 10 {
  .server-icons-wrap.collapsed .server-icons > .tooltip-trigger:nth-child(#{$i}) .server-slot {
    transition-delay: #{(10 - $i) * 22}ms;
  }
}

.flip-card {
  position: relative;
  width: 100%;
  height: 100%;
  --flip: 0deg;
  transform: rotateX(var(--flip));
  transform-style: preserve-3d;
  transition: transform 500ms cubic-bezier(0.4, 0.1, 0.2, 1);

  &.flipped { --flip: 180deg; }
}

.flip-face {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  backface-visibility: hidden;
  transition: opacity 300ms ease;

  &.offline { opacity: 0.45; }

  img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    image-rendering: pixelated;
  }
}

.flip-face.back {
  transform: rotateX(180deg);
}

.server-icon-fallback {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: $surface;
  border: 1px solid $border;
  font-size: 13px;
  font-weight: 700;
  color: $text-secondary;
}
</style>
