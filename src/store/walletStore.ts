import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useWalletStore = defineStore('wallet', () => {
  const balance = ref(0)
  const loading = ref(false)

  async function load() {
    loading.value = true
    try {
      const res = await window.api.wallet.getBalance()
      balance.value = res?.balance ?? 0
    } finally {
      loading.value = false
    }
  }

  function setBalance(value: number) {
    balance.value = value
  }

  return { balance, loading, load, setBalance }
})
