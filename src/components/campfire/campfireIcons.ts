// 8x8 pixel icons for the villager's trade bubbles (original pixel art, drawn here as rects in an SVG).

export type IconName = 'emerald' | 'bread' | 'apple' | 'coal' | 'sword' | 'wheat'

const PALETTES: Record<IconName, { rows: string[]; colors: Record<string, string> }> = {
  emerald: {
    rows: [
      '........',
      '...dd...',
      '..dlgd..',
      '.dlggGd.',
      '.dgggGd.',
      '..dgGd..',
      '...dd...',
      '........',
    ],
    colors: { d: '#0b6b2c', l: '#8dffb0', g: '#17c453', G: '#0f9a3e' },
  },
  bread: {
    rows: [
      '........',
      '........',
      '..bbbb..',
      '.bLbLbb.',
      '.bbbbbb.',
      '.dbbbbd.',
      '..dddd..',
      '........',
    ],
    colors: { b: '#c98a3c', L: '#e6b562', d: '#8a5a22' },
  },
  apple: {
    rows: [
      '.....s..',
      '....s...',
      '..rrrr..',
      '.rLrrrr.',
      '.rrrrrr.',
      '.rrrrrr.',
      '..rrrr..',
      '........',
    ],
    colors: { r: '#c8372d', L: '#f08a7e', s: '#4b8a2e' },
  },
  coal: {
    rows: [
      '........',
      '..kkk...',
      '.kKkkk..',
      '.kkkkkk.',
      '.kkkkKk.',
      '..kkkk..',
      '........',
      '........',
    ],
    colors: { k: '#26262b', K: '#4a4a52' },
  },
  sword: {
    rows: [
      '......ww',
      '.....www',
      '....www.',
      '...www..',
      '.b.ww...',
      '..bw....',
      '.bb.b...',
      'b.......',
    ],
    colors: { w: '#cfd8e0', b: '#8a5a22' },
  },
  wheat: {
    rows: [
      '...y....',
      '..yyy.y.',
      '.yyy.yy.',
      '..y.yyy.',
      '.yyyyy..',
      '...y....',
      '...g....',
      '...g....',
    ],
    colors: { y: '#e0c040', g: '#6a8a2e' },
  },
}

export interface IconRect { x: number; y: number; c: string }

export function iconRects(name: IconName): IconRect[] {
  const { rows, colors } = PALETTES[name]
  const out: IconRect[] = []
  rows.forEach((row, y) => [...row].forEach((ch, x) => { if (ch !== '.') out.push({ x, y, c: colors[ch] }) }))
  return out
}

export const TRADES: { give: IconName; get: IconName }[] = [
  { give: 'wheat', get: 'emerald' },
  { give: 'emerald', get: 'bread' },
  { give: 'apple', get: 'emerald' },
  { give: 'coal', get: 'emerald' },
  { give: 'emerald', get: 'sword' },
]
