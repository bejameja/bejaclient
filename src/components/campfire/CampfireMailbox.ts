import { BoxGeometry, Group, Mesh, MeshStandardMaterial, Vector3, type BufferGeometry, type Material, type Object3D, type Texture } from 'three'
import { pixelTexture } from './campfirePixels'

// A block-style mailbox on a wooden post, built from boxes (original geometry and pixel textures, 1 unit = 1 block,
// 1 px = 1/16 block). It carries the launcher's notifications: the red flag stands up while something is unread, and
// clicking the box opens the notifications drawer.

const PX = 1 / 16

function wood(w: number, h: number, seed: number): Texture {
  return pixelTexture(w, h, seed, (g, r) => {
    g.fillStyle = '#b88a55'
    g.fillRect(0, 0, w, h)
    for (let x = 0; x < w; x++) {   // vertical grain streaks
      if (r() < 0.45) { g.fillStyle = r() < 0.5 ? '#a97a46' : '#c99b63'; g.fillRect(x, 0, 1, h) }
    }
    for (let i = 0; i < w * h * 0.12; i++) {
      g.fillStyle = r() < 0.6 ? '#9a6c3c' : '#d3a870'
      g.fillRect(Math.floor(r() * w), Math.floor(r() * h), 1, 1)
    }
  })
}

function paint(base: string, a: string, b: string, size: number, seed: number): Texture {
  return pixelTexture(size, size, seed, (g, r) => {
    g.fillStyle = base
    g.fillRect(0, 0, size, size)
    for (let i = 0; i < size * size * 0.18; i++) {
      g.fillStyle = r() < 0.5 ? a : b
      g.fillRect(Math.floor(r() * size), Math.floor(r() * size), 1, 1)
    }
  })
}

export class CampfireMailbox {
  readonly group = new Group()
  /** Meshes to raycast against / outline. */
  readonly pickables: Object3D[] = []
  private flag = new Group()
  private letters: Mesh[] = []
  private geoms: BufferGeometry[] = []
  private mats: Material[] = []
  private texs: Texture[] = []
  private unread = 0
  private t = 0

  constructor(private pos: Vector3, private yaw: number, private scale: number) {
    this.group.visible = false
    this.build()
  }

  private mat(tex: Texture): MeshStandardMaterial {
    const m = new MeshStandardMaterial({ map: tex, roughness: 1, metalness: 0 })
    this.texs.push(tex)
    this.mats.push(m)
    return m
  }

  /** Box in pixels: size (w,h,d) and centre (x,y,z) relative to the post's foot. */
  private box(parent: Object3D, m: Material, w: number, h: number, d: number, x: number, y: number, z: number): Mesh {
    const geo = new BoxGeometry(w * PX, h * PX, d * PX)
    this.geoms.push(geo)
    const mesh = new Mesh(geo, m)
    mesh.position.set(x * PX, y * PX, z * PX)
    mesh.castShadow = true
    mesh.receiveShadow = true
    parent.add(mesh)
    this.pickables.push(mesh)
    return mesh
  }

  private build() {
    const postM = this.mat(wood(4, 32, 11))
    const beamM = this.mat(wood(16, 4, 12))
    const whiteM = this.mat(paint('#efece4', '#e1ded5', '#f8f6f0', 16, 13))
    const lidM = this.mat(paint('#f5f3ec', '#e6e3da', '#ffffff', 16, 14))
    const redM = this.mat(paint('#d11f1f', '#b81818', '#e43a3a', 8, 15))
    const darkM = this.mat(paint('#4a4a4e', '#3c3c40', '#58585c', 8, 16))

    // post, cap, support platform, diagonal strut
    this.box(this.group, postM, 3, 32, 3, 0, 16, 0)
    this.box(this.group, beamM, 6, 2, 6, 0, 33, 0)
    this.box(this.group, beamM, 20, 1.5, 6, 6, 22.75, 0)
    const strut = this.box(this.group, beamM, 13, 1.5, 3, 4.6, 18.5, 0)
    strut.rotation.z = Math.PI / 4

    // the box: body, a slightly wider lid, and a dark door seam at the front
    this.box(this.group, whiteM, 14, 6, 8, 8.5, 26.5, 0)
    this.box(this.group, lidM, 15, 2.5, 9, 8.5, 30.25, 0)
    this.box(this.group, darkM, 0.5, 6.5, 7.5, 15.75, 27, 0)

    // red flag on the side: pivot at its foot, stands up when unread, lies back when not
    this.flag.position.set(12.5 * PX, 28 * PX, 4.8 * PX)
    this.box(this.flag, redM, 2, 7, 1, 0, 3, 0)
    this.box(this.flag, redM, 4, 2.5, 1, 1, 6.2, 0)
    this.group.add(this.flag)

    // the letter lying on the ground in front of the box while something is unread
    const envM = this.mat(pixelTexture(8, 6, 17, (g, r) => {
      g.fillStyle = '#f3efe3'
      g.fillRect(0, 0, 8, 6)
      g.fillStyle = '#d9d4c4'   // flap lines
      for (let i = 0; i < 4; i++) { g.fillRect(i, i, 1, 1); g.fillRect(7 - i, i, 1, 1) }
      g.fillStyle = '#d11f1f'   // seal
      g.fillRect(3, 3, 2, 2)
      if (r() < 0) g.fillRect(0, 0, 1, 1)
    }))
    // eight envelopes in a pool; layoutLetters() decides which are shown and where (they sit above the ground surface
    // so they never z-fight with it)
    for (let i = 0; i < 8; i++) this.letters.push(this.box(this.group, envM, 12, 1.2, 9, 0, 0.9, 0))
    this.applyUnread()
  }

  async load(groundY: (x: number, z: number) => number): Promise<void> {
    this.group.scale.setScalar(this.scale)
    this.group.rotation.y = this.yaw
    this.group.position.set(this.pos.x, groundY(this.pos.x, this.pos.z), this.pos.z)
    this.group.visible = true
  }

  /** World point above the box, for the count badge. */
  badgeWorld(out: Vector3): Vector3 {
    return out.set(this.group.position.x, this.group.position.y + 36 * PX * this.scale + 0.25, this.group.position.z)
  }

  setUnread(n: number) {
    this.unread = n
    this.applyUnread()
  }

  private applyUnread() {
    this.flag.rotation.z = this.unread > 0 ? 0 : Math.PI / 2   // up, or folded back along the lid
    this.layoutLetters()
  }

  /** 1-3 unread: that many loose letters. More than 3: a messy pile (about a stack and a half) plus two strays. */
  private layoutLetters() {
    const n = this.unread
    const loose: [number, number, number][] = [[-4, 22, 0.3], [9, 26, -0.5], [2, 33, 0.9]]   // x, z (px), yaw
    // pile: x, z, yaw, layer
    const pile: [number, number, number, number][] = [
      [3, 24, 0.2, 0], [4.5, 25, -0.5, 1], [2.2, 23.4, 0.9, 2], [4, 24.6, -0.2, 3], [2.8, 25.4, 0.6, 4], [3.6, 24.2, -0.8, 5],
      [-9, 30, 1.2, 0], [13, 32, -0.3, 0],
    ]
    this.letters.forEach((m, i) => {
      if (n <= 0 || (n <= 3 && i >= n) ) { m.visible = false; return }
      m.visible = true
      if (n <= 3) {
        const [x, z, yaw] = loose[i]
        m.position.set(x * PX, 0.9 * PX, z * PX)
        m.rotation.y = yaw
      } else {
        const [x, z, yaw, layer] = pile[i]
        m.position.set(x * PX, (0.9 + layer * 1.2) * PX, z * PX)
        m.rotation.y = yaw
      }
    })
  }

  update(dt: number) {
    this.t += dt
    if (this.unread > 0) this.flag.rotation.z = Math.floor(this.t * 2) % 2 ? 0.08 : 0   // a two-step wiggle
  }

  dispose() {
    this.group.removeFromParent()
    this.pickables.length = 0
    for (const g of this.geoms) g.dispose()
    for (const m of this.mats) m.dispose()
    for (const t of this.texs) t.dispose()
  }
}
