import { BufferAttribute, BufferGeometry, Color, Points, PointsMaterial, type Material } from 'three'

// Christmas theme: a snow cover on every upward-facing block face, and pixel snowflakes drifting down.

/**
 * Whitens the top faces of a standard material, per 1/16-block pixel, so the snow looks like it was painted
 * on the block texture: sides keep their texture, tops get an uneven white coat.
 */
export function applySnowCover(mat: Material, coverage = 0.82) {
  const m = mat as any
  if (m.userData?.snowCover) return
  m.userData = { ...(m.userData ?? {}), snowCover: true }
  const prev = m.onBeforeCompile
  m.onBeforeCompile = (shader: any, renderer: any) => {
    prev?.call(m, shader, renderer)
    shader.uniforms.uSnowCoverage = { value: coverage }
    shader.vertexShader = shader.vertexShader
      .replace('#include <common>', '#include <common>\nvarying vec3 vSnowPos;')
      .replace('#include <begin_vertex>', '#include <begin_vertex>\nvSnowPos = (modelMatrix * vec4(transformed, 1.0)).xyz;')
    shader.fragmentShader = shader.fragmentShader
      .replace('#include <common>', `#include <common>
varying vec3 vSnowPos;
uniform float uSnowCoverage;
float snowHash(vec2 p) {
  p = fract(p * vec2(123.34, 456.21));
  p += dot(p, p + 45.32);
  return fract(p.x * p.y);
}`)
      .replace('#include <normal_fragment_maps>', `#include <normal_fragment_maps>
{
  vec3 snowUp = normalize((viewMatrix * vec4(0.0, 1.0, 0.0, 0.0)).xyz);
  float facing = smoothstep(0.55, 0.85, dot(normal, snowUp));
  vec2 cell = floor(vSnowPos.xz * 16.0);
  float n = snowHash(cell);
  float clump = snowHash(floor(vSnowPos.xz * 4.0) + 7.0);
  float cover = facing * step(1.0 - uSnowCoverage, 0.35 * n + 0.65 * clump);
  vec3 snowCol = vec3(0.90, 0.94, 1.0) * (0.9 + 0.1 * n);
  diffuseColor.rgb = mix(diffuseColor.rgb, snowCol, cover);
}`)
  }
  m.customProgramCacheKey = () => 'snow-cover'
  m.needsUpdate = true
}

export interface Snowfall {
  points: Points
  update(dt: number, time: number): void
  dispose(): void
}

/** Pixel-square flakes in a box over the plateau; each falls, sways, and wraps back to the top. */
export function createSnowfall(box: { x0: number; x1: number; z0: number; z1: number; yBottom: number; yTop: number }, count = 600): Snowfall {
  const pos = new Float32Array(count * 3)
  const speed = new Float32Array(count)
  const phase = new Float32Array(count)
  const w = box.x1 - box.x0, d = box.z1 - box.z0, h = box.yTop - box.yBottom
  for (let i = 0; i < count; i++) {
    pos[i * 3] = box.x0 + Math.random() * w
    pos[i * 3 + 1] = box.yBottom + Math.random() * h
    pos[i * 3 + 2] = box.z0 + Math.random() * d
    speed[i] = 1.1 + Math.random() * 1.3
    phase[i] = Math.random() * Math.PI * 2
  }
  const geo = new BufferGeometry()
  const attr = new BufferAttribute(pos, 3)
  geo.setAttribute('position', attr)
  const mat = new PointsMaterial({ color: new Color(1, 1, 1), size: 0.07, sizeAttenuation: true, transparent: true, opacity: 0.95, depthWrite: false })
  const points = new Points(geo, mat)
  points.frustumCulled = false
  return {
    points,
    update(dt, time) {
      for (let i = 0; i < count; i++) {
        const k = i * 3
        pos[k + 1] -= speed[i] * dt
        pos[k] += Math.sin(time * 0.9 + phase[i]) * 0.35 * dt + 0.25 * dt   // a light drift
        pos[k + 2] += Math.cos(time * 0.7 + phase[i]) * 0.25 * dt
        if (pos[k + 1] < box.yBottom) {
          pos[k + 1] = box.yTop
          pos[k] = box.x0 + Math.random() * w
          pos[k + 2] = box.z0 + Math.random() * d
        }
        if (pos[k] > box.x1) pos[k] = box.x0
      }
      attr.needsUpdate = true
    },
    dispose() { geo.dispose(); mat.dispose(); points.removeFromParent() },
  }
}
