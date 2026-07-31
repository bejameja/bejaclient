<template>
  <Teleport to="body">
    <Transition name="ctx-pop">
      <div
        v-if="state.open"
        ref="menuRef"
        class="ctx-menu"
        :style="{ left: pos.x + 'px', top: pos.y + 'px' }"
      >
        <template v-for="(item, i) in state.items" :key="i">
          <div v-if="item.separator" class="ctx-sep" />
          <button
            v-else
            class="ctx-item"
            :class="{ danger: item.danger, disabled: item.disabled }"
            :disabled="item.disabled"
            @click="run(item)"
          >
            <Icon v-if="item.icon" :name="item.icon" :size="15" />
            <span>{{ item.label }}</span>
          </button>
        </template>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { computePosition, shift, flip, offset } from '@floating-ui/dom'
import { useContextMenu, closeContextMenu, type ContextMenuItem } from '../../composables/useContextMenu'
import Icon from './Icon.vue'

const { state } = useContextMenu()
const menuRef = ref<HTMLElement | null>(null)
const pos = ref({ x: 0, y: 0 })

watch(() => state.open, async (open) => {
  if (!open) return
  await nextTick()
  if (!menuRef.value) return
  const virtualEl = {
    getBoundingClientRect: () => new DOMRect(state.x, state.y, 0, 0),
  }
  const result = await computePosition(virtualEl, menuRef.value, {
    placement: 'right-start',
    middleware: [offset(2), flip(), shift({ padding: 8 })],
  })
  pos.value = { x: result.x, y: result.y }
})

function run(item: ContextMenuItem) {
  if (item.disabled) return
  item.onClick?.()
  closeContextMenu()
}

function onOutsideClick(e: MouseEvent) {
  if (!state.open) return
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) closeContextMenu()
}

function onEscape(e: KeyboardEvent) {
  if (e.key === 'Escape') closeContextMenu()
}

document.addEventListener('mousedown', onOutsideClick)
document.addEventListener('keydown', onEscape)
</script>

<style lang="scss" scoped>
@import '../../styles/motion';

.ctx-menu {
  @extend %glass-panel;
  position: fixed;
  z-index: 9500;
  min-width: 180px;
  padding: 6px;
  border-radius: $radius;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.ctx-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 9px;
  border-radius: $radius-sm;
  border: none;
  background: transparent;
  color: $text-primary;
  font-size: 12.5px;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: background $transition-fast;

  &:hover:not(.disabled) { background: rgba(255, 255, 255, 0.08); }

  &.danger { color: $error; }
  &.disabled { opacity: 0.4; cursor: not-allowed; }
}

.ctx-sep {
  height: 1px;
  margin: 4px 2px;
  background: $border;
}
</style>
