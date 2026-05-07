<template>
  <aside class="sidebar">

    <!-- Logo -->
    <div class="sidebar-logo">
      <img :src="logoUrl" class="logo-img" alt="BC" />
    </div>

    <div class="sidebar-divider" />

    <!-- Nav items -->
    <nav class="sidebar-nav">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ active: isActive(item.path, item.exact) }"
        @mouseenter="playHover"
      >
        <img class="nav-icon-img" :src="item.icon" :alt="item.label" />
        <span class="nav-label">{{ item.label }}</span>
        <span v-if="item.badge" class="nav-badge">{{ item.badge }}</span>
      </RouterLink>
    </nav>

    <!-- Drag spacer -->
    <div class="sidebar-spacer" />

    <!-- Footer links -->
    <div class="sidebar-footer">
      <button class="footer-link" @click="openExternal('https://bejaclient.com/status')">Status</button>
      <button class="footer-link" @click="openExternal('https://bejaclient.com/rules')">Rules</button>
      <button class="footer-link" @click="openExternal('https://bejaclient.com/terms')">Terms</button>
      <button class="footer-link" @click="openExternal('https://bejaclient.com/privacy')">Privacy</button>
    </div>

  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import logoUrl from '../../../assets/logo.png'
import iconHome from '../../../assets/icon-home.png'
import iconNews from '../../../assets/icon-news.png'
import iconInstances from '../../../assets/icon-instances.png'
import iconMods from '../../../assets/icon-mods.png'
import iconSettings from '../../../assets/icon-settings.png'
import { playHover } from '../../composables/useSounds'

const route = useRoute()

const ICONS = {
  play: iconHome,
  news: iconNews,
  shop: iconHome,
  instances: iconInstances,
  mods: iconMods,
  settings: iconSettings,
}

const navItems = computed(() => [
  { label: 'Play',      path: '/',          exact: true,  icon: ICONS.play,      badge: null },
  { label: 'News',      path: '/versions',  exact: false, icon: ICONS.news,      badge: null },
  { label: 'Shop',      path: '/cosmetics', exact: false, icon: ICONS.shop,      badge: null },
  { label: 'Instances', path: '/profiles',  exact: false, icon: ICONS.instances, badge: null },
  { label: 'Mods',      path: '/mods',      exact: false, icon: ICONS.mods,      badge: null },
  { label: 'Settings',  path: '/settings',  exact: false, icon: ICONS.settings,  badge: null },
])

function isActive(path: string, exact: boolean) {
  return exact ? route.path === path : route.path.startsWith(path)
}

function openExternal(url: string) {
  window.open(url, '_blank')
}
</script>

<style lang="scss" scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  width: $sidebar-width;
  height: 100vh;
  background: $surface-panel;
  flex-shrink: 0;
  z-index: 100;
  -webkit-app-region: drag;
}

// ── Logo ──────────────────────────────────────────────────────────────────────
.sidebar-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 28px 0 22px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.logo-img {
  width: 72px;
  height: 72px;
  object-fit: contain;
}

// ── Divider ───────────────────────────────────────────────────────────────────
.sidebar-divider {
  height: 1px;
  background: $border;
  margin: 0 20px;
  flex-shrink: 0;
}

// ── Nav items ─────────────────────────────────────────────────────────────────
.sidebar-nav {
  display: flex;
  flex-direction: column;
  padding: 16px 0;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.nav-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 24px;
  text-decoration: none;
  cursor: pointer;
  user-select: none;
  color: $text-secondary;
  transition: color $transition, background $transition;
  border-left: 2px solid transparent;

  &:hover {
    color: $text-primary;
    background: rgba(255, 255, 255, 0.04);
  }

  &.active {
    color: $text-primary;
    border-left-color: var(--accent, #{$accent});
    background: rgba(255, 255, 255, 0.05);
  }
}

.nav-icon-img {
  width: 22px;
  height: 22px;
  flex-shrink: 0;
  object-fit: contain;
  opacity: 0.6;
  transition: opacity $transition;
}

.nav-item:hover .nav-icon-img {
  opacity: 0.85;
}

.nav-item.active .nav-icon-img {
  opacity: 1;
}

.nav-label {
  font-size: 15px;
  font-weight: 600;
  letter-spacing: 0.01em;
  line-height: normal;
}

.nav-badge {
  position: absolute;
  top: 8px;
  right: 14px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  background: $accent;
  color: #fff;
  font-size: 9px;
  font-weight: 700;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

// ── Spacer (drag region) ──────────────────────────────────────────────────────
.sidebar-spacer {
  flex: 1;
  -webkit-app-region: drag;
}

// ── Footer links ──────────────────────────────────────────────────────────────
.sidebar-footer {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 16px 24px 24px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.footer-link {
  background: none;
  border: none;
  padding: 3px 0;
  cursor: pointer;
  text-align: left;
  font-size: 12px;
  color: $text-muted;
  transition: color $transition;

  &:hover {
    color: $text-secondary;
  }
}
</style>
