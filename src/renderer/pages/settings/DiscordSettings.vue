<template>
  <div class="discord-settings-container">
    <!-- Left Column: Inputs -->
    <div class="settings-form">
      <!-- Tabs for Settings Sections -->
      <div class="rpc-tabs">
        <button
          class="rpc-tab-btn"
          :class="{ active: activeTab === 'general' }"
          @click="activeTab = 'general'"
        >
          General
        </button>
        <button
          class="rpc-tab-btn"
          :class="{ active: activeTab === 'idle' }"
          @click="activeTab = 'idle'"
        >
          Idle (Launcher)
        </button>
        <button
          class="rpc-tab-btn"
          :class="{ active: activeTab === 'playing' }"
          @click="activeTab = 'playing'"
        >
          Playing (Game)
        </button>
      </div>

      <!-- General Tab Content -->
      <div v-show="activeTab === 'general'" class="tab-content">
        <div class="setting-group">
          <!-- Toggle -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Enable Discord RPC</span>
              <span class="setting-desc">Show your launcher activity and game status on Discord</span>
            </div>
            <div class="setting-control">
              <div
                class="toggle"
                :class="{ 'toggle--on': s.discord.enabled }"
                @click="s.discord.enabled = !s.discord.enabled; save()"
              />
            </div>
          </div>
          
          <!-- Client ID -->
          <div class="setting-row setting-row--tall" :class="{ 'setting-row--muted': !s.discord.enabled }">
            <div class="setting-info">
              <span class="setting-label">Custom Discord Client ID</span>
              <span class="setting-desc">
                Override Client ID to change the application name (e.g. "Playing [Your Name]"). Leave blank to use default "BejaClient".
              </span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. 1491721511168639016"
                v-model="s.discord.clientId"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Idle Tab Content -->
      <div v-show="activeTab === 'idle'" class="tab-content" :class="{ 'tab-content--disabled': !s.discord.enabled }">
        <div class="setting-group">
          <!-- Details -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Details</span>
              <span class="setting-desc">First line of status text</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. Browsing the launcher"
                v-model="s.discord.idleDetails"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
          <!-- State -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">State</span>
              <span class="setting-desc">Second line of status text</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. Idle"
                v-model="s.discord.idleState"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
          <!-- Large Image Key -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Large Image Key</span>
              <span class="setting-desc">Discord asset name (e.g. 'logo') or direct HTTPS image URL (e.g. from Imgur)</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. logo"
                v-model="s.discord.idleLargeImageKey"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
          <!-- Large Image Text -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Large Image Hover Text</span>
              <span class="setting-desc">Tooltip shown when hovering the large icon</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. BejaClient"
                v-model="s.discord.idleLargeImageText"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
          <!-- Small Image Key -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Small Image Key</span>
              <span class="setting-desc">Discord asset name (e.g. 'active') or direct HTTPS image URL</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. active"
                v-model="s.discord.idleSmallImageKey"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
          <!-- Small Image Text -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Small Image Hover Text</span>
              <span class="setting-desc">Tooltip shown when hovering the small icon</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. Online"
                v-model="s.discord.idleSmallImageText"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Playing Tab Content -->
      <div v-show="activeTab === 'playing'" class="tab-content" :class="{ 'tab-content--disabled': !s.discord.enabled }">
        <div class="setting-group">
          <!-- Details -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Details</span>
              <span class="setting-desc">First line of status text. Use {version} for version, {profile} for profile name</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. Playing Minecraft {version}"
                v-model="s.discord.playingDetails"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
          <!-- State -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">State</span>
              <span class="setting-desc">Second line of status text. Use {version} for version, {profile} for profile name</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. In Game"
                v-model="s.discord.playingState"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
          <!-- Large Image Key -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Large Image Key</span>
              <span class="setting-desc">Discord asset name (e.g. 'logo') or direct HTTPS image URL (e.g. from Imgur)</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. logo"
                v-model="s.discord.playingLargeImageKey"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
          <!-- Large Image Text -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Large Image Hover Text</span>
              <span class="setting-desc">Tooltip shown when hovering the large icon</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. BejaClient"
                v-model="s.discord.playingLargeImageText"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
          <!-- Small Image Key -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Small Image Key</span>
              <span class="setting-desc">Discord asset name (e.g. 'active') or direct HTTPS image URL</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. playing"
                v-model="s.discord.playingSmallImageKey"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
          <!-- Small Image Text -->
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Small Image Hover Text</span>
              <span class="setting-desc">Tooltip shown when hovering the small icon</span>
            </div>
            <div class="setting-control">
              <input
                type="text"
                class="text-input wide-input"
                placeholder="e.g. In Game"
                v-model="s.discord.playingSmallImageText"
                :disabled="!s.discord.enabled"
                @change="save"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Right Column: Live Preview -->
    <div class="discord-preview-column">
      <div class="preview-title">Discord Live Preview</div>
      
      <div class="discord-card" :class="{ disabled: !s.discord.enabled }">
        <div class="discord-card-header">PLAYING A GAME</div>
        
        <div class="discord-card-body">
          <!-- Images block -->
          <div class="discord-assets-wrapper">
            <!-- Large Image -->
            <div class="discord-large-img-container">
              <img
                :src="previewLargeImageSrc"
                class="discord-large-img"
                :title="previewLargeImageText"
                alt="Large Image"
              />
              <!-- Small Image Overlay -->
              <div v-if="previewSmallImageSrc" class="discord-small-img-container">
                <img
                  :src="previewSmallImageSrc"
                  class="discord-small-img"
                  :title="previewSmallImageText"
                  alt="Small Image"
                />
              </div>
            </div>
          </div>
          
          <!-- Texts block -->
          <div class="discord-details-wrapper">
            <div class="discord-app-name" title="App Name">
              {{ s.discord.clientId.trim() ? 'Custom App' : 'BejaClient' }}
            </div>
            <div class="discord-details" :title="previewDetails">
              {{ previewDetails }}
            </div>
            <div class="discord-state" :title="previewState">
              {{ previewState }}
            </div>
            <div class="discord-time-elapsed">
              00:42 elapsed
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useSettingsStore } from '../../store/settingsStore'

const settingsStore = useSettingsStore()
const s = computed(() => settingsStore.settings)
const activeTab = ref<'general' | 'idle' | 'playing'>('general')

async function save() {
  await settingsStore.save()
}

// Fallback images
const DEFAULT_LOGO_URL = new URL('../../assets/bc-logo-new.png', import.meta.url).href

// Preview variables (replace placeholders for preview)
const previewDetails = computed(() => {
  if (activeTab.value === 'playing') {
    const text = s.value.discord.playingDetails || 'Playing Minecraft {version}'
    return text.replace(/{version}/g, '1.21.11').replace(/{profile}/g, 'test')
  } else {
    return s.value.discord.idleDetails || 'Browsing the launcher'
  }
})

const previewState = computed(() => {
  if (activeTab.value === 'playing') {
    const text = s.value.discord.playingState || 'In Game'
    return text.replace(/{version}/g, '1.21.11').replace(/{profile}/g, 'test')
  } else {
    return s.value.discord.idleState || 'Idle'
  }
})

const previewLargeImageSrc = computed(() => {
  const key = activeTab.value === 'playing'
    ? s.value.discord.playingLargeImageKey
    : s.value.discord.idleLargeImageKey
    
  if (!key || key.trim() === '') return DEFAULT_LOGO_URL
  if (key.trim().startsWith('http://') || key.trim().startsWith('https://')) {
    return key.trim()
  }
  return DEFAULT_LOGO_URL
})

const previewLargeImageText = computed(() => {
  const text = activeTab.value === 'playing'
    ? s.value.discord.playingLargeImageText
    : s.value.discord.idleLargeImageText
    
  if (activeTab.value === 'playing') {
    return (text || 'BejaClient').replace(/{version}/g, '1.21.11').replace(/{profile}/g, 'test')
  }
  return text || 'BejaClient'
})

const previewSmallImageSrc = computed(() => {
  const key = activeTab.value === 'playing'
    ? s.value.discord.playingSmallImageKey
    : s.value.discord.idleSmallImageKey
    
  if (!key || key.trim() === '') return ''
  if (key.trim().startsWith('http://') || key.trim().startsWith('https://')) {
    return key.trim()
  }
  return 'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><circle cx="50" cy="50" r="50" fill="%2323a55a"/></svg>'
})

const previewSmallImageText = computed(() => {
  const text = activeTab.value === 'playing'
    ? s.value.discord.playingSmallImageText
    : s.value.discord.idleSmallImageText
    
  if (activeTab.value === 'playing') {
    return (text || '').replace(/{version}/g, '1.21.11').replace(/{profile}/g, 'test')
  }
  return text || ''
})
</script>

<style lang="scss" scoped>
.discord-settings-container {
  display: flex;
  flex-direction: row;
  gap: 30px;
  align-items: flex-start;
  width: 100%;
}

.settings-form {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.discord-preview-column {
  width: 310px;
  flex-shrink: 0;
  position: sticky;
  top: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.preview-title {
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  color: #555;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

/* Tabs */
.rpc-tabs {
  display: flex;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  gap: 6px;
  padding-bottom: 2px;
}

.rpc-tab-btn {
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  letter-spacing: 0.08em;
  padding: 8px 16px;
  background: transparent;
  border: 1px solid transparent;
  border-bottom: none;
  color: #555;
  cursor: pointer;
  transition: all 80ms;
  text-transform: uppercase;

  &:hover {
    color: #aaa;
    background: rgba(255, 255, 255, 0.02);
  }

  &.active {
    color: var(--accent, #27ade0);
    border-color: rgba(255, 255, 255, 0.06);
    background: rgba(8, 8, 10, 0.6);
    box-shadow: inset 0 2px 0 var(--accent, #27ade0);
  }
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
  
  &--disabled {
    opacity: 0.5;
    pointer-events: none;
  }
}

/* Discord Mock Card */
.discord-card {
  background: #1e1f22;
  border-radius: 8px;
  padding: 16px;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  color: #dbdee1;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  transition: opacity 150ms;
  width: 100%;

  &.disabled {
    opacity: 0.3;
  }
}

.discord-card-header {
  font-size: 10px;
  font-weight: 800;
  color: #949ba4;
  letter-spacing: 0.04em;
  margin-bottom: 12px;
}

.discord-card-body {
  display: flex;
  flex-direction: row;
  align-items: center;
}

.discord-assets-wrapper {
  position: relative;
  flex-shrink: 0;
}

.discord-large-img-container {
  width: 72px;
  height: 72px;
  position: relative;
}

.discord-large-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 8px;
  background: #111214;
}

.discord-small-img-container {
  position: absolute;
  bottom: -4px;
  right: -4px;
  width: 24px;
  height: 24px;
  background: #1e1f22;
  border-radius: 50%;
  padding: 2.5px;
}

.discord-small-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 50%;
}

.discord-details-wrapper {
  margin-left: 12px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-width: 0;
  flex: 1;
}

.discord-app-name {
  font-size: 13.5px;
  font-weight: 700;
  color: #ffffff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 2px;
}

.discord-details,
.discord-state {
  font-size: 12.5px;
  color: #dbdee1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.35;
}

.discord-time-elapsed {
  font-size: 11.5px;
  color: #949ba4;
  margin-top: 3px;
  line-height: 1.2;
}

/* Reusing standard rows styles */
.setting-group {
  display: flex;
  flex-direction: column;
  border: 1px solid rgba(255, 255, 255, 0.05);
  background: rgba(8, 8, 10, 0.6);
  overflow: hidden;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 14px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  min-height: 56px;
  transition: background 80ms;

  &:last-child { border-bottom: none; }
  &:hover { background: rgba(255, 255, 255, 0.02); }
  &--tall { align-items: flex-start; padding-top: 16px; padding-bottom: 16px; }
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
  flex-shrink: 0;
  max-width: 55%;
}

.setting-label {
  font-family: 'Mojangles', monospace;
  font-size: 11px;
  color: #d9d9d9;
  letter-spacing: 0.02em;
}

.setting-desc {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  color: #444;
  letter-spacing: 0.03em;
  line-height: 1.5;
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.toggle {
  width: 38px;
  height: 22px;
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 11px;
  position: relative;
  cursor: pointer;
  transition: background 150ms, border-color 150ms;
  flex-shrink: 0;

  &::after {
    content: '';
    position: absolute;
    width: 14px;
    height: 14px;
    background: #444;
    border-radius: 50%;
    top: 3px;
    left: 3px;
    transition: transform 150ms cubic-bezier(0.2, 0, 0, 1), background 150ms;
  }

  &--on {
    background: color-mix(in srgb, var(--accent, #27ade0) 18%, #0d0d0d);
    border-color: color-mix(in srgb, var(--accent, #27ade0) 60%, transparent);

    &::after {
      background: var(--accent, #27ade0);
      transform: translateX(16px);
    }
  }
}

.text-input {
  height: 32px;
  background: #0a0a0b;
  border: 1px solid rgba(118, 119, 120, 0.5);
  color: #aaa;
  font-family: 'Mojangles', monospace;
  font-size: 10px;
  letter-spacing: 0.02em;
  padding: 0 10px;
  outline: none;
  transition: border-color 100ms;
  width: 200px;

  &:focus { border-color: rgba(255, 255, 255, 0.35); color: #ccc; }
  &::placeholder { color: #333; }

  &.wide-input { width: 260px; }
}
</style>
