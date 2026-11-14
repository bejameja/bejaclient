<template>
  <Transition name="login-fade">
    <div v-if="show" class="login-screen">
      <!-- The real titlebar controls (in TopBar) are inert behind this overlay - mirror them
           here so the window can still be minimized/maximized/closed while signed out. -->
      <div class="login-win-controls">
        <button class="win-btn" title="Minimize" @click="minimize">
          <svg width="11" height="11" viewBox="0 0 11 11" fill="none">
            <line x1="2" y1="5.5" x2="9" y2="5.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
          </svg>
        </button>
        <button class="win-btn" title="Maximize" @click="toggleMaximize">
          <svg v-if="!maximized" width="11" height="11" viewBox="0 0 11 11" fill="none">
            <rect x="1.9" y="1.9" width="7.2" height="7.2" rx="0.7" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/>
          </svg>
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

      <div class="login-frame">
        <img :src="logoUrl" class="login-logo" alt="BejaClient" />

        <div class="login-version">Version {{ version }}</div>

        <div class="login-left">
          <div class="login-heading">
            <span class="login-heading-line">Log into</span>
            <span class="login-heading-line"><span class="w-beja">Beja</span><span class="w-client">Client</span></span>
          </div>

          <button class="login-btn" :disabled="accountStore.loading" @click="handleLogin">
            <span v-if="accountStore.loading && !accountStore.loginStatus" class="login-spinner" />
            <template v-else>
              <svg class="login-ms-icon" viewBox="0 0 21 21" fill="none">
                <rect x="1"  y="1"  width="9" height="9" fill="#F25022"/>
                <rect x="11" y="1"  width="9" height="9" fill="#7FBA00"/>
                <rect x="1"  y="11" width="9" height="9" fill="#00A4EF"/>
                <rect x="11" y="11" width="9" height="9" fill="#FFB900"/>
              </svg>
              Sign in with Microsoft
            </template>
          </button>

          <div class="login-msg-slot">
            <p v-if="accountStore.loginStatus" class="login-status">{{ accountStore.loginStatus }}</p>
            <p v-if="accountStore.error && !accountStore.loading" class="login-error">{{ loginErrorMessage }}</p>
          </div>
        </div>

        <img :src="skinPose" class="login-skin-pose" alt="" />
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import skinPose from '../../assets/login-skin-pose.png'
import logoUrl from '../../assets/login-logo.png'
import { useAccountStore } from '../../store/accountStore'

const props = defineProps<{ show: boolean }>()
const accountStore = useAccountStore()

const maximized = ref(false)
const version   = ref('')
function minimize()       { window.api.window.minimize() }
function toggleMaximize() { window.api.window.maximize() }
function close()          { window.api.window.close() }

onMounted(async () => {
  maximized.value = await window.api.window.isMaximized()
  window.api.window.onMaximized((v: boolean) => { maximized.value = v })
  version.value = await window.api.system.getVersion()
})

const loginErrorMessage = computed(() => {
  const raw = accountStore.error
  if (!raw) return ''
  const msg = raw.replace(/^Error:\s*/i, '').trim()
  if (/cancel/i.test(msg)) return 'Sign-in cancelled.'
  if (/timed?\s*out/i.test(msg)) return 'Sign-in timed out. Try again.'
  if (/does not own minecraft/i.test(msg)) return 'This account does not own Minecraft Java Edition.'
  if (/no xbox profile/i.test(msg)) return 'No Xbox profile on this account. Set one up at xbox.com.'
  if (/not available in your country/i.test(msg)) return 'Xbox Live is not available in your region.'
  if (/no auth code/i.test(msg)) return 'Authorisation denied. Try again.'
  return msg || 'Sign-in failed. Try again.'
})

async function handleLogin() {
  await accountStore.login()
}
</script>

<style lang="scss" scoped>
.login-screen {
  position: fixed;
  inset: 0;
  // Above every other overlay in the app (CommandPalette 9800, ContextMenu 9500, TopBar's own
  // dropdowns 2000, etc.) - while signed out, nothing behind this should be reachable at all,
  // not even things that render via Teleport straight to <body>.
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  // App behind (Hub page, TopBar) shows through blurred instead of a flat/artwork background -
  // it's already `inert` while this is up, so it's just a backdrop, never interactive.
  background: rgba(12, 13, 18, 0.55);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  overflow: hidden;
}

// A bounded 16:9 frame, not the whole app window - everything below is positioned relative to
// this, not the screen.
.login-frame {
  position: relative;
  z-index: 2;
  width: min(760px, 56vw);
  aspect-ratio: 16 / 9;
  background: #000;
  border-radius: 0;
  overflow: hidden;
}

.login-logo {
  position: absolute;
  top: 14px;
  left: 14px;
  z-index: 3;
  height: 26px;
  width: auto;
  animation: login-logo-spin 4s linear infinite;
}

@keyframes login-logo-spin {
  to { transform: rotate(360deg); }
}

.login-version {
  position: absolute;
  bottom: 14px;
  left: 14px;
  z-index: 3;
  font-size: 10px;
  color: #fff;
}

.login-win-controls {
  position: absolute;
  top: 0;
  right: 0;
  z-index: 3;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 10px;
  -webkit-app-region: no-drag;
}

.win-btn {
  width: 30px;
  height: 30px;
  border-radius: $radius-sm;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  color: rgba(255, 255, 255, 0.7);
  transition: color 150ms ease, background 150ms ease;

  &:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.08);
  }

  svg { transform: scale(1.3); }
}

.win-close:hover {
  color: #fff;
  background: #c0413f;
}

// ── Left column: heading, sign-in button, status/error ───────────────────────
.login-left {
  position: absolute;
  left: 6%;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 20px;
  width: 42%;
  min-width: 320px;
  z-index: 1;
}

.login-heading {
  display: flex;
  flex-direction: column;
  font-size: 42px;
  line-height: 1.2;
  color: $text-primary;
}

.w-beja   { font-weight: 900; }
.w-client { font-weight: 300; }

// ── Skin pose, right side ────────────────────────────────────────────────────
.login-skin-pose {
  position: absolute;
  right: -14%;
  top: 52%;
  transform: translateY(-50%);
  height: 118%;
  width: auto;
  object-fit: contain;
  filter: brightness(0.85);
}

// ── Sign-in button, compact rounded pill ─────────────────────────────────────
.login-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 14px 44px;
  background: #3A5F9F;
  border: 1px solid transparent;
  border-radius: 0;
  color: #fff;
  font-size: 15px;
  font-weight: 600;
  white-space: nowrap;
  cursor: pointer;
  transition: border-color 120ms, background 120ms;

  &:hover:not(:disabled) {
    background: lighten(#3A5F9F, 6%);
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

.login-ms-icon {
  width: 17px;
  height: 17px;
  flex-shrink: 0;
}

.login-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255,255,255,0.35);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

.login-msg-slot {
  width: 100%;
}

.login-status {
  font-size: 12px;
  color: $accent;
}

.login-error {
  font-size: 12px;
  color: $error;
}

.login-fade-enter-active { transition: opacity 0.35s ease; }
.login-fade-leave-active {
  transition: opacity 0.45s ease;
  // On successful login, the frame collapses vertically from top+bottom toward the center
  // (not just fades) - the backdrop blur still just fades via the opacity transition above.
  .login-frame { animation: login-frame-collapse 0.45s cubic-bezier(0.6, 0, 0.8, 0.2) forwards; }

  html.reduce-motion & .login-frame { animation: none; }
}
.login-fade-enter-from, .login-fade-leave-to { opacity: 0; }

@keyframes login-frame-collapse {
  to { transform: scaleY(0); }
}
</style>
