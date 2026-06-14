<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="modelValue && player" class="modal-overlay" @click.self="close">
        <div class="modal">

          <button class="close-btn" @click="close">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>

          <div class="modal-body">

            <!-- 3D skin render -->
            <div class="skin-column">
              <HeroSkinViewer
                v-if="skinData"
                :skin-url="skinData"
                :cape-url="modelCapeData || null"
                :model="player.skinModel"
                animation="custom-idle"
                :zoom="0.78"
                :initial-rotation-y="0.4"
                class="skin-viewer"
              />
              <div v-else class="skin-placeholder">
                <div class="skin-spinner" />
              </div>
              <span class="drag-hint">Drag to rotate</span>
            </div>

            <!-- Info column -->
            <div class="info-column">
              <h2 class="player-name">{{ player.username }}</h2>

              <div class="field">
                <span class="field-label">UUID</span>
                <div class="uuid-row">
                  <code class="uuid-val">{{ player.uuid }}</code>
                  <button class="mini-btn" @click="copyUuid">{{ copiedUuid ? 'Copied!' : 'Copy' }}</button>
                </div>
              </div>

              <div class="field-grid">
                <div class="field">
                  <span class="field-label">BejaClient since</span>
                  <span class="field-val">{{ beja?.joinedAt ? fmtDate(beja.joinedAt) : (extraLoading ? '…' : 'Not registered') }}</span>
                </div>
                <div class="field">
                  <span class="field-label">Minecraft since</span>
                  <span class="field-val">{{ mcCreated ? fmtDate(mcCreated) : (extraLoading ? '…' : 'Unknown') }}</span>
                </div>
                <div class="field">
                  <span class="field-label">Skin model</span>
                  <span class="field-val">{{ player.skinModel === 'slim' ? 'Slim (Alex)' : 'Classic (Steve)' }}</span>
                </div>
                <div class="field">
                  <span class="field-label">XP</span>
                  <span v-if="beja" class="xp-val">{{ beja.xp.toLocaleString() }} <span class="xp-unit">XP</span></span>
                  <span v-else class="field-val">{{ extraLoading ? '…' : '—' }}</span>
                </div>
              </div>

              <div class="field">
                <span class="field-label">Equipped cape</span>
                <div v-if="capeThumbs.length" class="cape-row">
                  <div v-for="c in capeThumbs" :key="c.label" class="cape-item">
                    <img :src="c.thumb" :alt="c.label" class="cape-thumb" />
                    <span class="cape-label">{{ c.label }}</span>
                  </div>
                </div>
                <span v-else class="field-val">{{ extraLoading ? '…' : 'None' }}</span>
              </div>

              <div class="modal-actions">
                <button
                  class="btn-secondary"
                  :disabled="!player.skinUrl || saving"
                  @click="saveSkin"
                >{{ saveLabel }}</button>
                <button
                  class="btn-primary"
                  :disabled="requestSent"
                  @click="addFriend"
                >{{ requestSent ? 'Request Sent!' : 'Add Friend' }}</button>
              </div>

              <p v-if="saveResult" class="save-result">{{ saveResult }}</p>
            </div>

          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { PlayerProfile, BejaPlayerProfile } from '../../types'
import { useFriendsStore } from '../../store/friendsStore'
import HeroSkinViewer from '../skin/HeroSkinViewer.vue'

const props = defineProps<{
  modelValue: boolean
  player: PlayerProfile | null
}>()

const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>()

const friendsStore = useFriendsStore()

const copiedUuid   = ref(false)
const requestSent  = ref(false)
const saving       = ref(false)
const saveLabel    = ref('Save Skin')
const saveResult   = ref('')

const skinData      = ref('')
const modelCapeData = ref('')
const beja          = ref<BejaPlayerProfile | null>(null)
const mcCreated     = ref<string | null>(null)
const extraLoading  = ref(false)
const capeThumbs    = ref<{ label: string; thumb: string }[]>([])

watch(() => props.player, async player => {
  copiedUuid.value    = false
  requestSent.value   = false
  saving.value        = false
  saveLabel.value     = 'Save Skin'
  saveResult.value    = ''
  skinData.value      = ''
  modelCapeData.value = ''
  beja.value          = null
  mcCreated.value     = null
  capeThumbs.value    = []
  if (!player) return

  extraLoading.value = true
  // mc-heads serves the raw skin texture even for players without a Mojang profile
  const skinSrc = player.skinUrl ?? `https://mc-heads.net/skin/${player.uuid}`
  const [skin, vanillaCape, bejaProfile, created, serviceCapes] = await Promise.all([
    window.api.players.fetchImage(skinSrc),
    player.capeUrl ? window.api.players.fetchImage(player.capeUrl) : Promise.resolve(''),
    window.api.players.bejaProfile(player.uuid),
    window.api.players.mcCreated(player.uuid),
    window.api.players.capes(player.uuid),
  ])
  if (props.player !== player) return // modal switched player while loading

  skinData.value  = skin
  beja.value      = bejaProfile
  mcCreated.value = created

  let bejaCape = ''
  if (bejaProfile?.bejaCapeUrl) {
    bejaCape = await window.api.players.fetchImage(bejaProfile.bejaCapeUrl)
    if (props.player !== player) return
  }
  // BejaClient cape wins on the 3D model, vanilla as fallback
  modelCapeData.value = bejaCape || vanillaCape

  // Capes across services (capes.dev) + BejaClient. Vanilla comes from
  // capes.dev too; fall back to the session-server cape if it's missing.
  const thumbs: { label: string; thumb: string }[] = []
  for (const cape of serviceCapes) {
    const data = await window.api.players.fetchImage(cape.capeUrl)
    const t = data && await cropCapeFront(data)
    if (t) thumbs.push({ label: cape.service, thumb: t })
  }
  if (vanillaCape && !serviceCapes.some(c => c.service === 'Vanilla')) {
    const t = await cropCapeFront(vanillaCape)
    if (t) thumbs.unshift({ label: 'Vanilla', thumb: t })
  }
  if (bejaCape) {
    const t = await cropCapeFront(bejaCape)
    if (t) thumbs.push({ label: 'BejaClient', thumb: t })
  }
  if (props.player !== player) return
  capeThumbs.value   = thumbs
  extraLoading.value = false
}, { immediate: true })

// Front face of a cape texture lives at (1,1)-(11,17) at scale 1. Vanilla
// textures are 64x32 (2:1), OptiFine uses a 46x22 sheet — same pixel layout,
// different canvas size, so derive the scale from the matching base width.
function cropCapeFront(dataUrl: string): Promise<string> {
  return new Promise(resolve => {
    const img = new Image()
    img.onload = () => {
      const base = img.width / img.height === 2 ? 64 : 46
      const s = img.width / base
      const canvas = document.createElement('canvas')
      canvas.width  = 40
      canvas.height = 64
      const ctx = canvas.getContext('2d')!
      ctx.imageSmoothingEnabled = false
      ctx.drawImage(img, 1 * s, 1 * s, 10 * s, 16 * s, 0, 0, 40, 64)
      resolve(canvas.toDataURL())
    }
    img.onerror = () => resolve('')
    img.src = dataUrl
  })
}

function fmtDate(s: string): string {
  const d = new Date(s)
  if (isNaN(d.getTime())) return 'Unknown'
  return d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
}

function close() { emit('update:modelValue', false) }

function copyUuid() {
  if (!props.player) return
  navigator.clipboard.writeText(props.player.uuid)
  copiedUuid.value = true
  setTimeout(() => { copiedUuid.value = false }, 2000)
}

async function saveSkin() {
  if (!props.player?.skinUrl || saving.value) return
  saving.value    = true
  saveLabel.value = 'Saving…'
  try {
    const dest = await window.api.players.saveSkin(props.player.skinUrl, props.player.username)
    saveResult.value = `Saved to ${dest}`
    saveLabel.value  = 'Saved!'
  } catch {
    saveResult.value = 'Save failed.'
    saveLabel.value  = 'Save Skin'
    saving.value     = false
  }
}

async function addFriend() {
  if (!props.player || requestSent.value) return
  await friendsStore.sendRequest(props.player.username)
  requestSent.value = true
}
</script>

<style lang="scss" scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.modal {
  position: relative;
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius-lg;
  width: 640px;
  max-width: 92vw;
  overflow: hidden;
}

.close-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  background: $surface-elevated;
  border: 1px solid $border;
  border-radius: $radius;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: $text-secondary;
  transition: color $transition, background $transition;
  z-index: 1;
  &:hover { color: $text-primary; background: $border; }
}

.modal-body {
  display: flex;
  gap: 0;
}

// ── Skin column ───────────────────────────────────────────────────────────────
.skin-column {
  width: 220px;
  flex-shrink: 0;
  background: #0d0d0d;
  border-right: 1px solid $border;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 16px 8px 10px;
  min-height: 380px;
  position: relative;
}

.skin-viewer {
  width: 100%;
  height: 340px;
}

.drag-hint {
  font-size: 10px;
  color: $muted;
  letter-spacing: 0.05em;
  padding-bottom: 2px;
}

.skin-placeholder {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
}

.skin-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid $border;
  border-top-color: $text-secondary;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

// ── Info column ───────────────────────────────────────────────────────────────
.info-column {
  flex: 1;
  min-width: 0;
  padding: 26px 24px 22px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.player-name {
  font-size: 22px;
  font-weight: 700;
  color: $text-primary;
  margin: 0 0 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

// Same treatment as quest cards / leaderboard rows
.xp-val {
  font-size: 13px;
  font-weight: 800;
  color: $accent;
}

.xp-unit {
  font-family: 'Mojangles', monospace;
  font-size: 8px;
  letter-spacing: 0.1em;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.field-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px 16px;
}

.field-label {
  font-size: 11px;
  font-weight: 600;
  color: $muted;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.field-val {
  font-size: 13px;
  color: $text-secondary;
}

.uuid-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.uuid-val {
  font-size: 11px;
  color: $text-secondary;
  font-family: monospace;
  word-break: break-all;
  flex: 1;
}

.mini-btn {
  flex-shrink: 0;
  padding: 3px 10px;
  background: $surface-elevated;
  border: 1px solid $border;
  border-radius: $radius;
  color: $text-secondary;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: background $transition, color $transition;
  white-space: nowrap;
  &:hover { background: $border; color: $text-primary; }
}

// ── Capes ─────────────────────────────────────────────────────────────────────
.cape-row {
  display: flex;
  gap: 12px;
}

.cape-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.cape-thumb {
  width: 40px;
  height: 64px;
  image-rendering: pixelated;
  border-radius: 4px;
  border: 1px solid $border;
  background: #0d0d0d;
}

.cape-label {
  font-size: 10px;
  color: $muted;
}

.modal-actions {
  display: flex;
  gap: 8px;
  margin-top: auto;
  padding-top: 8px;
}

.btn-primary {
  flex: 1;
  padding: 9px 16px;
  background: $text-primary;
  color: $bg;
  border: none;
  border-radius: $radius;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background $transition;
  &:hover:not(:disabled) { background: $text-secondary; }
  &:disabled { opacity: 0.45; cursor: not-allowed; }
}

.btn-secondary {
  flex: 1;
  padding: 9px 16px;
  background: $surface-elevated;
  color: $text-primary;
  border: 1px solid $border;
  border-radius: $radius;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background $transition;
  &:hover:not(:disabled) { background: $border; }
  &:disabled { opacity: 0.45; cursor: not-allowed; }
}

.save-result {
  font-size: 11px;
  color: $muted;
  margin: 0;
  word-break: break-all;
}

// ── Transition ────────────────────────────────────────────────────────────────
.modal-fade-enter-active, .modal-fade-leave-active { transition: opacity 150ms ease; }
.modal-fade-enter-from,  .modal-fade-leave-to      { opacity: 0; }
</style>
