// The microphone the lobby voice chat uses. Chosen in Settings, remembered per launcher profile (localStorage), applied
// to a running voice chat through VOICE_INPUT_EVENT.

const KEY = 'beja_voice_input_device'
export const VOICE_INPUT_EVENT = 'beja-voice-input-changed'

/** The chosen device id, or '' for the system default. */
export function getVoiceInputId(): string {
  try { return localStorage.getItem(KEY) ?? '' } catch { return '' }
}

export function setVoiceInputId(id: string): void {
  try {
    if (id) localStorage.setItem(KEY, id)
    else localStorage.removeItem(KEY)
  } catch { /* storage unavailable: the choice just does not persist */ }
  window.dispatchEvent(new Event(VOICE_INPUT_EVENT))
}

/** Opens the chosen microphone; falls back to the system default when that device is gone (unplugged). */
export async function openMicStream(): Promise<MediaStream> {
  const id = getVoiceInputId()
  if (id) {
    try {
      return await navigator.mediaDevices.getUserMedia({ audio: { deviceId: { exact: id } }, video: false })
    } catch { /* fall through to the default device */ }
  }
  return navigator.mediaDevices.getUserMedia({ audio: true, video: false })
}
