<template>
  <div class="store-page">
    <div class="tab-row">
      <button
        v-for="t in tabs"
        :key="t.key"
        :ref="(el) => setTabBtnRef(t.key, el as HTMLElement | null)"
        class="tab-btn"
        :class="{ active: activeTab === t.key }"
        @click="activeTab = t.key"
      >
        {{ t.label }}
        <span v-if="t.wip" class="tab-wip">WIP</span>
      </button>
      <span class="tab-indicator" :style="indicatorStyle" />
    </div>

    <div class="store-content">
      <WipPage v-show="activeTab === 'pass'" label="Client Pass" sub="This feature is coming soon." />
      <WipPage v-show="activeTab === 'crates'" label="Crates" sub="This feature is coming soon." />
      <ShopPage v-show="activeTab === 'shop'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import WipPage from '../components/common/WipPage.vue'
import ShopPage from './ShopPage.vue'
import { useSlidingTabIndicator } from '../composables/useSlidingTabIndicator'

const route = useRoute()

const tabs = [
  { key: 'pass' as const,   label: 'Client Pass', wip: true },
  { key: 'crates' as const, label: 'Crates',       wip: true },
  { key: 'shop' as const,   label: 'Shop',          wip: false },
]

const activeTab = ref<'pass' | 'crates' | 'shop'>('shop')
const { setTabBtnRef, indicatorStyle } = useSlidingTabIndicator(activeTab)

onMounted(() => {
  const tab = route.query.tab
  if (tab === 'pass' || tab === 'crates' || tab === 'shop') activeTab.value = tab
})
</script>

<style lang="scss" scoped>
.store-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow: hidden;
}

.store-content {
  flex: 1;
  min-height: 0;
}

// ── Tabs ──────────────────────────────────────────────────────────────────────
.tab-row {
  position: relative;
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.tab-btn {
  position: relative;
  padding: 8px 22px;
  background: #0d0d0d;
  border: none;
  color: #aaa;
  font-family: 'Mojangles', monospace;
  font-size: 13px;
  cursor: pointer;
  letter-spacing: 0.02em;
  transition: background 80ms, color 80ms;
  border-radius: 0;
  display: flex;
  align-items: center;
  gap: 7px;

  &:hover { background: #1a1a1a; color: #ccc; }
  &.active {
    background: #111;
    color: #d9d9d9;
  }
}

.tab-indicator {
  position: absolute;
  bottom: 0;
  height: 2px;
  background: rgba(255, 255, 255, 0.3);
  pointer-events: none;
  transition: left 260ms cubic-bezier(0.16, 1, 0.3, 1), width 260ms cubic-bezier(0.16, 1, 0.3, 1);
}

.tab-wip {
  padding: 0 5px;
  height: 14px;
  background: rgba(224, 53, 53, 0.85);
  color: #fff;
  font-size: 8px;
  font-weight: 700;
  border-radius: 7px;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
