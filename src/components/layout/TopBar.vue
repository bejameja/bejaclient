<template>
  <header class="topbar">
    <div class="brand-group">
      <img :src="logoUrl" class="brand-logo" alt="BC" />
      <span class="wordmark">
        <span class="w-beja">Beja</span><span class="w-client">Client</span> <span class="w-launcher">Launcher</span>
      </span>
    </div>

    <div class="nav-center">
      <nav class="topnav">
        <RouterLink
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="nav-item"
          :class="{ active: isActive(item.path, item.exact) }"
        >
          <span class="nav-label">{{ item.label }}</span>
          <span v-if="item.badge" class="nav-badge">{{ item.badge }}</span>
          <span v-if="item.wip" class="nav-wip">
            <svg class="wip-clock" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
              <circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.4"/>
              <line class="clock-min" x1="7" y1="7" x2="10.5" y2="7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
              <line x1="7" y1="7" x2="7" y2="4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
            </svg>
          </span>
        </RouterLink>
      </nav>
    </div>

    <div class="right-group">
      <!-- Currency -->
      <Tooltip v-if="account" text="Violet Gems" placement="bottom">
        <div class="gem-pill">
          <img class="gem-pill-icon" :src="gemIcon" alt="" />
          <span>{{ gemDisplay.toLocaleString() }}</span>
        </div>
      </Tooltip>

      <!-- Account -->
      <template v-if="account">
        <div ref="pillRef" class="account-pill" @click="dropdownOpen = !dropdownOpen">
          <img
            class="account-head"
            :src="`https://mc-heads.net/avatar/${account.uuid}/22`"
            :alt="account.username"
            @error="(e) => ((e.target as HTMLImageElement).style.display = 'none')"
          />
          <span class="account-name">{{ account.username }}</span>
          <svg class="pill-chevron" :class="{ open: dropdownOpen }" width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>

        <!-- Dropdown — teleported to body to escape all stacking contexts -->
        <Teleport to="body">
          <Transition name="dd">
            <div v-if="dropdownOpen" ref="dropdownRef" class="acc-dropdown" :style="dropdownStyle" @click.stop>
              <div class="dd-label">Accounts</div>

              <div
                v-for="acc in accountStore.accounts"
                :key="acc.id"
                class="dd-item-row"
              >
                <button
                  class="dd-item"
                  :class="{ active: acc.selected }"
                  @click="switchAccount(acc.id)"
                >
                  <img class="dd-head" :src="`https://mc-heads.net/avatar/${acc.uuid}/18`" :alt="acc.username"
                    @error="(e) => ((e.target as HTMLImageElement).style.display = 'none')" />
                  <span class="dd-name">{{ acc.username }}</span>
                  <svg v-if="acc.selected" width="10" height="10" viewBox="0 0 10 10" fill="none">
                    <path d="M1.5 5L4 7.5L8.5 2.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </button>
                <button class="dd-signout" title="Sign out" @click.stop="signOut(acc.id)">
                  <svg width="13" height="13" viewBox="0 0 13 13" fill="none">
                    <path d="M5 2H2.5A.5.5 0 002 2.5v8a.5.5 0 00.5.5H5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
                    <path d="M8.5 9L11 6.5L8.5 4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
                    <line x1="11" y1="6.5" x2="5" y2="6.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
                  </svg>
                </button>
              </div>

              <div class="dd-sep" />

              <button class="dd-item dd-add" :disabled="accountStore.loading" @click="addAccount">
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                  <line x1="7" y1="2" x2="7" y2="12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                  <line x1="2" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                </svg>
                <span>Add Account</span>
              </button>
            </div>
          </Transition>
        </Teleport>
      </template>
      <button
        v-else
        class="account-pill account-pill--guest"
        :disabled="accountStore.loading"
        @click="accountStore.login()"
      >
        <svg width="13" height="13" viewBox="0 0 13 13" fill="none">
          <circle cx="6.5" cy="4.5" r="2.5" stroke="currentColor" stroke-width="1.3"/>
          <path d="M1.5 11.5c0-2.485 2.239-4.5 5-4.5s5 2.015 5 4.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
        </svg>
        <span class="account-name account-name--guest">{{ accountStore.loginStatus ?? (accountStore.loading ? 'Signing in…' : 'Sign in') }}</span>
      </button>

      <div class="win-controls">
        <button class="win-btn" title="Minimize" @click="minimize">
          <svg width="11" height="11" viewBox="0 0 11 11" fill="none">
            <line x1="2" y1="5.5" x2="9" y2="5.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
          </svg>
        </button>
        <button class="win-btn" title="Maximize" @click="toggleMaximize">
          <svg v-if="!maximized" width="11" height="11" viewBox="0 0 11 11" fill="none">
            <rect x="1.9" y="1.9" width="7.2" height="7.2" rx="0.7" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/>
          </svg>
          <!-- Restore icon: two custom inward-pointing corner brackets, not the OS double-square -->
          <svg v-else width="11" height="11" viewBox="0 0 11 11" fill="none">
            <path d="M1.5 4.5V1.5H4.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M9.5 6.5V9.5H6.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
        <button class="win-btn win-close" title="Close" @click="close">
          <svg width="11" height="11" viewBox="0 0 11 11" fill="none">
            <line x1="2" y1="2" x2="9" y2="9" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
            <line x1="9" y1="2" x2="2" y2="9" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAccountStore } from '../../store/accountStore'
import { useWalletStore } from '../../store/walletStore'
import { useFriendsStore } from '../../store/friendsStore'
import { useCountUp } from '../../composables/useCountUp'
import Tooltip from '../common/Tooltip.vue'
import gemIcon from '../../assets/violet-gem-pixel.svg'
import logoUrl from '../../assets/bc-logo-new.png'

import iconPlay      from '../../assets/icons8-spielen-64.png'
import iconLocker    from '../../assets/icons8-kimono-64.png'
import iconExplore   from '../../assets/icons8-kompass-48.png'
import iconProfiles  from '../../assets/icons8-name-50.png'
import iconFriends   from '../../assets/icons8-freunde-64.png'
import iconSettings  from '../../assets/icons8-settings-50.png'

const accountStore = useAccountStore()
const walletStore  = useWalletStore()
const friendsStore = useFriendsStore()
const account      = computed(() => accountStore.selectedAccount)
const gemDisplay   = useCountUp(computed(() => walletStore.balance))

// ── Nav (formerly the left sidebar, now folded into the top bar) ────────────
const route = useRoute()
const { t } = useI18n()

interface NavItem {
  label: string
  path: string
  exact: boolean
  icon: string
  badge: number | null
  wip?: boolean
}

const navItems = computed<NavItem[]>(() => [
  { label: t('nav.hub'),       path: '/',          exact: true,  icon: iconPlay,     badge: null },
  { label: t('nav.locker'),    path: '/cosmetics', exact: false, icon: iconLocker,   badge: null },
  { label: t('nav.explore'),   path: '/mods',      exact: false, icon: iconExplore,  badge: null },
  { label: t('nav.profiles'),  path: '/profiles',  exact: false, icon: iconProfiles, badge: null },
  { label: t('nav.friends'),   path: '/friends',   exact: false, icon: iconFriends,  badge: friendsStore.pendingCount || null },
  { label: t('nav.settings'),  path: '/settings',  exact: false, icon: iconSettings, badge: null },
])

function isActive(path: string, exact: boolean) {
  return exact ? route.path === path : route.path.startsWith(path)
}

const maximized    = ref(false)
const dropdownOpen = ref(false)
const pillRef      = ref<HTMLElement | null>(null)
const dropdownRef  = ref<HTMLElement | null>(null)

const dropdownStyle = computed(() => {
  if (!pillRef.value) return {}
  const rect = pillRef.value.getBoundingClientRect()
  return {
    top: `${rect.bottom + 6}px`,
    right: `${window.innerWidth - rect.right}px`,
  }
})

function minimize()       { window.api.window.minimize() }
function toggleMaximize() { window.api.window.maximize() }
function close()          { window.api.window.close() }

async function switchAccount(id: string) {
  await accountStore.selectAccount(id)
  dropdownOpen.value = false
}

async function addAccount() {
  dropdownOpen.value = false
  await accountStore.login()
}

async function signOut(id: string) {
  await accountStore.logout(id)
  if (!accountStore.hasAccounts) dropdownOpen.value = false
}

function onClickOutside(e: MouseEvent) {
  const t = e.target as Node
  if (
    pillRef.value && !pillRef.value.contains(t) &&
    dropdownRef.value && !dropdownRef.value.contains(t)
  ) {
    dropdownOpen.value = false
  }
}

onMounted(async () => {
  maximized.value = await window.api.window.isMaximized()
  window.api.window.onMaximized((v: boolean) => { maximized.value = v })
  document.addEventListener('mousedown', onClickOutside, true)
})

// The topbar can mount before login finishes, so re-load whenever an account becomes active.
watch(account, acc => { if (acc) walletStore.load() }, { immediate: true })

onUnmounted(() => {
  document.removeEventListener('mousedown', onClickOutside, true)
})
</script>

<style lang="scss" scoped>
.topbar {
  flex-shrink: 0;
  display: grid;
  // Right column is a FIXED width (not auto) so gem/account/win-controls content
  // changes never reflow the 1fr nav column — see .right-group/.account-pill below.
  grid-template-columns: auto 1fr 320px;
  align-items: center;
  padding: 8px 12px 8px 14px;
  -webkit-app-region: drag;
}

// Middle grid column — centers .topnav within the space between the brand
// and the right-hand controls, regardless of how wide either side is.
.nav-center {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  min-width: 0;
  // Recenters against the visible end of the "BejaClientLauncher" text rather than
  // the grid column boundary, which includes .brand-group's 34px margin-right.
  margin-left: -17px;
}

.right-group {
  display: flex;
  align-items: center;
  // Stretches to fill the fixed 420px column so .gem-pill (flex-start, first
  // child) always sits at the same spot; .account-pill's margin-left: auto
  // pushes the account+win-controls cluster flush right within this box.
  justify-self: stretch;
  flex-shrink: 0;
}

// ── Brand ─────────────────────────────────────────────────────────────────────
.brand-group {
  display: flex;
  align-items: center;
  gap: 9px;
  flex-shrink: 0;
  margin-right: 34px;
  animation: topbar-drop-in 700ms cubic-bezier(0.16, 1, 0.3, 1) 100ms both;
}

.brand-logo {
  width: 26px;
  height: 26px;
  object-fit: contain;
  filter: brightness(0) invert(1);
  opacity: 0.9;
  flex-shrink: 0;
}

.wordmark {
  display: flex;
  align-items: baseline;
  font-size: 14px;
  font-family: 'Plus Jakarta Sans', sans-serif;
  line-height: 1;
  white-space: nowrap;
}

.w-beja     { font-weight: 900; color: #fff; }
.w-client   { font-weight: 300; color: #fff; }
.w-launcher { font-weight: 800; color: #fff; }

// ── Top nav (formerly the left sidebar) ──────────────────────────────────────
.topnav {
  position: relative;
  display: flex;
  align-items: center;
  gap: 6px;
  height: 100%;
  min-width: 0;
  flex-shrink: 1;
  overflow: visible;
  -webkit-app-region: no-drag;
}

.nav-item {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  height: 100%;
  padding: 0 26px;
  text-decoration: none;
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.nav-item:hover:not(.active) .nav-label {
  color: rgba(255, 255, 255, 0.45);
}

.nav-label {
  position: relative;
  z-index: 1;
  display: inline-block;
  font-size: 14px;
  line-height: 1;
  font-weight: 500;
  transition: color 200ms ease;
  color: rgba(255, 255, 255, 0.22);
}

.nav-item.active .nav-label {
  color: #fff;
}

.nav-badge {
  position: absolute;
  top: 10px;
  right: 8px;
  z-index: 2;
  min-width: 14px;
  height: 14px;
  padding: 0 3px;
  background: #e0353c;
  color: #fff;
  font-size: 8px;
  font-weight: 700;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.nav-wip {
  position: relative;
  z-index: 2;
  width: 15px;
  height: 15px;
  background: rgba(18, 20, 24, 0.85);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  flex-shrink: 0;
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

// ── Currency pill ────────────────────────────────────────────────────────────
.gem-pill {
  -webkit-app-region: no-drag;
  display: flex;
  align-items: center;
  gap: 3px;
  height: 30px;
  padding: 0 12px;
  margin-right: 8px;
  background: none;
  color: #fff;
  font-size: 12px;
  line-height: 1;
  flex-shrink: 0;
  animation: topbar-drop-in 700ms cubic-bezier(0.16, 1, 0.3, 1) 150ms both;
}

.gem-pill-icon {
  width: 15px;
  height: 15px;
  flex-shrink: 0;
  image-rendering: pixelated;
}

// ── Account pill ──────────────────────────────────────────────────────────────
.account-pill {
  position: relative;
  margin-left: -14px;
  display: flex;
  align-items: center;
  gap: 8px;
  height: 30px;
  padding: 0 10px 0 6px;
  margin-right: 4px;
  background: none;
  cursor: pointer;
  min-width: 0;
  flex-shrink: 1;
  -webkit-app-region: no-drag;
  color: rgba(255, 255, 255, 0.7);
  animation: topbar-drop-in 700ms cubic-bezier(0.16, 1, 0.3, 1) 150ms both;
  transition: opacity 160ms ease;

  &:hover { opacity: 0.75; }
}

.account-pill--guest {
  color: #e05555;
  &:hover:not(:disabled) { opacity: 0.75; }
  &:disabled { opacity: 0.4; cursor: not-allowed; }
}

.account-head {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  image-rendering: pixelated;
  flex-shrink: 0;
}

.account-name {
  font-size: 14px;
  font-weight: 300;
  color: rgba(255, 255, 255, 0.9);
  min-width: 0;
  max-width: 400px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  &--guest {
    color: #e05555;
    font-weight: 500;
  }
}

.pill-chevron {
  margin-left: -4px;
  flex-shrink: 0;
  transition: transform 150ms ease;
  &.open { transform: rotate(180deg); }
}

.acc-dropdown {
  position: fixed;
  min-width: 200px;
  background: rgba(13, 13, 13, 0.9);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  border: 1px solid rgba(255, 255, 255, 0.18);
  overflow: hidden;
  z-index: 2000;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
}

.dd-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: rgba(255, 255, 255, 0.35);
  padding: 10px 12px 6px;
}

.dd-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.7);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  text-align: left;
  transition: background 100ms, color 100ms;

  &:hover:not(:disabled) { background: #1a1a1a; color: #fff; }
  &.active { color: #fff; font-weight: 600; }
  &:disabled { opacity: 0.4; cursor: not-allowed; }
}

.dd-head {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  image-rendering: pixelated;
  flex-shrink: 0;
}

.dd-name { flex: 1; }

.dd-sep {
  height: 1px;
  background: rgba(255, 255, 255, 0.12);
  margin: 4px 0;
}

.dd-add { color: #6b9bff; }

.dd-item-row {
  display: flex;
  align-items: center;
  .dd-item { flex: 1; }
}

.dd-signout {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  margin-right: 4px;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  border-radius: 4px;
  opacity: 0;
  transition: opacity 100ms, background 100ms, color 100ms;

  .dd-item-row:hover & { opacity: 1; }
  &:hover { background: rgba(255, 80, 80, 0.15); color: #ff6060; }
}

.dd-enter-active, .dd-leave-active { transition: opacity 120ms ease, transform 120ms ease; }
.dd-enter-from, .dd-leave-to { opacity: 0; transform: translateY(-6px); }

.win-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: auto;
  animation: topbar-drop-in 700ms cubic-bezier(0.16, 1, 0.3, 1) 150ms both;
  -webkit-app-region: no-drag;
  flex-shrink: 0;
}

.win-btn {
  width: 30px;
  height: 30px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  color: rgba(255, 255, 255, 0.55);
  transition: color 150ms ease;

  &:hover {
    color: #ffffff;
  }

  svg {
    transform: scale(1.3);
  }
}

.win-close:hover {
  color: #b56b6b;
}

@keyframes topbar-drop-in {
  from { opacity: 0; transform: translateY(-18px); }
  to   { opacity: 1; transform: translateY(0); }
}
</style>
