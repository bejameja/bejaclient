export type Rarity = 'leather' | 'gold' | 'iron' | 'diamond' | 'netherite' | 'enchanted'
export type CosmeticType = 'cape' | 'hat' | 'wings' | 'pet' | 'trail' | 'kill_effect' | 'win_effect' | 'emote' | 'accessory'

export interface RarityInfo {
  label: string
  color: string
  bg: string
  glow: string
  animated?: boolean
}

export const RARITIES: Record<Rarity, RarityInfo> = {
  leather:   { label: 'Leather',   color: '#A89070', bg: 'rgba(168,144,112,0.14)', glow: 'rgba(168,144,112,0.28)' },
  gold:      { label: 'Gold',      color: '#FFB300', bg: 'rgba(255,179,0,0.13)',   glow: 'rgba(255,179,0,0.40)'   },
  iron:      { label: 'Iron',      color: '#C8C8C8', bg: 'rgba(200,200,200,0.11)', glow: 'rgba(200,200,200,0.28)' },
  diamond:   { label: 'Diamond',   color: '#4AFFE0', bg: 'rgba(74,255,224,0.11)',  glow: 'rgba(74,255,224,0.40)'  },
  netherite: { label: 'Netherite', color: '#9B59B6', bg: 'rgba(155,89,182,0.14)', glow: 'rgba(155,89,182,0.44)'  },
  enchanted: { label: 'Enchanted', color: '#FF88FF', bg: 'rgba(255,136,255,0.11)', glow: 'rgba(255,136,255,0.54)', animated: true },
}

export const RARITY_ORDER: Rarity[] = ['leather', 'iron', 'gold', 'diamond', 'netherite', 'enchanted']

export interface Cosmetic {
  id: string
  name: string
  type: CosmeticType
  rarity: Rarity
  model_url?: string
  preview_url?: string
  description?: string
}

export interface PlayerCosmetic extends Cosmetic {
  equipped: boolean
}

export interface CrateType {
  id: string
  name: string
  description: string
  image_url?: string
}

export interface CrateResult {
  cosmetic: Cosmetic
  is_new: boolean
}
