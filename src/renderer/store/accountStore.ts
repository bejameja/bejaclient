import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Account } from '../types'
import { useFriendsStore } from './friendsStore'

export const useAccountStore = defineStore('account', () => {
  const accounts = ref<Account[]>([])
  const loading = ref(false)
  // eslint-disable-next-line prefer-const
  let error = ref<string | null>(null)

  const selectedAccount = computed(() => accounts.value.find(a => a.selected) ?? null)
  const hasAccounts = computed(() => accounts.value.length > 0)

  async function loadAccounts() {
    loading.value = true
    error.value = null
    try {
      accounts.value = await window.api.auth.listAccounts()
      if (accounts.value.some(a => a.selected)) {
        useFriendsStore().connect()
      }
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  const loginStatus = ref<string | null>(null)

  async function login(): Promise<Account | null> {
    loading.value = true
    loginStatus.value = null
    error.value = null
    const cleanup = () => { loginStatus.value = null }
    window.api.auth.onBrowserOpened(() => {
      loginStatus.value = 'Complete sign-in in your browser...'
    })
    try {
      const account = await window.api.auth.login()
      await loadAccounts()
      await selectAccount(account.id)
      return account
    } catch (e) {
      error.value = String(e)
      console.error('[Auth] login failed:', e)
      return null
    } finally {
      loading.value = false
      cleanup()
    }
  }

  async function logout(id: string) {
    await window.api.auth.logout(id)
    await loadAccounts()
  }

  async function selectAccount(id: string) {
    accounts.value = await window.api.auth.selectAccount(id)
    useFriendsStore().connect()
  }

  async function importFromLauncher() {
    loading.value = true
    error.value = null
    try {
      await window.api.auth.importFromLauncher()
      await loadAccounts()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function refreshAccount(id: string) {
    const refreshed = await window.api.auth.refreshAccount(id)
    if (refreshed) {
      const idx = accounts.value.findIndex(a => a.id === id)
      if (idx >= 0) accounts.value[idx] = refreshed
    }
  }

  return {
    accounts,
    loading,
    loginStatus,
    error,
    selectedAccount,
    hasAccounts,
    loadAccounts,
    login,
    logout,
    selectAccount,
    refreshAccount,
    importFromLauncher,
  }
})
