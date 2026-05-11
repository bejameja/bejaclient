import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { LaunchProfile, LaunchStatus, VersionProgress, NewsEntry } from '../types'

export const useLauncherStore = defineStore('launcher', () => {
  const profiles = ref<LaunchProfile[]>([])
  const activeProfile = ref<LaunchProfile | null>(null)
  const status = ref<LaunchStatus>('idle')
  const statusMsg = ref<string>('')
  const logs = ref<string[]>([])
  const lastError = ref<string | null>(null)
  const installProgress = ref<VersionProgress | null>(null)
  const news = ref<NewsEntry[]>([])

  const isRunning = computed(() => status.value === 'running')
  const isLaunching = computed(() => status.value === 'starting')

  async function loadProfiles() {
    profiles.value = await window.api.profiles.list()
    activeProfile.value = await window.api.profiles.getActive()
  }

  async function createProfile(data: Omit<LaunchProfile, 'id' | 'createdAt' | 'lastPlayed'>) {
    const profile = await window.api.profiles.create(JSON.parse(JSON.stringify(data)))
    profiles.value.push(profile)
    if (!activeProfile.value) {
      await setActiveProfile(profile.id)
    }
    return profile
  }

  async function updateProfile(id: string, data: Partial<LaunchProfile>) {
    const updated = await window.api.profiles.update(id, JSON.parse(JSON.stringify(data)))
    if (updated) {
      const idx = profiles.value.findIndex(p => p.id === id)
      if (idx >= 0) profiles.value[idx] = updated
      if (activeProfile.value?.id === id) activeProfile.value = updated
    }
    return updated
  }

  async function deleteProfile(id: string) {
    await window.api.profiles.delete(id)
    profiles.value = profiles.value.filter(p => p.id !== id)
    if (activeProfile.value?.id === id) {
      const next = profiles.value[0] ?? null
      if (next) {
        await setActiveProfile(next.id)
      } else {
        activeProfile.value = null
      }
    }
  }

  async function setActiveProfile(id: string) {
    activeProfile.value = await window.api.profiles.setActive(id)
  }

  async function launch() {
    if (!activeProfile.value) return
    status.value = 'starting'
    logs.value = []
    lastError.value = null
    try {
      await window.api.launch.start(activeProfile.value.id)
    } catch (e) {
      status.value = 'error'
      lastError.value = String(e)
    }
  }

  async function killGame() {
    await window.api.launch.kill()
    status.value = 'idle'
  }

  function setupLaunchListeners() {
    window.api.launch.onLog(line => {
      logs.value.push(line)
      if (logs.value.length > 500) logs.value.shift()
    })
    window.api.launch.onStatus(s => {
      if (s === 'running') {
        status.value = 'running'
        statusMsg.value = ''
      } else if (s.startsWith('stopped:')) {
        const code = parseInt(s.split(':')[1] ?? '0', 10)
        statusMsg.value = ''
        if (code === 0) {
          status.value = 'idle'
        } else {
          status.value = 'error'
          const errLines = logs.value.filter(l => l.startsWith('[ERR]')).slice(-5)
          lastError.value = errLines.length
            ? errLines.join('\n')
            : `Process exited with code ${code}`
        }
      } else if (s === 'starting') {
        status.value = 'starting'
        statusMsg.value = ''
      } else {
        statusMsg.value = s
      }
    })
    window.api.versions.onProgress(p => {
      installProgress.value = p
    })
  }

  async function fetchNews() {
    try {
      news.value = await window.api.news.fetch()
    } catch {
      news.value = []
    }
  }

  return {
    profiles,
    activeProfile,
    status,
    statusMsg,
    logs,
    lastError,
    installProgress,
    news,
    isRunning,
    isLaunching,
    loadProfiles,
    createProfile,
    updateProfile,
    deleteProfile,
    setActiveProfile,
    launch,
    killGame,
    setupLaunchListeners,
    fetchNews,
  }
})
