<template>
  <div class="friends-page">

    <!-- ── Left sidebar ─────────────────────────────────────────────────────── -->
    <div class="fp-sidebar">

      <div class="fp-toggle-row">
        <button class="fp-toggle-btn" :class="{ active: sideTab === 'friends' }" @click="sideTab = 'friends'">
          <span class="fp-friends-word">{{ $t('friends.tabs.friends') }}</span>
        </button>
        <button class="fp-toggle-btn" :class="{ active: sideTab === 'groups' }" @click="sideTab = 'groups'">
          <span class="fp-groups-word">Groups</span>
        </button>
      </div>

      <div class="fp-search-wrap" :class="{ focused: searchFocused }">
        <input
          v-model="search"
          class="fp-search-input"
          :placeholder="$t('friends.searchPlaceholder')"
          spellcheck="false"
          @focus="searchFocused = true"
          @blur="onSearchBlur"
          @input="onAddInput"
          @keyup.enter="sendRequest"
        />
        <img :src="findFriendIcon" class="fp-search-icon" alt="" />

        <!-- Add-friend suggestions — same lookup/add flow as before, now living in the sidebar search box -->
        <div v-if="searchFocused && suggestions.length" class="fp-suggest-drop" @mousedown.prevent>
          <div v-for="s in suggestions" :key="s.uuid" class="fp-suggest-row">
            <img
              class="fp-suggest-head"
              :src="`https://mc-heads.net/head/${s.uuid}/64`"
              :alt="s.username"
              @error="(e: Event) => ((e.target as HTMLImageElement).src = 'https://mc-heads.net/head/MHF_Steve/64')"
            />
            <div class="fp-suggest-info">
              <span class="fp-suggest-name">{{ s.username }}</span>
              <span class="fp-suggest-tag" :class="`fp-suggest-tag--${s.source}`">
                {{ s.source === 'beja' ? 'BEJACLIENT' : 'MINECRAFT' }}
              </span>
            </div>
            <button class="fp-suggest-btn fp-suggest-btn--primary" @click="addSuggestion(s)">{{ $t('friends.add') }}</button>
          </div>
        </div>
      </div>

      <template v-if="sideTab === 'friends'">

        <!-- Pending requests — compact, kept alongside the list rather than a separate tab -->
        <template v-if="friendsStore.incomingRequests.length">
          <div class="fp-section-label">{{ $t('friends.sections.incoming', { count: friendsStore.incomingRequests.length }) }}</div>
          <div class="fp-list fp-list--requests">
            <div v-for="r in friendsStore.incomingRequests" :key="r.uuid" class="fp-request-row">
              <img
                class="fp-avatar fp-avatar--sm"
                :src="`https://mc-heads.net/head/${r.uuid}/64`"
                :alt="r.username"
                @error="(e: Event) => ((e.target as HTMLImageElement).src = 'https://mc-heads.net/head/MHF_Steve/64')"
              />
              <span class="fp-friend-name">{{ r.username }}</span>
              <div class="fp-request-actions">
                <button class="fp-req-btn fp-req-btn--accept" title="Accept" @click="acceptRequest(r.uuid)">✓</button>
                <button class="fp-req-btn fp-req-btn--decline" title="Decline" @click="declineRequest(r.uuid)">✕</button>
              </div>
            </div>
          </div>
        </template>

        <div class="fp-list">
          <div
            v-for="f in filteredFriends"
            :key="f.uuid"
            class="fp-friend-row"
            :class="{ active: chatFriend?.uuid === f.uuid }"
            @click="openChat(f)"
          >
            <div class="fp-avatar-wrap">
              <img
                class="fp-avatar"
                :src="`https://mc-heads.net/head/${f.uuid}/64`"
                :alt="f.username"
                @error="(e: Event) => ((e.target as HTMLImageElement).src = 'https://mc-heads.net/head/MHF_Steve/64')"
              />
              <span class="fp-status-dot" :class="{ online: f.online }" />
            </div>
            <span class="fp-friend-name">{{ f.username }}</span>
          </div>

          <p v-if="!loading && !filteredFriends.length" class="fp-empty">
            {{ search ? $t('friends.noResults') : $t('friends.noFriends') }}
          </p>
        </div>
      </template>

      <template v-else>
        <div class="fp-groups-empty">
          <span class="fp-groups-empty-text">Groups — coming soon</span>
        </div>
      </template>

    </div>

    <!-- ── Right: chat pane (always present, not a slide-over) ─────────────── -->
    <div class="fp-chat" v-if="chatFriend">

      <div class="fp-chat-header">
        <img
          class="fp-chat-avatar"
          :src="`https://mc-heads.net/head/${chatFriend.uuid}/64`"
          @error="(e: Event) => ((e.target as HTMLImageElement).src = `https://mc-heads.net/head/MHF_Steve/64`)"
        />
        <span class="fp-chat-name">{{ chatFriend.username }}</span>
        <span v-if="friendTyping" class="fp-chat-typing">{{ $t('friends.chat.typing') }}</span>

        <div class="fp-chat-header-actions">
          <button class="fp-header-icon-btn" title="Call" disabled><img :src="callIcon" alt="" /></button>
          <div class="fp-popover-anchor">
            <button
              class="fp-header-icon-btn"
              :class="{ active: pinnedOpen }"
              title="Pinned messages"
              @click="pinnedOpen = !pinnedOpen; searchOpen = false"
            >
              <img :src="pinIcon" alt="" />
              <span v-if="pinnedForFriend.length" class="fp-header-badge">{{ pinnedForFriend.length }}</span>
            </button>
            <div v-if="pinnedOpen" class="fp-popover fp-pinned-popover" @mousedown.stop>
              <div class="fp-popover-title">Pinned messages <span class="fp-popover-hint">(only visible on this device)</span></div>
              <div v-if="!pinnedForFriend.length" class="fp-popover-empty">No pinned messages yet — hover a message and click the pin icon.</div>
              <div v-else class="fp-pinned-list">
                <div v-for="msg in pinnedForFriend" :key="msg.id" class="fp-pinned-item">
                  <span class="fp-pinned-text">{{ msg.content }}</span>
                  <button class="fp-pinned-unpin" title="Unpin" @click="togglePin(msg)">✕</button>
                </div>
              </div>
            </div>
          </div>
          <div class="fp-popover-anchor">
            <button
              class="fp-header-icon-btn"
              :class="{ active: searchOpen }"
              title="Search messages"
              @click="searchOpen = !searchOpen; pinnedOpen = false; if (searchOpen) nextTick(() => searchInputEl?.focus())"
            >
              <img :src="moreIcon" alt="" />
            </button>
          </div>
        </div>
      </div>

      <div v-if="searchOpen" class="fp-msg-search-row">
        <input
          ref="searchInputEl"
          v-model="msgSearch"
          class="fp-msg-search-input"
          placeholder="Search this conversation…"
        />
        <span v-if="msgSearch" class="fp-msg-search-count">{{ visibleMessages.length }} found</span>
      </div>

      <div class="fp-chat-messages" ref="chatScrollEl">
        <div v-if="chatLoading" class="fp-chat-state">
          <span class="fp-spinner" />
        </div>
        <div v-else-if="!visibleMessages.length" class="fp-chat-state">
          <span class="fp-chat-empty-text">{{ msgSearch ? 'No messages match your search' : $t('friends.chat.noMessages') }}</span>
        </div>
        <template v-else>
          <div
            v-for="msg in visibleMessages"
            :key="msg.id"
            class="fp-msg"
            :class="{ 'fp-msg--mine': msg.fromUuid === myUuid }"
          >
            <div class="fp-msg-row">
              <button
                class="fp-pin-toggle"
                :class="{ pinned: isPinned(msg) }"
                title="Pin message"
                @click="togglePin(msg)"
              >📌</button>
              <span class="fp-msg-bubble" @contextmenu="onMsgContextMenu($event, msg)">
                <template v-for="(part, i) in parseChatContent(msg.content)" :key="i">
                  <img v-if="part.type === 'image'" :src="part.value" class="fp-msg-image" alt="" />
                  <a
                    v-else-if="part.type === 'link'"
                    :href="part.value"
                    class="fp-chat-link"
                    :title="$t('friends.chat.linkHint')"
                    @click="onChatLinkClick($event, part.value)"
                  >{{ part.value }}</a>
                  <template v-else>{{ part.value }}</template>
                </template>
              </span>
            </div>
            <span class="fp-msg-time">{{ formatTime(msg.sentAt) }}</span>
          </div>
        </template>
      </div>

      <div class="fp-chat-input-row">
        <input ref="fileInputEl" type="file" accept="image/*" class="fp-file-input" @change="onFilePicked" />
        <button class="fp-attach-btn" title="Send an image" @click="fileInputEl?.click()">+</button>
        <input
          v-model="chatInput"
          class="fp-chat-input"
          :placeholder="$t('friends.chat.messagePlaceholder', { name: chatFriend.username })"
          spellcheck="false"
          maxlength="2000"
          @keyup.enter="sendChat"
          @input="onChatInput"
        />
        <div class="fp-popover-anchor">
          <button class="fp-icon-btn" :class="{ active: gifOpen }" title="GIF" @click="toggleGifPopover">
            <img :src="gifIcon" alt="" />
          </button>
          <div v-if="gifOpen" class="fp-popover fp-gif-popover" @mousedown.stop>
            <input
              ref="gifSearchEl"
              v-model="gifQuery"
              class="fp-gif-search-input"
              placeholder="Search GIFs…"
              @input="onGifQueryInput"
            />
            <div class="fp-gif-grid">
              <span v-if="gifLoading" class="fp-gif-status">Loading…</span>
              <span v-else-if="gifError" class="fp-gif-status">{{ gifError }}</span>
              <span v-else-if="!gifResults.length" class="fp-gif-status">No GIFs found.</span>
              <button
                v-for="g in gifResults"
                :key="g.id"
                class="fp-gif-thumb"
                :title="g.title"
                @click="sendGif(g)"
              >
                <img :src="g.thumb" :alt="g.title" loading="lazy" />
              </button>
            </div>
            <div class="fp-popover-hint fp-gif-credit">Powered by GIPHY</div>
          </div>
        </div>
        <div class="fp-popover-anchor">
          <button class="fp-icon-btn" :class="{ active: emojiOpen }" title="Emoji" @click="emojiOpen = !emojiOpen; gifOpen = false">
            <img :src="emojiIcon" alt="" />
          </button>
          <div v-if="emojiOpen" class="fp-popover fp-emoji-popover" @mousedown.stop>
            <button v-for="e in EMOJI_SET" :key="e" class="fp-emoji-btn" @click="insertEmoji(e)">{{ e }}</button>
          </div>
        </div>
      </div>

    </div>

    <div v-else class="fp-chat fp-chat--empty">
      <span class="fp-chat-empty-hint">{{ $t('friends.noFriendsSub') }}</span>
    </div>

    <!-- Toast -->
    <Transition name="toast">
      <div v-if="toast" class="toast" :class="`toast--${toast.type}`">{{ toast.msg }}</div>
    </Transition>

    <PlayerProfileModal v-model="profileOpen" :player="profilePlayer" />

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFriendsStore } from '../store/friendsStore'
import { useAccountStore } from '../store/accountStore'
import type { ChatMessage, PlayerProfile } from '../types'
import findFriendIcon from '../assets/icons8-find-friend-50.png'
import callIcon from '../assets/icons8-call-50.png'
import pinIcon from '../assets/icons8-pin-50.png'
import moreIcon from '../assets/icons8-more-50.png'
import gifIcon from '../assets/icons8-gif-50.png'
import emojiIcon from '../assets/icons8-emoji-50.png'
import PlayerProfileModal from '../components/friends/PlayerProfileModal.vue'
import { openContextMenu, type ContextMenuItem } from '../composables/useContextMenu'

const friendsStore  = useFriendsStore()
const accountStore  = useAccountStore()
const route         = useRoute()
const myUuid        = computed(() => accountStore.selectedAccount?.uuid ?? '')
const { t } = useI18n()

const sideTab       = ref<'friends' | 'groups'>('friends')
const search        = ref('')
const searchFocused = ref(false)
const loading       = ref(false)

const filteredFriends = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return friendsStore.friends
  return friendsStore.friends.filter(f => f.username.toLowerCase().includes(q))
})

// ── Toast ──────────────────────────────────────────────────────────────────────
interface Toast { msg: string; type: 'ok' | 'err' | 'info' }
const toast = ref<Toast | null>(null)
let toastTimer: ReturnType<typeof setTimeout> | null = null

function showToast(msg: string, type: Toast['type'] = 'info') {
  toast.value = { msg, type }
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toast.value = null }, 3000)
}

// ── Username suggestions (search box doubles as add-friend input) ──────────────
interface Suggestion { uuid: string; username: string; source: 'beja' | 'mojang' }

const suggestions = ref<Suggestion[]>([])
let suggestTimer: ReturnType<typeof setTimeout> | null = null
let suggestSeq = 0

const profileOpen   = ref(false)
const profilePlayer = ref<PlayerProfile | null>(null)

function onAddInput() {
  if (suggestTimer) clearTimeout(suggestTimer)
  const q = search.value.trim()
  if (q.length < 2) { suggestions.value = []; return }
  suggestTimer = setTimeout(async () => {
    const seq = ++suggestSeq
    const results = await window.api.players.search(q)
    if (seq !== suggestSeq) return
    const bare = (u: string) => u.replace(/-/g, '').toLowerCase()
    suggestions.value = results.filter(s =>
      bare(s.uuid) !== bare(myUuid.value) &&
      !friendsStore.friends.some(f => bare(f.uuid) === bare(s.uuid))
    )
  }, 300)
}

function onSearchBlur() {
  searchFocused.value = false
}

async function addSuggestion(s: Suggestion) {
  suggestions.value = []
  const result = await friendsStore.sendRequest(s.username)
  if (result === 'sent') showToast(t('friends.toast.sent', { name: s.username }), 'ok')
  else if (result === 'already_pending') showToast(t('friends.toast.alreadyPending'), 'info')
  else if (result === 'already_friends') showToast(t('friends.toast.alreadyFriends', { name: s.username }), 'info')
  else showToast(t('friends.toast.error'), 'err')
}

async function sendRequest() {
  const name = search.value.trim()
  if (!name || !suggestions.value.length) return
  await addSuggestion(suggestions.value[0])
}

async function acceptRequest(uuid: string) {
  await friendsStore.acceptRequest(uuid)
  showToast(t('friends.toast.accepted'), 'ok')
}

async function declineRequest(uuid: string) {
  await friendsStore.declineRequest(uuid)
}

// ── Chat ───────────────────────────────────────────────────────────────────────
interface ChatFriend { uuid: string; username: string; online: boolean }

const chatFriend   = ref<ChatFriend | null>(null)
const chatMessages = ref<ChatMessage[]>([])
const chatInput    = ref('')
const chatLoading  = ref(false)
const chatScrollEl = ref<HTMLElement | null>(null)
const friendTyping = ref(false)

let typingClearTimer: ReturnType<typeof setTimeout> | null = null
let lastTypingSentAt = 0

async function openChat(friend: ChatFriend) {
  chatFriend.value   = friend
  chatMessages.value = []
  chatLoading.value  = true
  friendTyping.value = false
  friendsStore.activeChatUsername = friend.username
  if (typingClearTimer) { clearTimeout(typingClearTimer); typingClearTimer = null }
  try {
    chatMessages.value = await window.api.chat.history(friend.uuid)
  } catch { /* non-fatal */ }
  chatLoading.value = false
  scrollChatBottom()
}

async function sendChat() {
  const content = chatInput.value.trim()
  if (!content || !chatFriend.value) return
  chatInput.value = ''
  await window.api.chat.send(chatFriend.value.uuid, content)
}

const CHAT_URL_RE = /https?:\/\/[^\s]+/g
const TRAILING_PUNCT_RE = /[.,!?;:)\]}'"]+$/
const IMAGE_URL_RE = /\.(png|jpe?g|gif|webp)(\?.*)?$/i

function isImageUrl(url: string): boolean {
  return url.startsWith('data:image/') || IMAGE_URL_RE.test(url)
}

// A whole message that's nothing but a single image/GIF link or a pasted data
// URL renders as just the image, no surrounding link chrome.
function parseChatContent(content: string): { type: 'text' | 'link' | 'image'; value: string }[] {
  const trimmed = content.trim()
  if (isImageUrl(trimmed) && trimmed === content) {
    return [{ type: 'image', value: trimmed }]
  }

  const parts: { type: 'text' | 'link' | 'image'; value: string }[] = []
  let lastIndex = 0
  for (const match of content.matchAll(CHAT_URL_RE)) {
    const start = match.index ?? 0
    if (start > lastIndex) parts.push({ type: 'text', value: content.slice(lastIndex, start) })

    let url = match[0]
    const trailing = url.match(TRAILING_PUNCT_RE)?.[0] ?? ''
    if (trailing) url = url.slice(0, -trailing.length)

    parts.push({ type: isImageUrl(url) ? 'image' : 'link', value: url })
    if (trailing) parts.push({ type: 'text', value: trailing })
    lastIndex = start + match[0].length
  }
  if (lastIndex < content.length) parts.push({ type: 'text', value: content.slice(lastIndex) })
  return parts
}

function onChatLinkClick(e: MouseEvent, url: string) {
  e.preventDefault()
  if (e.ctrlKey || e.metaKey) window.api.system.openExternal(url)
}

// ── Message search (client-side, over already-loaded history) ──────────────────
const searchOpen    = ref(false)
const msgSearch     = ref('')
const searchInputEl = ref<HTMLInputElement | null>(null)

const visibleMessages = computed(() => {
  const q = msgSearch.value.trim().toLowerCase()
  if (!q) return chatMessages.value
  return chatMessages.value.filter(m => m.content.toLowerCase().includes(q))
})

// ── Pinned messages — local-only (no server field for this), keyed by friend uuid.
const PIN_STORAGE_KEY = 'beja_pinned_messages_v1'
const pinnedOpen = ref(false)
const pinnedIds  = ref<Record<string, string[]>>(loadPinnedIds())

function loadPinnedIds(): Record<string, string[]> {
  try {
    const raw = localStorage.getItem(PIN_STORAGE_KEY)
    return raw ? JSON.parse(raw) : {}
  } catch { return {} }
}

function savePinnedIds() {
  localStorage.setItem(PIN_STORAGE_KEY, JSON.stringify(pinnedIds.value))
}

const pinnedForFriend = computed(() => {
  if (!chatFriend.value) return []
  const ids = new Set(pinnedIds.value[chatFriend.value.uuid] ?? [])
  return chatMessages.value.filter(m => ids.has(String(m.id)))
})

function isPinned(msg: ChatMessage): boolean {
  if (!chatFriend.value) return false
  return (pinnedIds.value[chatFriend.value.uuid] ?? []).includes(String(msg.id))
}

function togglePin(msg: ChatMessage) {
  if (!chatFriend.value) return
  const uuid = chatFriend.value.uuid
  const current = pinnedIds.value[uuid] ?? []
  const id = String(msg.id)
  pinnedIds.value = {
    ...pinnedIds.value,
    [uuid]: current.includes(id) ? current.filter(x => x !== id) : [...current, id],
  }
  savePinnedIds()
}

// ── Custom right-click menu — only inside chat messages, replacing the native
// OS/browser menu there (elsewhere in the app right-click is untouched). ────
function onMsgContextMenu(event: MouseEvent, msg: ChatMessage) {
  const parts = parseChatContent(msg.content)
  const isSingleImage = parts.length === 1 && parts[0].type === 'image'

  const items: ContextMenuItem[] = isSingleImage
    ? [
        { label: 'Copy image link', icon: 'copy', onClick: () => navigator.clipboard.writeText(msg.content) },
        { label: 'Open image', icon: 'external-link', onClick: () => window.api.system.openExternal(msg.content) },
      ]
    : [
        { label: 'Copy text', icon: 'copy', onClick: () => navigator.clipboard.writeText(msg.content) },
      ]

  items.push(
    { separator: true, label: '' },
    { label: isPinned(msg) ? 'Unpin message' : 'Pin message', onClick: () => togglePin(msg) },
  )

  openContextMenu(event, items)
}

// ── Emoji picker ─────────────────────────────────────────────────────────────
const emojiOpen = ref(false)
const EMOJI_SET = [
  '😀', '😂', '😅', '😉', '😊', '😍', '😘', '😜', '🤔', '😎',
  '😢', '😭', '😡', '🥳', '😴', '🙄', '😱', '🤝', '👍', '👎',
  '👏', '🙏', '🔥', '💯', '❤️', '💀', '🎉', '✨', '⭐', '🎮',
]

function insertEmoji(e: string) {
  chatInput.value += e
  emojiOpen.value = false
}

// ── GIF search — proxied through Rust (giphy_service.rs) so the API key stays
// server-side, never shipped in the webview's JS bundle. Key itself lives in
// Settings → Launcher (settings.json's launcher.giphyApiKey).
interface GifResult { id: string; thumb: string; url: string; title: string }

const gifOpen    = ref(false)
const gifQuery   = ref('')
const gifResults = ref<GifResult[]>([])
const gifLoading = ref(false)
const gifError   = ref('')
const gifSearchEl = ref<HTMLInputElement | null>(null)
let gifDebounce: ReturnType<typeof setTimeout> | null = null
let gifSeq = 0

async function runGifSearch(query: string) {
  const seq = ++gifSeq
  gifLoading.value = true
  gifError.value = ''
  try {
    gifResults.value = await window.api.giphy.search(query)
  } catch (e) {
    if (seq !== gifSeq) return
    gifError.value = e instanceof Error ? e.message : String(e)
    gifResults.value = []
  } finally {
    if (seq === gifSeq) gifLoading.value = false
  }
}

function onGifQueryInput() {
  if (gifDebounce) clearTimeout(gifDebounce)
  gifDebounce = setTimeout(() => runGifSearch(gifQuery.value.trim()), 350)
}

async function sendGif(g: GifResult) {
  if (!chatFriend.value) return
  gifOpen.value = false
  await window.api.chat.send(chatFriend.value.uuid, g.url)
}

function toggleGifPopover() {
  gifOpen.value = !gifOpen.value
  emojiOpen.value = false
  if (gifOpen.value && !gifResults.value.length) runGifSearch('')
  if (gifOpen.value) nextTick(() => gifSearchEl.value?.focus())
}

// ── Send an image file ───────────────────────────────────────────────────────
// No chat-attachment endpoint exists server-side — content is plain text —
// so the picked file is sent as a data: URL through the same text pipe the
// rest of chat already uses, and rendered as an image by the parser above.
// Capped well under typical request-body limits since base64 inflates size ~33%.
const fileInputEl = ref<HTMLInputElement | null>(null)
const MAX_IMAGE_BYTES = 1.5 * 1024 * 1024

function onFilePicked(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file || !chatFriend.value) return
  if (file.size > MAX_IMAGE_BYTES) {
    showToast('Image too large (max 1.5MB) — try a smaller file or paste a link instead.', 'err')
    return
  }
  const reader = new FileReader()
  reader.onload = async () => {
    const dataUrl = reader.result as string
    await window.api.chat.send(chatFriend.value!.uuid, dataUrl)
  }
  reader.onerror = () => showToast('Could not read that image.', 'err')
  reader.readAsDataURL(file)
}

// ── Close popovers on outside click ──────────────────────────────────────────
function onDocClick(e: MouseEvent) {
  // .fp-msg-search-row is an inline bar (toggled by its own icon), not a floating
  // popover, so it's deliberately excluded here — only the three overlay popovers close.
  if (!(e.target as HTMLElement)?.closest?.('.fp-popover-anchor')) {
    pinnedOpen.value = false
    gifOpen.value = false
    emojiOpen.value = false
  }
}

function onChatInput() {
  if (!chatFriend.value) return
  const now = Date.now()
  if (now - lastTypingSentAt < 2000) return
  lastTypingSentAt = now
  window.api.chat.sendTyping(chatFriend.value.uuid)
}

function scrollChatBottom() {
  nextTick(() => {
    if (chatScrollEl.value) chatScrollEl.value.scrollTop = chatScrollEl.value.scrollHeight
  })
}

function formatTime(iso: string): string {
  const d = new Date(iso)
  const now = new Date()
  const sameDay = d.toDateString() === now.toDateString()
  if (sameDay) return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  return d.toLocaleDateString([], { month: 'short', day: 'numeric' }) + ' ' + d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

onMounted(async () => {
  loading.value = true
  await friendsStore.refresh()
  loading.value = false

  const chatWith = route.query.chatWith
  if (typeof chatWith === 'string') {
    const target = friendsStore.friends.find(f => f.uuid === chatWith)
    if (target) openChat(target)
  }

  window.api.chat.onTyping((d: { fromUuid: string }) => {
    if (!chatFriend.value || d.fromUuid !== chatFriend.value.uuid) return
    friendTyping.value = true
    if (typingClearTimer) clearTimeout(typingClearTimer)
    typingClearTimer = setTimeout(() => { friendTyping.value = false }, 3000)
  })

  window.api.chat.onMessage((msg: ChatMessage) => {
    if (
      chatFriend.value &&
      (msg.fromUuid === chatFriend.value.uuid || msg.toUuid === chatFriend.value.uuid)
    ) {
      chatMessages.value.push(msg)
      scrollChatBottom()
      friendTyping.value = false
      if (typingClearTimer) { clearTimeout(typingClearTimer); typingClearTimer = null }
    }
  })

  document.addEventListener('mousedown', onDocClick, true)
})

onUnmounted(() => {
  document.removeEventListener('mousedown', onDocClick, true)
  friendsStore.activeChatUsername = null
})
</script>

<style lang="scss" scoped>
// ── Page shell — persistent two-pane layout (sidebar list + chat), matching
// the Figma reference (node 424-24) instead of the old tab-grid + slide-over. ─
.friends-page {
  height: 100%;
  display: flex;
  overflow: hidden;
  position: relative;
  background: #0a0a0b;
}

// ── Sidebar ───────────────────────────────────────────────────────────────────
.fp-sidebar {
  width: 330px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 18px 16px;
  overflow: hidden;
}

.fp-toggle-row {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.fp-toggle-btn {
  padding: 9px 16px;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.55);
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.03em;
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease, border-color 140ms ease;

  &:hover { color: rgba(255, 255, 255, 0.85); border-color: rgba(255, 255, 255, 0.3); }

  &.active {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.3);
    color: #fff;
  }
}

// Same treatment as the "Friends" heading on the Hub page.
.fp-friends-word {
  display: inline-block;
  font-size: 17px;
  font-weight: 800;
  color: #4660FE;
  background: #090C20;
  border-radius: 4px;
  padding: 1px 6px;
  margin-left: -6px;
  font-family: 'Minecrafter', 'Plus Jakarta Sans', sans-serif;
  letter-spacing: 0.12em;
}

.fp-groups-word {
  display: inline-block;
  font-size: 17px;
  font-weight: 800;
  color: #F999E6;
  background: #21141E;
  border-radius: 4px;
  padding: 1px 6px;
  font-family: 'Minecrafter', 'Plus Jakarta Sans', sans-serif;
  letter-spacing: 0.12em;
}

// ── Search / add-friend box ──────────────────────────────────────────────────
.fp-search-wrap {
  position: relative;
  display: flex;
  align-items: center;
  height: 38px;
  background: #131315;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 0 10px;
  gap: 8px;
  flex-shrink: 0;
  transition: border-color 140ms ease;

  &.focused { border-color: rgba(255, 255, 255, 0.25); }
}

.fp-search-input {
  flex: 1;
  min-width: 0;
  background: none;
  border: none;
  outline: none;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 12.5px;
  color: rgba(255, 255, 255, 0.85);
  &::placeholder { color: rgba(255, 255, 255, 0.35); }
}

.fp-search-icon {
  width: 15px;
  height: 15px;
  flex-shrink: 0;
  opacity: 0.6;
  filter: brightness(0) invert(1);
}

.fp-suggest-drop {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: #131315;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  overflow: hidden;
  z-index: 30;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
}

.fp-suggest-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  transition: background 120ms ease;
  &:hover { background: rgba(255, 255, 255, 0.05); }
  & + & { border-top: 1px solid rgba(255, 255, 255, 0.05); }
}

.fp-suggest-head {
  width: 26px;
  height: 26px;
  border-radius: 5px;
  image-rendering: pixelated;
  flex-shrink: 0;
}

.fp-suggest-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.fp-suggest-name {
  font-size: 12.5px;
  font-weight: 600;
  color: $text-primary;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.fp-suggest-tag {
  font-size: 8px;
  letter-spacing: 0.08em;
  color: $text-muted;
  &--beja { color: $accent; }
}

.fp-suggest-btn {
  flex-shrink: 0;
  padding: 5px 10px;
  border-radius: 6px;
  border: none;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;

  &--primary {
    background: #fff;
    color: #0a0a0b;
    &:hover { background: rgba(255, 255, 255, 0.8); }
  }
}

// ── Section label + list ──────────────────────────────────────────────────────
.fp-section-label {
  font-size: 19px;
  font-weight: 800;
  color: #fff;
  font-family: 'Plus Jakarta Sans', sans-serif;
  flex-shrink: 0;
  margin-top: 4px;
}

.fp-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
  scrollbar-width: none;
  &::-webkit-scrollbar { display: none; }

  &--requests {
    flex: 0 0 auto;
    max-height: 160px;
    padding-bottom: 6px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    margin-bottom: 4px;
  }
}

.fp-empty {
  font-size: 12px;
  color: $text-muted;
  font-style: italic;
  padding: 6px 2px;
}

.fp-friend-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  margin: 0 -8px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 140ms ease;

  &:hover { background: rgba(255, 255, 255, 0.05); }
  &.active { background: rgba(255, 255, 255, 0.09); }
}

.fp-avatar-wrap {
  position: relative;
  flex-shrink: 0;
  width: 32px;
  height: 32px;
}

.fp-avatar {
  width: 32px;
  height: 32px;
  border-radius: 7px;
  image-rendering: pixelated;
  display: block;
  &--sm { width: 28px; height: 28px; border-radius: 6px; }
}

.fp-status-dot {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #3a3a3f;
  border: 2px solid #0a0a0b;
  &.online { background: #30d158; box-shadow: 0 0 5px rgba(48, 209, 88, 0.65); }
}

.fp-friend-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

// ── Requests mini-rows ────────────────────────────────────────────────────────
.fp-request-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px;
  margin: 0 -8px;
}

.fp-request-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.fp-req-btn {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  border: none;
  font-size: 11px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;

  &--accept { background: rgba(48, 209, 88, 0.15); color: #30d158; &:hover { background: rgba(48, 209, 88, 0.28); } }
  &--decline { background: rgba(255, 255, 255, 0.06); color: $text-muted; &:hover { background: rgba(248, 113, 113, 0.18); color: #f87171; } }
}

.fp-groups-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.fp-groups-empty-text {
  font-size: 12px;
  color: $text-muted;
  font-style: italic;
}

// ── Chat pane ─────────────────────────────────────────────────────────────────
.fp-chat {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  border-left: 1px solid rgba(255, 255, 255, 0.06);
  background: #0e0e10;

  &--empty {
    align-items: center;
    justify-content: center;
  }
}

.fp-chat-empty-hint {
  font-size: 13px;
  color: $text-muted;
}

.fp-chat-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.fp-chat-avatar {
  width: 34px;
  height: 34px;
  border-radius: 7px;
  image-rendering: pixelated;
  flex-shrink: 0;
}

.fp-chat-name {
  font-size: 16px;
  font-weight: 700;
  color: #fff;
  font-family: 'Plus Jakarta Sans', sans-serif;
}

.fp-chat-typing {
  font-size: 11px;
  color: #f97316;
  font-style: italic;
}

.fp-chat-header-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 4px;
}

.fp-header-icon-btn {
  position: relative;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: transparent;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 120ms ease;

  img { width: 16px; height: 16px; opacity: 0.7; filter: brightness(0) invert(1); transition: opacity 120ms ease; }
  &:hover, &.active { background: rgba(255, 255, 255, 0.07); img { opacity: 1; } }
  &:disabled { cursor: not-allowed; img { opacity: 0.3; } &:hover { background: transparent; } }
}

.fp-header-badge {
  position: absolute;
  top: 2px;
  right: 2px;
  min-width: 13px;
  height: 13px;
  padding: 0 3px;
  border-radius: 7px;
  background: #f97316;
  color: #fff;
  font-size: 8px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
}

// ── Popovers (pinned / GIF / emoji) ──────────────────────────────────────────
.fp-popover-anchor { position: relative; }

.fp-popover {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  width: 280px;
  background: #131315;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 12px;
  z-index: 40;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
}

.fp-popover-title {
  font-size: 12px;
  font-weight: 700;
  color: #fff;
  margin-bottom: 4px;
}

.fp-popover-hint {
  font-size: 10.5px;
  color: $text-muted;
  font-weight: 400;
}

.fp-popover-empty {
  font-size: 11.5px;
  color: $text-muted;
  padding: 6px 0 2px;
}

.fp-pinned-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 220px;
  overflow-y: auto;
  margin-top: 8px;
}

.fp-pinned-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 6px;
}

.fp-pinned-text {
  flex: 1;
  min-width: 0;
  font-size: 11.5px;
  color: rgba(255, 255, 255, 0.8);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.fp-pinned-unpin {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  border-radius: 5px;
  background: transparent;
  border: none;
  color: $text-muted;
  font-size: 9px;
  cursor: pointer;
  &:hover { background: rgba(248, 113, 113, 0.18); color: #f87171; }
}

// ── Inline message search bar ────────────────────────────────────────────────
.fp-msg-search-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.fp-msg-search-input {
  flex: 1;
  background: #131315;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  outline: none;
  padding: 6px 10px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.85);
  &::placeholder { color: rgba(255, 255, 255, 0.35); }
}

.fp-msg-search-count {
  font-size: 10.5px;
  color: $text-muted;
  flex-shrink: 0;
}

.fp-chat-messages {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  scrollbar-width: thin;
  scrollbar-color: #222 transparent;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: #222; }
}

.fp-chat-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.fp-chat-empty-text {
  font-size: 12px;
  color: $text-muted;
}

.fp-spinner {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid #2a2a2a;
  border-top-color: #ccc;
  animation: fp-spin 0.7s linear infinite;
}

@keyframes fp-spin { to { transform: rotate(360deg); } }

.fp-msg {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  max-width: 70%;

  &--mine {
    align-self: flex-end;
    align-items: flex-end;

    .fp-msg-row { flex-direction: row-reverse; }

    .fp-msg-bubble {
      background: rgba(255, 255, 255, 0.12);
      color: #fff;
    }
  }
}

.fp-msg-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

.fp-pin-toggle {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  border-radius: 5px;
  background: transparent;
  border: none;
  font-size: 10px;
  cursor: pointer;
  opacity: 0;
  filter: grayscale(1);
  transition: opacity 120ms ease, filter 120ms ease, background 120ms ease;

  .fp-msg:hover & { opacity: 0.5; }
  &:hover { opacity: 1 !important; background: rgba(255, 255, 255, 0.08); }
  &.pinned { opacity: 1; filter: none; }
}

.fp-msg-bubble {
  font-size: 16px;
  color: rgba(255, 255, 255, 0.85);
  background: rgba(255, 255, 255, 0.05);
  border-radius: 10px;
  padding: 10px 14px;
  line-height: 1.5;
  word-break: break-word;
  white-space: pre-wrap;
  user-select: text;
  cursor: text;
}

.fp-msg-image {
  display: block;
  max-width: 320px;
  max-height: 320px;
  border-radius: 8px;
  object-fit: contain;
}

.fp-chat-link {
  color: #f97316;
  text-decoration: underline;
  text-decoration-style: dotted;
  cursor: pointer;
  &:hover { color: #fbbf24; }
}

.fp-msg-time {
  font-size: 9px;
  color: $text-muted;
  padding: 0 2px;
}

.fp-chat-input-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0 20px 20px;
  padding: 4px 6px;
  background: #131315;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  flex-shrink: 0;
}

.fp-file-input { display: none; }

.fp-attach-btn {
  width: 30px;
  height: 30px;
  flex-shrink: 0;
  border-radius: 8px;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.5);
  font-size: 16px;
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease;
  &:hover { background: rgba(255, 255, 255, 0.07); color: #fff; }
}

.fp-chat-input {
  flex: 1;
  min-width: 0;
  background: none;
  border: none;
  outline: none;
  font-size: 16px;
  color: rgba(255, 255, 255, 0.9);
  padding: 10px 0;
  &::placeholder { color: rgba(255, 255, 255, 0.35); }
}

.fp-icon-btn {
  width: 30px;
  height: 30px;
  flex-shrink: 0;
  border-radius: 8px;
  background: transparent;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 120ms ease;

  img { width: 17px; height: 17px; opacity: 0.6; filter: brightness(0) invert(1); transition: opacity 120ms ease; }
  &:hover, &.active { background: rgba(255, 255, 255, 0.07); img { opacity: 1; } }
}

// ── GIF search popover ────────────────────────────────────────────────────────
.fp-gif-popover {
  bottom: calc(100% + 8px);
  top: auto;
  width: 320px;
}

.fp-gif-search-input {
  width: 100%;
  background: #0a0a0b;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  outline: none;
  padding: 8px 10px;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.9);
  box-sizing: border-box;
  &::placeholder { color: rgba(255, 255, 255, 0.3); }
}

.fp-gif-grid {
  margin-top: 8px;
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 4px;
  max-height: 260px;
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: #222 transparent;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: #222; }
}

.fp-gif-status {
  grid-column: 1 / -1;
  text-align: center;
  padding: 20px 0;
  font-size: 11.5px;
  color: $text-muted;
}

.fp-gif-thumb {
  aspect-ratio: 1;
  padding: 0;
  border: none;
  border-radius: 6px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.04);
  cursor: pointer;

  img { width: 100%; height: 100%; object-fit: cover; display: block; }
  &:hover { outline: 2px solid rgba(255, 255, 255, 0.4); outline-offset: -2px; }
}

.fp-gif-credit {
  margin-top: 8px;
  text-align: right;
}

// ── Emoji popover ─────────────────────────────────────────────────────────────
.fp-emoji-popover {
  bottom: calc(100% + 8px);
  top: auto;
  width: 240px;
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 2px;
  max-height: 200px;
  overflow-y: auto;
}

.fp-emoji-btn {
  width: 100%;
  aspect-ratio: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 6px;
  font-size: 18px;
  cursor: pointer;
  transition: background 100ms ease;
  &:hover { background: rgba(255, 255, 255, 0.08); }
}

// ── Toast ─────────────────────────────────────────────────────────────────────
.toast {
  position: absolute;
  bottom: 18px;
  left: 50%;
  transform: translateX(-50%);
  padding: 8px 20px;
  font-size: 12px;
  border-radius: 8px;
  border: 1px solid;
  white-space: nowrap;
  z-index: 50;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.7);

  &--ok   { background: rgba(8, 8, 10, 0.95); color: #30d158; border-color: rgba(48, 209, 88, 0.3); }
  &--err  { background: rgba(8, 8, 10, 0.95); color: #f87171; border-color: rgba(248, 113, 113, 0.3); }
  &--info { background: rgba(8, 8, 10, 0.95); color: #aaa;    border-color: rgba(255, 255, 255, 0.15); }
}

.toast-enter-active { transition: opacity 150ms, transform 150ms; }
.toast-leave-active { transition: opacity 200ms; }
.toast-enter-from   { opacity: 0; transform: translateX(-50%) translateY(8px); }
.toast-leave-to     { opacity: 0; }
</style>
