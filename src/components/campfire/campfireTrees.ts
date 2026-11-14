import { BoxGeometry, CanvasTexture, Group, InstancedMesh, Matrix4, Mesh, MeshStandardMaterial, NearestFilter, PlaneGeometry, RepeatWrapping, sRGBEncoding, type Material } from 'three'

// Blocky oak trees on the terrain behind the fire. The glTF is a cut-out of a forest chunk, and from the hub camera the
// view over the fire plateau ends in a bare edge; trees standing out there give the background some depth.
// They reuse the scene's own oak log / oak leaves materials, so they look like the rest of the world.

export interface TreeOptions {
  logMaterial: Material
  leafMaterial: Material
  /** Top of the terrain at (x, z), or null where there is none (the model ends). */
  groundTop: (x: number, z: number) => number | null
  /** Where to try to put a tree (xz, in blocks); each spot is jittered a little and some are skipped. */
  positions: [number, number][]
  /** The spot to keep clear around the fire. */
  keepClear: { x: number; z: number; r: number }
  seed?: number
}

function rng(seed: number): () => number {
  let a = seed >>> 0
  return () => {
    a = (a + 0x6d2b79f5) >>> 0
    let t = a
    t = Math.imul(t ^ (t >>> 15), t | 1)
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

export function buildTrees(o: TreeOptions): { group: Group; dispose(): void } {
  const r = rng(o.seed ?? 1337)
  const logs: Matrix4[] = []
  const leaves: Matrix4[] = []
  const put = (list: Matrix4[], x: number, y: number, z: number) => list.push(new Matrix4().makeTranslation(x + 0.5, y + 0.5, z + 0.5))

  for (const [px, pz] of o.positions) {
    {
      const x = Math.floor(px + (r() - 0.5) * 2)
      const z = Math.floor(pz + (r() - 0.5) * 2)
      if (r() < 0.40) continue   // gaps, so it does not look like an orchard
      if (Math.hypot(x - o.keepClear.x, z - o.keepClear.z) < o.keepClear.r) continue
      const top = o.groundTop(x + 0.5, z + 0.5)
      if (top === null) continue
      const base = Math.round(top)
      const h = 3 + Math.floor(r() * 2)                       // trunk 3..4 (low enough to leave the moon free)
      for (let i = 0; i < h; i++) put(logs, x, base + i, z)
      // canopy: two wide layers just under the top, then a narrower one and a cap
      const layers: [number, number][] = [[h - 2, 2], [h - 1, 2], [h, 1], [h + 1, 1]]
      for (const [dy, rad] of layers) {
        for (let dx = -rad; dx <= rad; dx++) {
          for (let dz = -rad; dz <= rad; dz++) {
            if (rad === 2 && Math.abs(dx) === 2 && Math.abs(dz) === 2 && r() < 0.7) continue     // rounded corners
            if (rad === 1 && dy === h + 1 && Math.abs(dx) + Math.abs(dz) > 1) continue          // plus-shaped cap
            if (dx === 0 && dz === 0 && dy < h) continue                                         // trunk
            put(leaves, x + dx, base + dy, z + dz)
          }
        }
      }
    }
  }

  const geo = new BoxGeometry(1, 1, 1)
  const group = new Group()
  const make = (list: Matrix4[], mat: Material, cast: boolean) => {
    if (!list.length) return
    const m = new InstancedMesh(geo, mat, list.length)
    list.forEach((mx, i) => m.setMatrixAt(i, mx))
    m.instanceMatrix.needsUpdate = true
    m.castShadow = cast
    m.receiveShadow = false   // the moon shadow map only covers the fire area: out here it would cut off in hard edges
    m.frustumCulled = false
    group.add(m)
  }
  make(logs, o.logMaterial, false)
  make(leaves, o.leafMaterial, false)
  return { group, dispose: () => geo.dispose() }
}

/**
 * A flat grass plain a little below the plateau's level, out where the model's terrain drops away or ends, so the
 * trees stand on something. It is hidden under the real terrain wherever that reaches the plateau level.
 */
export function buildFarGround(centre: { x: number; z: number }, size: { w: number; l: number }, y: number, angle: number): { mesh: Mesh; dispose(): void } {
  const c = document.createElement('canvas')
  c.width = c.height = 16
  const g = c.getContext('2d')!
  let seed = 7
  const rnd = () => { seed = (seed * 16807) % 2147483647; return seed / 2147483647 }
  for (let yy = 0; yy < 16; yy++) {
    for (let xx = 0; xx < 16; xx++) {
      const k = rnd()
      g.fillStyle = k < 0.33 ? '#243f26' : k < 0.66 ? '#2b4a2d' : '#1e3622'
      g.fillRect(xx, yy, 1, 1)
    }
  }
  const tex = new CanvasTexture(c)
  tex.wrapS = tex.wrapT = RepeatWrapping
  tex.magFilter = tex.minFilter = NearestFilter
  tex.encoding = sRGBEncoding
  tex.repeat.set(size.w, size.l)
  const mat = new MeshStandardMaterial({ map: tex, roughness: 1, metalness: 0 })
  const geo = new PlaneGeometry(size.w, size.l)
  const mesh = new Mesh(geo, mat)
  mesh.rotation.x = -Math.PI / 2
  mesh.rotation.z = angle
  mesh.position.set(centre.x, y, centre.z)
  mesh.receiveShadow = false
  return { mesh, dispose: () => { geo.dispose(); mat.dispose(); tex.dispose() } }
}
