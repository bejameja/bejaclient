import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface Friend {
  id: string
  gamertag: string
  addedAt: number
}

export interface FriendRequest {
  id: string
  gamertag: string
  direction: 'incoming' | 'outgoing'
  sentAt: number
}

const KEY = 'beja-friends'

export const useFriendsStore = defineStore('friends', () => {
  const friends     = ref<Friend[]>([])
  const requests    = ref<FriendRequest[]>([])
  const panelHidden = ref(false)

  const incomingRequests = computed(() => requests.value.filter(r => r.direction === 'incoming'))
  const outgoingRequests = computed(() => requests.value.filter(r => r.direction === 'outgoing'))
  const pendingCount     = computed(() => incomingRequests.value.length)

  function persist() {
    localStorage.setItem(KEY, JSON.stringify({ friends: friends.value, requests: requests.value, panelHidden: panelHidden.value }))
  }

  function load() {
    try {
      const raw = localStorage.getItem(KEY)
      if (raw) {
        const data = JSON.parse(raw)
        friends.value     = data.friends     ?? []
        requests.value    = data.requests    ?? []
        panelHidden.value = data.panelHidden ?? false
      }
    } catch {}
  }

  function sendRequest(gamertag: string): 'sent' | 'already_friends' | 'already_pending' {
    const lower = gamertag.trim().toLowerCase()
    if (!lower) return 'already_pending'
    if (friends.value.some(f => f.gamertag.toLowerCase() === lower))  return 'already_friends'
    if (requests.value.some(r => r.gamertag.toLowerCase() === lower)) return 'already_pending'
    requests.value.push({ id: crypto.randomUUID(), gamertag: gamertag.trim(), direction: 'outgoing', sentAt: Date.now() })
    persist()
    return 'sent'
  }

  function acceptRequest(id: string) {
    const req = requests.value.find(r => r.id === id)
    if (!req) return
    friends.value.push({ id: crypto.randomUUID(), gamertag: req.gamertag, addedAt: Date.now() })
    requests.value = requests.value.filter(r => r.id !== id)
    persist()
  }

  function declineRequest(id: string) {
    requests.value = requests.value.filter(r => r.id !== id)
    persist()
  }

  function cancelRequest(id: string) {
    requests.value = requests.value.filter(r => r.id !== id)
    persist()
  }

  function removeFriend(id: string) {
    friends.value = friends.value.filter(f => f.id !== id)
    persist()
  }

  function togglePanel() {
    panelHidden.value = !panelHidden.value
    persist()
  }

  load()

  return {
    friends, requests, panelHidden, incomingRequests, outgoingRequests, pendingCount,
    sendRequest, acceptRequest, declineRequest, cancelRequest, removeFriend, togglePanel,
  }
})
