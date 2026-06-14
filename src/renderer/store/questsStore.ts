import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import type { Quest, LeaderboardEntry } from '../types'
import { useLauncherStore } from './launcherStore'
import { useFriendsStore } from './friendsStore'
import { useLobbyStore } from './lobbyStore'
import { useLockerStore } from './lockerStore'

export const useQuestsStore = defineStore('quests', () => {
  const quests       = ref<Quest[]>([])
  const week         = ref('')
  const loading      = ref(false)
  const leaderboard  = ref<LeaderboardEntry[]>([])
  const myRank       = ref<number | null>(null)
  const lbLoading    = ref(false)
  const totalXp      = ref(0)              // lifetime XP from player_pass
  const lastAward    = ref<number | null>(null)  // for the +XP toast animation

  const claimableCount = computed(() =>
    quests.value.filter(q => !q.claimed && q.progress >= q.goal).length)

  async function load() {
    loading.value = true
    try {
      const res = await window.api.quests.list()
      quests.value = res.quests ?? []
      week.value   = res.week ?? ''
    } finally {
      loading.value = false
    }
  }

  async function loadLeaderboard() {
    lbLoading.value = true
    try {
      const res = await window.api.quests.leaderboard()
      leaderboard.value = res.entries ?? []
      myRank.value      = res.myRank ?? null
    } finally {
      lbLoading.value = false
    }
  }

  async function claim(questId: string) {
    const res = await window.api.quests.claim(questId)
    if (res?.awarded) {
      const q = quests.value.find(q => q.id === questId)
      if (q) q.claimed = true
      if (typeof res.xp === 'number') totalXp.value = res.xp
      lastAward.value = res.xp_gained ?? null
      setTimeout(() => { lastAward.value = null }, 2400)
      loadLeaderboard()
    }
    return res
  }

  async function report(questId: string, amount = 1) {
    const res = await window.api.quests.progress(questId, amount)
    if (res) {
      const q = quests.value.find(q => q.id === res.questId)
      if (q) q.progress = Math.min(res.progress, q.goal)
    }
  }

  // ── Automatic progress tracking ──────────────────────────────────────────────
  // Called once from App.vue after stores are ready. Watches existing app state
  // so no other store needs to know about quests.
  let tracking = false
  function setupTracking() {
    if (tracking) return
    tracking = true

    const launcherStore = useLauncherStore()
    const friendsStore  = useFriendsStore()
    const lobbyStore    = useLobbyStore()
    const lockerStore   = useLockerStore()

    // launch_3 — each transition into 'running' counts one launch.
    // playtime_60 — one tick per minute while the game runs.
    // party_1 — launching while in a party with someone else.
    let playtimeTimer: ReturnType<typeof setInterval> | null = null
    watch(() => launcherStore.status, (val, prev) => {
      if (val === 'running' && prev !== 'running') {
        report('launch_3', 1)
        if ((lobbyStore.party?.members?.length ?? 0) >= 2) report('party_1', 1)
        playtimeTimer = setInterval(() => report('playtime_60', 1), 60_000)
      } else if (val !== 'running' && playtimeTimer) {
        clearInterval(playtimeTimer)
        playtimeTimer = null
      }
    })

    // friend_1 — friends list grew while the app is open
    watch(() => friendsStore.friends.length, (val, prev) => {
      if (prev !== undefined && val > prev) report('friend_1', 1)
    })

    // cosmetic_1 — cape equipped after initial load settles
    let cosmeticArmed = false
    setTimeout(() => { cosmeticArmed = true }, 10_000)
    watch(() => lockerStore.capeUrl, (val, prev) => {
      if (cosmeticArmed && val && val !== prev) report('cosmetic_1', 1)
    })
  }

  return {
    quests, week, loading,
    leaderboard, myRank, lbLoading,
    totalXp, lastAward, claimableCount,
    load, loadLeaderboard, claim, report, setupTracking,
  }
})
