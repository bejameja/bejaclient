import { BackSide, Mesh, ShaderMaterial, SphereGeometry, Vector3 } from 'three'

// Night sky for the campfire scene: a gradient dome with blocky (pixel-grid) stars that twinkle, a square
// pixel-art moon with a soft halo, drifting square clouds, a blocky far horizon, and an occasional shooting star. Everything is procedural in one fragment
// shader. Values go above 1.0 on purpose (moon, bright stars): the bloom pass picks those up.

export const SKY_COLOR = 0x0a1126   // also the fog colour, so distant terrain melts into the horizon

// The hub camera looks along the glTF's camera marker; place things relative to that heading.
const FORWARD_AZ = Math.atan2(-8.31, 13.875)

export function azEl(az: number, el: number): Vector3 {
  return new Vector3(Math.cos(el) * Math.cos(az), Math.sin(el), Math.cos(el) * Math.sin(az))
}

/** Where the moon is drawn: left of centre above the low part of the skyline (the hub card is wide and short,
 *  so only the first ~6 degrees above the horizon are visible). */
const MOON_AZ = FORWARD_AZ - 0.22
export const MOON_DIR = azEl(MOON_AZ, 0.055)
/** Direction moonlight comes from (higher than the drawn moon so shadows stay short). */
export const MOONLIGHT_DIR = azEl(MOON_AZ, 0.62)

const VERT = /* glsl */ `
varying vec3 vDir;
void main() {
  vDir = position;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}`

const FRAG = /* glsl */ `
varying vec3 vDir;
uniform float uTime;
uniform vec3 uHorizon;
uniform vec3 uMoonDir;
uniform float uFwdAz;
uniform float uShootT0;
uniform float uShootSeed;

float hash21(vec2 p) {
  p = fract(p * vec2(123.34, 456.21));
  p += dot(p, p + 45.32);
  return fract(p.x * p.y);
}

float vnoise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float a = hash21(i);
  float b = hash21(i + vec2(1.0, 0.0));
  float c = hash21(i + vec2(0.0, 1.0));
  float d = hash21(i + vec2(1.0, 1.0));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

float fbm(vec2 p, int oct) {
  float s = 0.0;
  float a = 0.5;
  mat2 m = mat2(1.6, 1.2, -1.2, 1.6);
  for (int i = 0; i < 5; i++) {
    if (i >= oct) break;
    s += a * vnoise(p);
    p = m * p;
    a *= 0.5;
  }
  return s;
}

// Minecraft-style clouds: one flat square per grid cell, hard edges, no blending. A cell is cloud where a slow
// fbm pattern (sampled per cell, not per pixel) is above a threshold.
float mcCloud(vec2 cell) {
  return step(0.42, fbm(cell * 0.09 + vec2(3.0, 7.0), 4));
}

void main() {
  vec3 dir = normalize(vDir);
  float h = clamp(dir.y, 0.0, 1.0);
  float az = atan(dir.z, dir.x);
  float el = asin(clamp(dir.y, -1.0, 1.0));

  // gradient: horizon colour (= fog) -> deep blue -> near black at the zenith
  vec3 zenith = vec3(0.002, 0.004, 0.016);
  vec3 mid = vec3(0.009, 0.017, 0.055);
  vec3 col = mix(uHorizon, mid, smoothstep(0.0, 0.11, h));
  col = mix(col, zenith, smoothstep(0.10, 0.85, h));
  // faint violet haze hugging the horizon
  col += vec3(0.022, 0.012, 0.035) * exp(-h * 38.0);

  float haze = smoothstep(0.0, 0.035, h);   // stars fade out into the horizon haze

  // small stars: one grid cell (~0.12 degrees, about 3 px) each, so they read as pixels
  vec2 cell = floor(vec2(az, el) / 0.0021);
  float r = hash21(cell);
  if (r > 0.9950) {
    float b = hash21(cell + 7.1);
    float tw = 0.65 + 0.35 * sin(uTime * (1.0 + b * 3.0) + b * 60.0);
    vec3 tint = mix(vec3(0.72, 0.84, 1.0), vec3(1.0, 0.86, 0.66), hash21(cell + 3.3));
    col += tint * (0.45 + b * b * b * 2.2) * tw * haze;
  }
  // rarer, larger stars: 2x2 cells
  vec2 cell2 = floor(vec2(az, el) / 0.0042);
  float r2 = hash21(cell2 + 13.7);
  if (r2 > 0.9984) {
    float b2 = hash21(cell2 + 2.2);
    float tw2 = 0.7 + 0.3 * sin(uTime * (0.8 + b2 * 2.0) + b2 * 40.0);
    col += vec3(0.85, 0.92, 1.0) * (1.3 + b2 * 1.5) * tw2 * haze;
  }

  // moon: square, 10x10 pixel texture, soft halo
  float md = dot(dir, uMoonDir);
  if (md > 0.0) {
    vec3 right = normalize(cross(vec3(0.0, 1.0, 0.0), uMoonDir));
    vec3 up = cross(uMoonDir, right);
    vec2 q = vec2(dot(dir, right), dot(dir, up)) / md;
    float s = 0.022;
    float dist = length(q);
    col += vec3(0.32, 0.42, 0.75) * 0.09 * exp(-(dist * dist) / (0.07 * 0.07));
    col += vec3(0.35, 0.45, 0.8) * 0.02 * exp(-dist / 0.28);
    if (abs(q.x) < s && abs(q.y) < s) {
      vec2 g = floor((q + s) / (2.0 * s) * 10.0);
      float crater = step(0.80, hash21(g + 1.7));
      float maria = step(0.62, hash21(g * 1.31 + 9.1));
      float shade = 1.0 - 0.45 * crater - 0.20 * maria;
      col = vec3(0.86, 0.90, 1.0) * 0.8 * shade;
    }
  }

  // clouds: Minecraft-style square clouds in the perspective-projected cloud plane, drifting slowly, over a faint
  // teal / violet glow.
  {
    float hc = max(dir.y, 0.0);
    float edge = smoothstep(0.0, 0.02, hc) * (1.0 - smoothstep(0.55, 0.95, hc));
    vec2 sp = dir.xz / (hc + 0.40) * 0.85 + vec2(uTime * 0.010, uTime * 0.003);

    float g1 = fbm(sp * 0.45 + 40.0, 4);
    float g2 = fbm(sp * 0.30 + 77.0, 3);
    vec3 glow = mix(vec3(0.060, 0.025, 0.150), vec3(0.020, 0.150, 0.210), smoothstep(0.35, 0.65, g1));
    glow += vec3(0.02, 0.05, 0.09) * smoothstep(0.45, 0.75, g2);
    col += glow * (0.45 + 1.0 * smoothstep(0.30, 0.70, g1)) * edge * smoothstep(0.015, 0.22, hc);

    if (edge > 0.001) {
      // grid rows run across the view (rotated to the camera heading), like the game's cloud layer
      float ca = cos(uFwdAz);
      float sa = sin(uFwdAz);
      vec2 spr = vec2(sp.x * sa - sp.y * ca, sp.x * ca + sp.y * sa);
      vec2 cell = floor(spr * vec2(22.0, 34.0));
      float dens = mcCloud(cell);
      if (dens > 0.5) {
        // flat shading like the game: cell faces towards the moon are bright, the far side is dark
        vec2 lxz = normalize(uMoonDir.xz + 1e-4);
        vec2 off = vec2(abs(lxz.x) > 0.35 ? sign(lxz.x) : 0.0, abs(lxz.y) > 0.35 ? sign(lxz.y) : 0.0);
        float openToMoon = 1.0 - mcCloud(cell + off);
        float openAway = 1.0 - mcCloud(cell - off);
        vec3 top = vec3(0.30, 0.36, 0.56);
        float shade = 0.55 + 0.45 * openToMoon - 0.22 * openAway;
        vec3 cc = top * shade;
        float near = pow(max(md, 0.0), 40.0);
        cc += vec3(0.20, 0.26, 0.45) * near * 0.4;
        float moonGap = smoothstep(0.9930, 0.9985, md);   // keep a clearing around the moon
        col = mix(col, cc, 0.9 * edge * (1.0 - 0.9 * moonGap));
      }
    }
  }

  // far horizon: two blocky silhouette layers (hills, then a tree line) on a pixel grid, with a few tiny warm lights
  // of a far village on the hills. Both are tinted by the sky behind them, so they melt into the haze.
  {
    const float PX = 0.0017;                       // one block, in radians
    float colX = floor(az / (PX * 1.6));
    float hills = 0.004 + 0.034 * fbm(vec2(colX * 0.017, 1.7), 3);
    hills = floor(hills / PX) * PX;
    if (el >= 0.0 && el < hills) {
      col = col * 0.55 + vec3(0.003, 0.006, 0.016);
      // village lights: single block, just under the hill top, flickering a little
      float lr = hash21(vec2(colX, 31.7));
      float rowY = floor(el / PX) * PX;
      if (lr > 0.985 && rowY > hills - PX * 3.0 && rowY < hills - PX * 1.0) {
        col = vec3(1.0, 0.62, 0.28) * (1.3 + 0.5 * sin(uTime * (2.0 + lr * 9.0) + lr * 90.0));
      }
    }
    // below the horizon the sky dome shows the far land: a pixel field of forest canopy and a few lakes, seen from
    // above and melting into the haze towards the horizon, with the odd far light
    if (el < 0.0) {
      vec2 gp = dir.xz / max(-dir.y, 0.004);
      vec2 gc = floor(gp * 0.30);
      float n1 = fbm(gc * 0.11 + 5.0, 3);
      float n2 = hash21(gc);
      vec3 land = mix(vec3(0.012, 0.045, 0.052), vec3(0.036, 0.090, 0.066), smoothstep(0.35, 0.70, n1));
      land *= 0.75 + 0.5 * n2;
      land = mix(land, vec3(0.010, 0.030, 0.085), step(0.62, fbm(gc * 0.05 + 33.0, 3)));    // lakes catch the sky
      float haze = exp(el * 55.0);                                     // 1 at the horizon, gone below it
      vec3 farCol = col * 0.55 + vec3(0.003, 0.006, 0.016);
      col = mix(land, farCol, haze);
      float lr2 = hash21(gc + 71.3);
      if (lr2 > 0.9965) col = mix(col, vec3(1.0, 0.62, 0.28) * (1.1 + 0.4 * sin(uTime * (2.0 + lr2 * 40.0))), 1.0 - haze * 0.7);
    }
    float grp = floor(colX / 4.0);
    float lx = (colX - grp * 4.0 + 0.5) / 4.0;
    float tall = 0.003 + 0.011 * hash21(vec2(grp, 4.1));
    bool pine = hash21(vec2(grp, 9.3)) > 0.40;
    float tri = 1.0 - abs(2.0 * lx - 1.0);
    float trees = pine ? 0.0015 + tall * (0.25 + 0.75 * tri) : 0.0015 + tall * 0.55 * (abs(2.0 * lx - 1.0) > 0.75 ? 0.6 : 1.0);
    trees = floor(trees / PX) * PX;
    if (el >= 0.0 && el < trees) {
      col = col * 0.22 + vec3(0.001, 0.002, 0.006);
    }
  }

  // shooting star every ~13 s, visible for ~0.9 s
  float cyc = (uTime - uShootT0) / 13.0;
  float id = floor(cyc) + uShootSeed;
  float ph = fract(cyc);
  if (ph < 0.07) {
    float tt = ph / 0.07;
    vec2 p0 = vec2(uFwdAz - 0.05 + (hash21(vec2(id, 1.0)) - 0.5) * 0.7, 0.105 + hash21(vec2(id, 2.0)) * 0.03);
    vec2 v = vec2(0.30, -0.075);
    vec2 head = p0 + v * tt;
    vec2 d = normalize(v);
    vec2 pa = vec2(az, el) - head;
    float along = dot(pa, d);                       // negative = behind the head
    float across = abs(pa.x * d.y - pa.y * d.x);
    float trail = smoothstep(-0.06, 0.0, along) * step(along, 0.002);
    float w = 0.0016;
    col += vec3(0.9, 0.95, 1.0) * 4.0 * trail * smoothstep(w, 0.0, across) * (1.0 - tt);
  }

  gl_FragColor = vec4(col, 1.0);
}`

export function createSky(horizon: Vector3) {
  const material = new ShaderMaterial({
    side: BackSide,
    depthWrite: false,
    fog: false,
    uniforms: {
      uTime: { value: 0 },
      uHorizon: { value: horizon },
      uMoonDir: { value: MOON_DIR.clone() },
      uFwdAz: { value: FORWARD_AZ },
      uShootT0: { value: 0 },
      uShootSeed: { value: 0 },
    },
    vertexShader: VERT,
    fragmentShader: FRAG,
  })
  const mesh = new Mesh(new SphereGeometry(120, 32, 16), material)
  mesh.renderOrder = -1000
  mesh.frustumCulled = false
  let now = 0
  return {
    mesh,
    /** Test action: a shooting star right now (restarts the ~13 s cycle). */
    shoot() {
      material.uniforms.uShootT0.value = now
      material.uniforms.uShootSeed.value += 1
    },
    update(time: number, cameraPosition: Vector3) {
      now = time
      material.uniforms.uTime.value = time
      mesh.position.copy(cameraPosition)
    },
    dispose() {
      mesh.geometry.dispose()
      material.dispose()
    },
  }
}
