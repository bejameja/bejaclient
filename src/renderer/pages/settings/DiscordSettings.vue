<template>
  <div class="discord-settings">
    <div class="setting-group">
      <!-- Enable RPC Toggle -->
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
    </div>

    <!-- Only show customization if RPC is enabled -->
    <template v-if="s.discord.enabled">
      <!-- Client ID Configuration -->
      <div class="setting-group">
        <div class="setting-row setting-row--tall">
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
              @change="save"
            />
          </div>
        </div>
      </div>

      <!-- Idle Status (Launcher) -->
      <div class="section-title-sub">Idle Status (In Launcher)</div>
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
              @change="save"
            />
          </div>
        </div>
        <!-- Large Image Key -->
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Large Image Key</span>
            <span class="setting-desc">Discord asset name for large icon (e.g. 'logo')</span>
          </div>
          <div class="setting-control">
            <input
              type="text"
              class="text-input wide-input"
              placeholder="e.g. logo"
              v-model="s.discord.idleLargeImageKey"
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
              @change="save"
            />
          </div>
        </div>
        <!-- Small Image Key -->
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Small Image Key</span>
            <span class="setting-desc">Discord asset name for small icon overlay</span>
          </div>
          <div class="setting-control">
            <input
              type="text"
              class="text-input wide-input"
              placeholder="e.g. active"
              v-model="s.discord.idleSmallImageKey"
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
              @change="save"
            />
          </div>
        </div>
      </div>

      <!-- Playing Status (In Game) -->
      <div class="section-title-sub">Playing Status (In Game)</div>
      <div class="setting-group">
        <!-- Details -->
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Details</span>
            <span class="setting-desc">First line of status text. Use {version} for the Minecraft version</span>
          </div>
          <div class="setting-control">
            <input
              type="text"
              class="text-input wide-input"
              placeholder="e.g. Playing Minecraft {version}"
              v-model="s.discord.playingDetails"
              @change="save"
            />
          </div>
        </div>
        <!-- State -->
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">State</span>
            <span class="setting-desc">Second line of status text. Use {version} for the Minecraft version</span>
          </div>
          <div class="setting-control">
            <input
              type="text"
              class="text-input wide-input"
              placeholder="e.g. In Game"
              v-model="s.discord.playingState"
              @change="save"
            />
          </div>
        </div>
        <!-- Large Image Key -->
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Large Image Key</span>
            <span class="setting-desc">Discord asset name for large icon (e.g. 'logo')</span>
          </div>
          <div class="setting-control">
            <input
              type="text"
              class="text-input wide-input"
              placeholder="e.g. logo"
              v-model="s.discord.playingLargeImageKey"
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
              @change="save"
            />
          </div>
        </div>
        <!-- Small Image Key -->
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Small Image Key</span>
            <span class="setting-desc">Discord asset name for small icon overlay</span>
          </div>
          <div class="setting-control">
            <input
              type="text"
              class="text-input wide-input"
              placeholder="e.g. playing"
              v-model="s.discord.playingSmallImageKey"
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
              @change="save"
            />
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useSettingsStore } from '../../store/settingsStore'

const settingsStore = useSettingsStore()
const s = computed(() => settingsStore.settings)

async function save() {
  await settingsStore.save()
}
</script>

<style lang="scss" scoped>
.discord-settings {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.section-title-sub {
  font-family: 'Mojangles', monospace;
  font-size: 9px;
  color: #555;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  margin-top: 15px;
  margin-bottom: 5px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  padding-bottom: 6px;
}

// reuse styles from SettingsPage.vue
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
