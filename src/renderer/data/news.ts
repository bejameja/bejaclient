export interface NewsEntry {
  id: string
  title: string
  body: string
  date: string
  tag: 'Release' | 'Update' | 'Feature' | 'Fix' | 'Content' | 'Event'
}

// Edit this array to update the pinboard on the home screen.
// Max 5 entries shown; add as many as you like — only the first 5 appear.
export const bejaNews: NewsEntry[] = [
  {
    id: 'v1.0',
    title: 'BejaClient v1.0 Released',
    body: 'The first stable release is live. Vanilla & Fabric fully supported.',
    date: 'Apr 21, 2026',
    tag: 'Release',
  },
  {
    id: 'mods-browser',
    title: 'Mods Browser Added',
    body: 'Search and install Modrinth mods directly from the launcher.',
    date: 'Apr 18, 2026',
    tag: 'Feature',
  },
  {
    id: 'launch-speed',
    title: 'Faster Launch Times',
    body: 'Optimised Java detection — launches are now up to 40% faster.',
    date: 'Apr 14, 2026',
    tag: 'Fix',
  },
  {
    id: 'skin-preview',
    title: 'Live Skin Previews',
    body: 'Your Minecraft skin is now shown live in the sidebar and home room.',
    date: 'Apr 10, 2026',
    tag: 'Content',
  },
  {
    id: 'forge-support',
    title: 'Forge & NeoForge Support',
    body: 'Forge and NeoForge profiles work alongside Fabric and Quilt.',
    date: 'Apr 5, 2026',
    tag: 'Update',
  },
]
