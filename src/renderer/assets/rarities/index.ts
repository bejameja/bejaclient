import leatherIcon   from './leather.png'
import ironIcon      from './iron.webp'
import goldIcon      from './gold.png'
import diamondIcon   from './diamond.png'
import netheriteIcon from './netherite.png'
import type { Rarity } from '../../types/cosmetics'

export const RARITY_ICONS: Partial<Record<Rarity, string>> = {
  leather:   leatherIcon,
  iron:      ironIcon,
  gold:      goldIcon,
  diamond:   diamondIcon,
  netherite: netheriteIcon,
}
