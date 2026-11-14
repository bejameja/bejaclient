// All launcher UI sounds are switched off (hover, click, toggle, launch, success). The exports stay as no-ops so existing
// imports keep working; nothing creates an AudioContext or loads audio any more. (Voice chat audio is separate: useLobbyVoice.ts.)

export function warmAudio(): void {}
export function playHover(): void {}
export function playClick(): void {}
export function playMouseClick(): void {}
export function playLaunch(): void {}
export function playToggle(): void {}
export function playSuccess(): void {}

export function useSounds() {
  return { playHover, playClick, playMouseClick, playLaunch, playSuccess, playToggle }
}
