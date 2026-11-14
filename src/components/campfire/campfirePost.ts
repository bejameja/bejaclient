import { Color, Vector2, type Object3D, type PerspectiveCamera, type Scene, type WebGLRenderer } from 'three'
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer'
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass'
import { UnrealBloomPass } from 'three/examples/jsm/postprocessing/UnrealBloomPass'
import { ShaderPass } from 'three/examples/jsm/postprocessing/ShaderPass'
import { OutlinePass } from 'three/examples/jsm/postprocessing/OutlinePass'

// Post-processing for the campfire scene: bloom on the HDR highlights (fire, moon, bright stars), a gentle
// highlight roll-off, a cool-shadows / warm-highlights grade, a vignette, and the sRGB conversion + dither
// (the composer renders in linear half-float, so the final pass has to do the encoding the renderer would
// otherwise do itself).

const GRADE = {
  uniforms: { tDiffuse: { value: null as unknown }, uTime: { value: 0 } },
  vertexShader: /* glsl */ `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }`,
  fragmentShader: /* glsl */ `
    uniform sampler2D tDiffuse;
    uniform float uTime;
    varying vec2 vUv;

    vec3 toSRGB(vec3 c) {
      return mix(c * 12.92, 1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, step(vec3(0.0031308), c));
    }
    float hash(vec2 p) {
      p = fract(p * vec2(443.897, 441.423));
      p += dot(p, p.yx + 19.19);
      return fract((p.x + p.y) * p.x);
    }

    void main() {
      vec3 c = texture2D(tDiffuse, vUv).rgb * 0.92;   // exposure

      // soft highlight roll-off above the knee (keeps normal colours untouched)
      float peak = max(c.r, max(c.g, c.b));
      float knee = 0.85;
      if (peak > knee) {
        float t = peak - knee;
        c *= (knee + t / (1.0 + t * 1.4)) / peak;
      }

      // grade: slightly richer colour, cool lift in the shadows, warm highlights
      float lum = dot(c, vec3(0.2126, 0.7152, 0.0722));
      c = mix(vec3(lum), c, 1.12);
      c += vec3(0.004, 0.0, 0.008) * (1.0 - smoothstep(0.0, 0.3, lum));   // cool shadows
      c *= mix(vec3(1.0), vec3(1.06, 1.0, 0.91), smoothstep(0.3, 1.0, lum));   // warm highlights

      // vignette
      vec2 q = (vUv - 0.5) * vec2(1.0, 1.15);
      c *= clamp(1.0 - dot(q, q) * 1.15, 0.0, 1.0);

      c = toSRGB(max(c, 0.0));
      // dither: the dark sky gradient bands badly in 8 bit otherwise
      c += (hash(gl_FragCoord.xy + fract(uTime) * 91.7) - 0.5) * (1.2 / 255.0);
      gl_FragColor = vec4(c, 1.0);
    }`,
}

export function createPost(renderer: WebGLRenderer, scene: Scene, camera: PerspectiveCamera, width: number, height: number) {
  const composer = new EffectComposer(renderer)
  composer.setSize(width, height)
  // 2x MSAA on the composer targets (RAM: each extra sample costs a full half-float target): the plain canvas antialiasing does not apply to post-processing
  if (renderer.capabilities.isWebGL2) {
    const targets = [composer.renderTarget1, composer.renderTarget2] as any[]
    for (const rt of targets) rt.samples = 2
  }
  composer.addPass(new RenderPass(scene, camera))
  const bloom = new UnrealBloomPass(new Vector2(width, height), 0.18, 0.35, 0.95)   // strength, radius, threshold
  composer.addPass(bloom)
  // hover outline (after the bloom so it stays crisp; costs nothing while nothing is selected)
  const outline = new OutlinePass(new Vector2(width, height), scene, camera)
  outline.edgeStrength = 3
  outline.edgeThickness = 1
  outline.edgeGlow = 0
  outline.pulsePeriod = 0
  outline.visibleEdgeColor = new Color(1, 1, 1)
  outline.hiddenEdgeColor = new Color(0.55, 0.55, 0.55)
  composer.addPass(outline)
  const grade = new ShaderPass(GRADE as any)
  composer.addPass(grade)
  return {
    render(time: number) {
      grade.uniforms.uTime.value = time
      composer.render()
    },
    setSize(w: number, h: number) { composer.setSize(w, h) },
    setPixelRatio(r: number) { composer.setPixelRatio(r) },
    setBloom(on: boolean) { bloom.enabled = on },
    setOutline(objects: Object3D[]) { outline.selectedObjects = objects },
    dispose() {
      composer.renderTarget1.dispose()
      composer.renderTarget2.dispose()
      ;(bloom as any).dispose?.()
    },
  }
}
