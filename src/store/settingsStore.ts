import { defineStore } from 'pinia'
import { ref, toRaw } from 'vue'
import type { AppSettings } from '../types'

const defaultSettings: AppSettings = {
  game: {
    defaultGameDir: '',
    defaultJavaPath: '',
    minRam: 512,
    maxRam: 2048,
    jvmArgs: '',
    resolution: { width: 854, height: 480 },
    fullscreen: false,
    extraGameArgs: '',
  },
  launcher: {
    closeOnLaunch: false,
    keepLauncherOpen: true,
    autoUpdate: true,
    concurrentDownloads: 16,
    soundEnabled: true,
    soundVolume: 50,
    soundStyle: 'soft' as const,
    curseforgeApiKey: '',
    advancedMode: false,
    debugLogging: false,
  },
  appearance: {
    language: 'en',
    accentColor: '#27ade0',
    reduceMotion: 'system',
    pageTransition: 'fade',
    disableHoverEffects: false,
    disableSplashScreen: false,
    theme: 'default',
  },
  customization: {
    experimentalEnabled: false,
    editorMode: false,
    wiggleEnabled: true,
    elements: {},
  },
  activeProfileId: null,
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({ ...defaultSettings })
  const loaded = ref(false)

  async function load() {
    const saved = await window.api.settings.get()
    settings.value = {
      ...defaultSettings,
      ...saved,
      game:       { ...defaultSettings.game,       ...saved.game },
      launcher:   { ...defaultSettings.launcher,   ...saved.launcher },
      appearance: { ...defaultSettings.appearance, ...saved.appearance },
      customization: {
        ...defaultSettings.customization,
        ...saved.customization,
        elements: { ...defaultSettings.customization.elements, ...saved.customization?.elements },
      },
    }
    // Migrate old orange default → launcher cyan
    if (settings.value.appearance.accentColor === '#F97316') {
      settings.value.appearance.accentColor = '#27ade0'
      await save()
    }
    // Editor Mode just became an Experimental-gated feature — a save from
    // before that change can have editorMode:true with no experimentalEnabled
    // key yet, which would leave it silently on with no UI left to turn it
    // off (that toggle only shows once Experimental is enabled).
    if (!settings.value.customization.experimentalEnabled && settings.value.customization.editorMode) {
      settings.value.customization.editorMode = false
      await save()
    }
    loaded.value = true
  }

  async function save() {
    await window.api.settings.set(toRaw(settings.value))
  }

  async function patch(patch: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...patch }
    await save()
  }

  async function chooseGameDir() {
    const dir = await window.api.settings.chooseDir()
    if (dir) {
      settings.value.game.defaultGameDir = dir
      await save()
    }
    return dir
  }

  async function chooseJava() {
    const path = await window.api.settings.chooseJava()
    if (path) {
      settings.value.game.defaultJavaPath = path
      await save()
    }
    return path
  }

  return {
    settings,
    loaded,
    load,
    save,
    patch,
    chooseGameDir,
    chooseJava,
  }
})
