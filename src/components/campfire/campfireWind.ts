import { BufferAttribute, type BufferGeometry, type Material } from 'three'

// Wind for the campfire scene's plants, done in the vertex shader (Three 0.136, onBeforeCompile).
//
// What the glTF contains (checked against the real file):
//  - Short_Grass, Bush, Dandelion, Azure_Bluet, Oxeye_Daisy, Cornflower, Tall_Grass: every plant is two
//    crossed quads (diagonal, 0.9 x 0.9), each quad exactly 1 block high with its base on a block edge and
//    its own 4 vertices (vertex i belongs to quad i / 4). One mesh holds all plants of a type.
//  - Tall_Grass is two blocks tall: the lower and upper halves are separate quads, the upper one starts
//    where the lower one ends.
//  - Oak_Leaves: loose cube faces (4 own vertices each), no plants.
//  - Glow_Lichen is a wall decal and is left static.
//
// Per vertex we store `aSway` = height above the plant's base in blocks (0 at the root, 1 at the tip of a
// one-block plant, up to 2 for tall grass - the upper half continues where the lower half ends, so the two
// halves never tear apart). Leaves use a constant 1 and a phase taken from the exact vertex position, so the
// shared corners of neighbouring faces always move identically.

/** Shared clock, advanced by the scene. */
export const windTime = { value: 0 }

export type WindKind = 'grass' | 'tallGrass' | 'flower' | 'bush' | 'leaves'

interface WindParams { slow: number; flutter: number; cell: number }

// Amplitudes in blocks (1 pixel of a 16px texture = 0.0625): kept around one pixel so the nearest-filtered
// textures are never visibly distorted.
const PARAMS: Record<WindKind, WindParams> = {
  grass: { slow: 0.07, flutter: 0.018, cell: 1 },
  tallGrass: { slow: 0.045, flutter: 0.012, cell: 1 },   // multiplied by the larger sway of the upper half
  flower: { slow: 0.05, flutter: 0.015, cell: 1 },
  bush: { slow: 0.03, flutter: 0.01, cell: 1 },
  leaves: { slow: 0.02, flutter: 0.008, cell: 0 },
}

export function windKindFor(name: string): WindKind | null {
  if (/lichen/i.test(name)) return null
  if (/leaves/i.test(name)) return 'leaves'
  if (/tall_grass/i.test(name)) return 'tallGrass'
  if (/short_grass|^grass$/i.test(name)) return 'grass'
  if (/bush/i.test(name)) return 'bush'
  if (/dandelion|bluet|daisy|cornflower/i.test(name)) return 'flower'
  return null
}

/** Adds the `aSway` attribute (see above). Quads are 4 consecutive vertices. */
export function addSwayAttribute(geometry: BufferGeometry, kind: WindKind): void {
  const pos = geometry.getAttribute('position')
  const n = pos.count
  const sway = new Float32Array(n)

  if (kind === 'leaves' || n % 4 !== 0) {
    sway.fill(1)
  } else {
    const quads = n / 4
    const ymin = new Float32Array(quads)
    const key: string[] = new Array(quads)
    for (let q = 0; q < quads; q++) {
      let lo = Infinity, cx = 0, cz = 0
      for (let k = 0; k < 4; k++) {
        lo = Math.min(lo, pos.getY(q * 4 + k))
        cx += pos.getX(q * 4 + k)
        cz += pos.getZ(q * 4 + k)
      }
      ymin[q] = lo
      key[q] = `${Math.round((cx / 4) * 100)}|${Math.round((cz / 4) * 100)}`
    }
    // quad whose base sits one block above another quad's base (same column) is an upper half
    const byBase = new Map<string, number>()
    for (let q = 0; q < quads; q++) byBase.set(`${key[q]}|${Math.round(ymin[q] * 100)}`, q)
    const offset = new Int8Array(quads).fill(-1)
    const stack = (q: number): number => {
      if (offset[q] >= 0) return offset[q]
      const below = byBase.get(`${key[q]}|${Math.round((ymin[q] - 1) * 100)}`)
      return (offset[q] = below === undefined ? 0 : stack(below) + 1)
    }
    for (let q = 0; q < quads; q++) {
      const off = stack(q)
      for (let k = 0; k < 4; k++) sway[q * 4 + k] = pos.getY(q * 4 + k) - ymin[q] + off
    }
  }
  geometry.setAttribute('aSway', new BufferAttribute(sway, 1))
}

/** Patches a MeshStandardMaterial so its vertices sway with the wind. */
export function applyWind(material: Material, kind: WindKind): void {
  const p = PARAMS[kind]
  material.customProgramCacheKey = () => 'campfire-wind'
  material.onBeforeCompile = (shader) => {
    shader.uniforms.uTime = windTime
    shader.uniforms.uAmpSlow = { value: p.slow }
    shader.uniforms.uAmpFlutter = { value: p.flutter }
    shader.uniforms.uCell = { value: p.cell }
    shader.vertexShader = shader.vertexShader
      .replace('#include <common>', `#include <common>
attribute float aSway;
uniform float uTime;
uniform float uAmpSlow;
uniform float uAmpFlutter;
uniform float uCell;`)
      .replace('#include <begin_vertex>', `#include <begin_vertex>
{
  vec3 wp = (modelMatrix * vec4(position, 1.0)).xyz;
  // plants sway as a whole per block (phase from the block), leaves from their exact position
  vec2 ph = mix(wp.xz, floor(wp.xz) + 0.5, uCell);
  // slow wave rolling across the meadow, plus a faster small flutter
  float slow = 0.6 * sin(uTime * 0.9 + ph.x * 0.35 + ph.y * 0.22) + 0.4 * sin(uTime * 0.37 + ph.x * 0.11 - ph.y * 0.17);
  float flut = sin(uTime * 4.1 + ph.x * 2.3 + ph.y * 1.9) * sin(uTime * 2.7 + ph.y * 3.1 + ph.x * 0.7);
  float d = (slow * uAmpSlow + flut * uAmpFlutter) * pow(aSway, 1.4);
  transformed.x += d * 0.8;
  transformed.z += d * 0.6;
}`)
  }
  material.needsUpdate = true
}
