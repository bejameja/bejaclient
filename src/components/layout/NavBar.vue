<template>
  <aside class="sidebar">

    <div class="sidebar-divider" />

    <!-- Nav items -->
    <nav ref="navListEl" class="sidebar-nav">
      <div class="active-rail" :style="railStyle" />
      <Tooltip
        v-for="item in navItems"
        :key="item.path"
        :text="item.label"
        placement="right"
        :delay="250"
      >
        <RouterLink
          :ref="(el) => setItemRef(item.path, el)"
          :to="item.path"
          class="nav-item"
          :class="{ active: isActive(item.path, item.exact) }"
          :style="{ '--icon-size': (item.iconSize ?? 30) + 'px' }"
          @mouseenter="playHover"
        >
          <span class="nav-icon-wrap">
            <img class="nav-icon-img" :src="item.icon" :alt="item.label" />
          </span>
          <span v-if="item.badge" class="nav-badge">{{ item.badge }}</span>
          <span v-if="item.wip" class="nav-wip">
            <svg class="wip-clock" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
              <circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.4"/>
              <line class="clock-min" x1="7" y1="7" x2="10.5" y2="7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
              <line x1="7" y1="7" x2="7" y2="4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
            </svg>
          </span>
        </RouterLink>
      </Tooltip>
    </nav>

    <!-- Drag spacer -->
    <div class="sidebar-spacer" />

    <div class="sidebar-divider" />

    <!-- Footer links -->
    <div class="sidebar-footer">
      <button class="footer-link" @click="openExternal('https://bejaclient.com/terms')">{{ $t('nav.footer.terms') }}</button>
      <button class="footer-link" @click="openExternal('https://bejaclient.com/privacy')">{{ $t('nav.footer.privacy') }}</button>
      <button class="footer-link" @click="openExternal('https://bejaclient.com/support')">{{ $t('nav.footer.support') }}</button>
    </div>

  </aside>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useLauncherStore } from '../../store/launcherStore'
import { useFriendsStore }  from '../../store/friendsStore'
import { playHover }        from '../../composables/useSounds'
import Tooltip from '../common/Tooltip.vue'

import iconPlay      from '../../assets/icons8-spielen-64.png'
import iconLocker    from '../../assets/icons8-kimono-64.png'
import iconExplore   from '../../assets/icons8-kompass-48.png'
import iconProfiles  from '../../assets/icons8-name-50.png'
import iconFriends   from '../../assets/icons8-freunde-64.png'
import iconQuests    from '../../assets/icons8-aufgabe-50.png'
import iconStore     from '../../assets/icons8-package-50.png'
import iconSettings  from '../../assets/icons8-settings-50.png'

const route         = useRoute()
const launcherStore = useLauncherStore()
const friendsStore  = useFriendsStore()
const { t } = useI18n()

interface NavItem {
  label: string
  path: string
  exact: boolean
  icon: string
  badge: number | null
  iconSize: number
  wip?: boolean
}

const navItems = computed<NavItem[]>(() => [
  { label: t('nav.hub'),        path: '/',          exact: true,  icon: iconPlay,     badge: null, iconSize: 30 },
  { label: t('nav.locker'),     path: '/cosmetics', exact: false, icon: iconLocker,   badge: null, iconSize: 38 },
  { label: t('nav.explore'),    path: '/mods',      exact: false, icon: iconExplore,  badge: null, iconSize: 30 },
  { label: t('nav.profiles'),   path: '/profiles',  exact: false, icon: iconProfiles, badge: null, iconSize: 30 },
  { label: t('nav.friends'),    path: '/friends',   exact: false, icon: iconFriends,  badge: friendsStore.pendingCount || null, iconSize: 34 },
  { label: t('nav.store'),      path: '/store',     exact: false, icon: iconStore,    badge: null, iconSize: 30 },
  { label: t('nav.quests'),    path: '/quests',    exact: false, icon: iconQuests,   badge: null, iconSize: 30 },
  { label: t('nav.settings'),   path: '/settings',  exact: false, icon: iconSettings, badge: null, iconSize: 30 },
])

const navListEl = ref<HTMLElement | null>(null)
const itemEls = new Map<string, HTMLElement>()
const railStyle = ref({ transform: 'translateY(0px)', height: '0px', opacity: '0' })

function setItemRef(path: string, el: unknown) {
  const domEl = (el && typeof el === 'object' && '$el' in el)
    ? (el as { $el: HTMLElement }).$el
    : (el as HTMLElement | null)
  if (domEl) itemEls.set(path, domEl)
}

function updateRail() {
  const active = navItems.value.find(i => isActive(i.path, i.exact))
  const container = navListEl.value
  const el = active ? itemEls.get(active.path) : null
  if (!active || !el || !container) {
    railStyle.value = { ...railStyle.value, opacity: '0' }
    return
  }
  const top = el.offsetTop
  railStyle.value = {
    transform: `translateY(${top}px)`,
    height: `${el.offsetHeight}px`,
    opacity: '1',
  }
}

watch(() => route.path, () => nextTick(updateRail))
onMounted(() => nextTick(updateRail))

function isActive(path: string, exact: boolean) {
  return exact ? route.path === path : route.path.startsWith(path)
}

function openExternal(url: string) {
  window.open(url, '_blank')
}
</script>

<style lang="scss" scoped>
// Cropped to start near the skin preview embed level instead of spanning the
// full window height — pulled up a bit tighter than the exact preview-top
// alignment to leave less empty space above the bar.
.sidebar {
  display: flex;
  flex-direction: column;
  width: 88px;
  margin-top: 44px;
  height: calc(100vh - 44px);
  background: linear-gradient(180deg, rgba(0, 0, 0, 0.5), rgba(0, 0, 0, 0.5));
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px 12px 0 0;
  flex-shrink: 0;
  z-index: 100;
  overflow: hidden;
  position: relative;
  -webkit-app-region: drag;
  animation: nav-slide-in 900ms cubic-bezier(0.16, 1, 0.3, 1) both;
}

@keyframes nav-slide-in {
  from { opacity: 0; transform: translateX(-40px); filter: blur(4px); }
  to   { opacity: 1; transform: translateX(0); filter: blur(0); }
}

@keyframes nav-item-pop {
  from { opacity: 0; transform: translateY(10px) scale(0.85); }
  to   { opacity: 1; transform: translateY(0) scale(1); }
}

// ── Right-border sweep ────────────────────────────────────────────────────────
.sidebar::after {
  content: '';
  position: absolute;
  right: 0;
  top: 0;
  width: 1px;
  height: 240px;
  background: linear-gradient(180deg,
    transparent 0%,
    rgba(255, 255, 255, 0.55) 40%,
    rgba(255, 255, 255, 0.9) 50%,
    rgba(255, 255, 255, 0.55) 60%,
    transparent 100%
  );
  pointer-events: none;
  z-index: 101;
  will-change: transform;
  animation: border-sweep 20s linear infinite;
}

@keyframes border-sweep {
  0%  { transform: translateY(calc(100vh - 44px)); opacity: 0; animation-timing-function: cubic-bezier(0.4, 0, 0.15, 1); }
  1%  { opacity: 1; }
  47% { opacity: 1; }
  49% { transform: translateY(-240px); opacity: 0; }
  50% { transform: translateY(-240px); opacity: 0; animation-timing-function: cubic-bezier(0.85, 0, 0.55, 1); }
  51% { opacity: 1; }
  98% { opacity: 1; }
  99% { transform: translateY(calc(100vh - 44px)); opacity: 0; }
  100% { transform: translateY(calc(100vh - 44px)); opacity: 0; }
}

// ── Divider ───────────────────────────────────────────────────────────────────
.sidebar-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.07);
  margin: 0 16px;
  flex-shrink: 0;
}

// ── Nav items ─────────────────────────────────────────────────────────────────
.sidebar-nav {
  position: relative;
  display: flex;
  flex-direction: column;
  padding: 12px 0;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

// ── Sliding active indicator ─────────────────────────────────────────────────
.active-rail {
  position: absolute;
  left: 0;
  top: 0;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background: var(--accent, #{$primary});
  box-shadow: 0 0 4px var(--accent, #{$primary}), 0 0 14px color-mix(in srgb, var(--accent, #{$primary}) 55%, transparent);
  transition: transform 320ms $ease-out, height 320ms $ease-out, opacity 200ms ease;
  pointer-events: none;
  z-index: 2;
}

.nav-item {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px 0;
  text-decoration: none;
  cursor: pointer;
  user-select: none;
  transition: background $transition-fast;
  opacity: 0;
  animation: nav-item-pop 520ms cubic-bezier(0.34, 1.56, 0.64, 1) 480ms both;

  &::before {
    content: '';
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%) scale(0.9);
    width: 50px;
    height: 50px;
    border-radius: 11px;
    padding: 2px;
    background: linear-gradient(160deg, #354B74, #1e3050, #111B29);
    background-size: 200% 200%;
    background-position: 0% 0%;
    -webkit-mask:
      linear-gradient(#fff 0 0) content-box,
      linear-gradient(#fff 0 0);
    -webkit-mask-composite: xor;
    mask-composite: exclude;
    opacity: 0;
    transition: opacity 200ms $ease-out, transform 200ms $ease-out;
    animation: none;
    pointer-events: none;
  }

  &:hover {
    background: transparent;

    &::before {
      opacity: 0.5;
      transform: translate(-50%, -50%) scale(1);
    }
  }

  &.active {
    background: transparent;

    &::before {
      opacity: 1;
      transform: translate(-50%, -50%) scale(1);
      animation: gradient-shift 4s ease infinite;
    }
  }
}

@for $i from 1 through 8 {
  .sidebar-nav > span.tooltip-trigger:nth-of-type(#{$i}) .nav-item {
    animation-delay: #{480ms + ($i - 1) * 80ms};
  }
}

.nav-icon-wrap {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--icon-size, 30px);
  height: var(--icon-size, 30px);
  flex-shrink: 0;
}

.nav-icon-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  filter: brightness(0.55);
  transition: transform 80ms ease-in, filter 150ms ease;
}

.nav-item.active .nav-icon-img {
  filter: brightness(0) invert(1);
}

.nav-item:hover .nav-icon-img {
  transform: scale(1.14) translateZ(0);
  // enter: smooth ease-out, no overshoot
  transition: transform 160ms cubic-bezier(0.2, 0, 0, 1);
}

.nav-badge {
  position: absolute;
  top: 9px;
  right: 12px;
  min-width: 14px;
  height: 14px;
  padding: 0 3px;
  background: var(--accent, #3eb8ff);
  color: #fff;
  font-size: 8px;
  font-weight: 700;
  border-radius: 7px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-wip {
  position: absolute;
  top: 8px;
  right: 11px;
  width: 15px;
  height: 15px;
  background: rgba(18, 20, 24, 0.85);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  z-index: 2;
}

.wip-clock {
  width: 11px;
  height: 11px;
  color: #e03535;
}

.clock-min {
  transform-origin: 7px 7px;
  animation: clock-tick 4s linear infinite;
}

@keyframes clock-tick {
  0%   { transform: rotate(0deg);   animation-timing-function: cubic-bezier(0.4, 0, 0.2, 1); }
  20%  { transform: rotate(90deg); }
  25%  { transform: rotate(90deg);  animation-timing-function: cubic-bezier(0.4, 0, 0.2, 1); }
  45%  { transform: rotate(180deg); }
  50%  { transform: rotate(180deg); animation-timing-function: cubic-bezier(0.4, 0, 0.2, 1); }
  70%  { transform: rotate(270deg); }
  75%  { transform: rotate(270deg); animation-timing-function: cubic-bezier(0.4, 0, 0.2, 1); }
  95%  { transform: rotate(360deg); }
  100% { transform: rotate(360deg); }
}

// ── Spacer ────────────────────────────────────────────────────────────────────
.sidebar-spacer {
  flex: 1;
  -webkit-app-region: drag;
}

@keyframes gradient-shift {
  0%   { background-position: 0% 0%; }
  50%  { background-position: 100% 100%; }
  100% { background-position: 0% 0%; }
}

// ── Footer links ──────────────────────────────────────────────────────────────
.sidebar-footer {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 12px 0 20px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
  animation: hub-fade-in 700ms ease 1300ms both;
}

@keyframes hub-fade-in {
  from { opacity: 0; }
  to   { opacity: 1; }
}

.footer-link {
  background: none;
  border: none;
  padding: 3px 0;
  cursor: pointer;
  font-size: 10px;
  letter-spacing: 0.01em;
  color: rgba(255, 255, 255, 0.25);
  transition: color 200ms $ease-out;
  font-family: 'Plus Jakarta Sans', sans-serif;

  &:hover { color: rgba(255, 255, 255, 0.6); }
}
</style>
