import { CanvasTexture, NearestFilter, sRGBEncoding } from 'three'

// Tiny runtime-drawn pixel textures for the dog and the villager (original art, no vanilla textures).

export function rand(seed: number) {
  let s = seed >>> 0
  return () => { s = (s * 1664525 + 1013904223) >>> 0; return s / 4294967296 }
}

export function pixelTexture(w: number, h: number, seed: number, paint: (g: CanvasRenderingContext2D, r: () => number) => void): CanvasTexture {
  const c = document.createElement('canvas')
  c.width = w
  c.height = h
  const g = c.getContext('2d')!
  paint(g, rand(seed))
  const t = new CanvasTexture(c)
  t.magFilter = NearestFilter
  t.minFilter = NearestFilter
  t.encoding = sRGBEncoding
  return t
}

/** Solid colour with a few lighter / darker pixels, so it reads as material and not as a flat box. */
export function furTexture(base: string, light: string, dark: string, seed: number, size = 8): CanvasTexture {
  return pixelTexture(size, size, seed, (g, r) => {
    g.fillStyle = base
    g.fillRect(0, 0, size, size)
    for (let i = 0; i < size * 2; i++) {
      g.fillStyle = r() < 0.5 ? light : dark
      g.fillRect(Math.floor(r() * size), Math.floor(r() * size), 1, 1)
    }
  })
}
