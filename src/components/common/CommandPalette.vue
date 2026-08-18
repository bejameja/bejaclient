<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="open" class="palette-overlay" @click.self="closePalette">
        <div class="palette-panel">
          <div class="palette-search">
            <Icon name="search" :size="16" class="search-icon" />
            <input
              ref="inputRef"
              v-model="query"
              type="text"
              placeholder="Jump to a page, launch a profile..."
              @keydown.down.prevent="move(1)"
              @keydown.up.prevent="move(-1)"
              @keydown.enter.prevent="execute(highlighted)"
              @keydown.esc="closePalette"
            />
            <kbd class="kbd-hint">Esc</kbd>
          </div>

          <div class="palette-results">
            <button
              v-for="(cmd, i) in results"
              :key="cmd.id"
              class="palette-item"
              :class="{ active: i === highlighted }"
              @mouseenter="highlighted = i; playHover()"
              @click="execute(i)"
            >
              <Icon :name="cmd.icon" :size="16" />
              <span class="item-label">{{ cmd.label }}</span>
              <Icon name="corner-down-left" :size="13" class="item-hint" />
            </button>
            <div v-if="!results.length" class="palette-empty">No matches</div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useCommandPalette, closePalette } from '../../composables/useCommandPalette'
import { useLauncherStore } from '../../store/launcherStore'
import { playHover, playClick } from '../../composables/useSounds'
import Icon from './Icon.vue'
import type { IconName } from './icons'

interface Command {
  id: string
  label: string
  icon: IconName
  run: () => void
}

const { open } = useCommandPalette()
const router = useRouter()
const launcherStore = useLauncherStore()

const query = ref('')
const highlighted = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

const navCommands: Command[] = [
  { id: 'nav-home',     label: 'Go to Home',      icon: 'play',     run: () => router.push('/') },
  { id: 'nav-locker',   label: 'Go to Locker',    icon: 'sparkles', run: () => router.push('/cosmetics') },
  { id: 'nav-explore',  label: 'Go to Explore (Mods)', icon: 'search', run: () => router.push('/mods') },
  { id: 'nav-profiles', label: 'Go to Profiles',  icon: 'folder',   run: () => router.push('/profiles') },
  { id: 'nav-friends',  label: 'Go to Friends',   icon: 'link',     run: () => router.push('/friends') },
  { id: 'nav-shop',     label: 'Go to Store',     icon: 'download', run: () => router.push({ path: '/store', query: { tab: 'shop' } }) },
  { id: 'nav-quests',   label: 'Go to Quests',    icon: 'check',    run: () => router.push('/quests') },
  { id: 'nav-settings', label: 'Go to Settings',  icon: 'settings', run: () => router.push('/settings') },
  { id: 'add-friend',   label: 'Add a friend',    icon: 'plus',     run: () => router.push('/friends') },
]

const commands = computed<Command[]>(() => {
  const launchCommands: Command[] = launcherStore.profiles.map(p => ({
    id: `launch-${p.id}`,
    label: `Launch ${p.name}`,
    icon: 'play',
    run: async () => {
      await launcherStore.setActiveProfile(p.id)
      await launcherStore.launch()
      router.push('/')
    },
  }))
  return [...navCommands, ...launchCommands]
})

function score(q: string, target: string): number {
  const query = q.toLowerCase().trim()
  const t = target.toLowerCase()
  if (!query) return 1
  if (t.startsWith(query)) return 100 - t.length * 0.01
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  if (new RegExp(`\\b${escaped}`).test(t)) return 60 - t.length * 0.01
  let ti = 0
  for (const ch of query) {
    ti = t.indexOf(ch, ti)
    if (ti === -1) return -1
    ti++
  }
  return 20 - t.length * 0.01
}

const results = computed(() => {
  return commands.value
    .map(cmd => ({ cmd, s: score(query.value, cmd.label) }))
    .filter(r => r.s >= 0)
    .sort((a, b) => b.s - a.s)
    .slice(0, 8)
    .map(r => r.cmd)
})

watch(results, () => { highlighted.value = 0 })

watch(open, async (isOpen) => {
  if (isOpen) {
    query.value = ''
    highlighted.value = 0
    await nextTick()
    inputRef.value?.focus()
  }
})

function move(delta: number) {
  if (!results.value.length) return
  highlighted.value = (highlighted.value + delta + results.value.length) % results.value.length
  playHover()
}

function execute(index: number) {
  const cmd = results.value[index]
  if (!cmd) return
  playClick()
  cmd.run()
  closePalette()
}
</script>

<style lang="scss" scoped>
@font-face {
  src: url('../../assets/fonts/mojangles.ttf') format('truetype');
}

.palette-overlay {
  position: fixed;
  inset: 0;
  z-index: 9800;
  background: rgba(0, 0, 0, 0.65);
  display: flex;
  justify-content: center;
  padding-top: 14vh;
}

.palette-panel {
  width: 560px;
  max-width: 90vw;
  max-height: 60vh;
  background: #0d0d0d;
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 0;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: fit-content;
}

.palette-search {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  flex-shrink: 0;

  .search-icon { color: rgba(255, 255, 255, 0.35); flex-shrink: 0; }

  input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    color: #fff;
    font-size: 14px;
    font-family: $font-family;

    &::placeholder { color: rgba(255, 255, 255, 0.3); }
  }
}

.kbd-hint {
  font-size: 9px;
  font-weight: 400;
  color: rgba(255, 255, 255, 0.5);
  background: none;
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 0;
  padding: 3px 6px;
}

.palette-results {
  overflow-y: auto;
  padding: 4px;
}

.palette-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 9px 10px;
  border-radius: 0;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.55);
  font-size: 13px;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: background 100ms, color 100ms;

  .item-label { flex: 1; }
  .item-hint { opacity: 0; color: rgba(255, 255, 255, 0.4); transition: opacity 100ms; }

  &.active {
    background: color-mix(in srgb, var(--accent, #27ade0) 16%, transparent);
    color: #fff;
    box-shadow: inset 2px 0 0 var(--accent, #27ade0);

    .item-hint { opacity: 1; }
  }
}

.palette-empty {
  padding: 24px;
  text-align: center;
  color: rgba(255, 255, 255, 0.35);
  font-size: 12px;
}
</style>
