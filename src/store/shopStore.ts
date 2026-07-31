import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ShopItem } from '../types'
import { useWalletStore } from './walletStore'

export const useShopStore = defineStore('shop', () => {
  const items         = ref<ShopItem[]>([])
  const owned         = ref<string[]>([])
  const loading       = ref(false)
  const purchasingId  = ref<string | null>(null)
  const lastError     = ref<string | null>(null)

  async function load() {
    loading.value = true
    try {
      const res = await window.api.shop.list()
      items.value = res.items ?? []
      owned.value = res.owned ?? []
    } finally {
      loading.value = false
    }
  }

  async function purchase(itemId: string) {
    purchasingId.value = itemId
    lastError.value = null
    try {
      const res = await window.api.shop.purchase(itemId)
      if (res.purchased) {
        if (!owned.value.includes(itemId)) owned.value.push(itemId)
        if (typeof res.balance === 'number') useWalletStore().setBalance(res.balance)
      } else {
        lastError.value = res.reason ?? 'purchase_failed'
      }
      return res
    } finally {
      purchasingId.value = null
    }
  }

  return { items, owned, loading, purchasingId, lastError, load, purchase }
})
