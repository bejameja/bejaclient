// Separate module so its top-level state survives Vue component
// remounts (KeepAlive cache eviction) but resets on a real app restart.
let played = false

export function consumeHubIntro(): boolean {
  if (played) return false
  played = true
  return true
}
