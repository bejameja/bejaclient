import { computed } from 'vue'
import { useSettingsStore } from '../store/settingsStore'
import type { ElementOverride } from '../types'

const RADIUS_VALUES: Partial<Record<NonNullable<ElementOverride['radius']>, string>> = {
  sharp: '0px',
  rounded: '18px',
}

const SHADOW_VALUES: Partial<Record<NonNullable<ElementOverride['shadow']>, string>> = {
  soft: '0 6px 20px rgba(0, 0, 0, 0.35)',
  glow: '0 0 0 1px var(--accent, #3159a3), 0 0 24px color-mix(in srgb, var(--accent, #3159a3) 55%, transparent)',
  strong: '0 16px 40px rgba(0, 0, 0, 0.6)',
}

const FONT_WEIGHT_VALUES: Partial<Record<NonNullable<ElementOverride['fontWeight']>, string>> = {
  normal: '400',
  medium: '600',
  bold: '800',
}

/** Per-element Editor Mode override — reactive, persisted to settings.json, keyed by a stable id. */
export function useElementStyle(id: string) {
  const settingsStore = useSettingsStore()

  const override = computed<ElementOverride>(() => settingsStore.settings.customization.elements[id] ?? {})

  // Vars meant to cascade to descendants — components opt in via var(--edr-x, fallback).
  const cssVars = computed(() => {
    const o = override.value
    const vars: Record<string, string> = {}
    if (o.radius && RADIUS_VALUES[o.radius]) vars['--edr-radius'] = RADIUS_VALUES[o.radius]!
    if (o.outline) vars['--edr-outline'] = `${o.outlineWidth ?? 2}px solid ${o.outlineColor || 'var(--accent, #3159a3)'}`
    if (o.color) vars['--edr-color'] = o.color
    if (o.bgColor) vars['--edr-bg'] = o.bgColor
    if (o.fontFamily) vars['--edr-font'] = o.fontFamily
    return vars
  })

  // Universal wrapper-level styles — applied directly to the EditableRegion box
  // itself, so they work on every wrapped element with zero per-component CSS.
  const wrapperStyle = computed(() => {
    const o = override.value
    const style: Record<string, string> = {}
    if (o.opacity !== undefined && o.opacity !== 100) style.opacity = String(o.opacity / 100)
    if (o.blur) style.backdropFilter = `blur(${o.blur}px)`
    if (o.shadow && o.shadow !== 'none' && SHADOW_VALUES[o.shadow]) style.boxShadow = SHADOW_VALUES[o.shadow]!
    if (o.fontSize) style.fontSize = `${o.fontSize}px`
    if (o.fontWeight && FONT_WEIGHT_VALUES[o.fontWeight]) style.fontWeight = FONT_WEIGHT_VALUES[o.fontWeight]!
    if (o.letterSpacing) style.letterSpacing = `${o.letterSpacing}px`
    return style
  })

  const hasOverride = computed(() => Object.keys(override.value).length > 0)

  function patch(p: Partial<ElementOverride>) {
    const elements = { ...settingsStore.settings.customization.elements, [id]: { ...override.value, ...p } }
    settingsStore.patch({ customization: { ...settingsStore.settings.customization, elements } })
  }

  function applyPreset(presetPatch: Partial<ElementOverride>) {
    // Presets replace the visual-style fields wholesale but keep position/size/media.
    const { offsetX, offsetY, scale, bgVideo, bgImage } = override.value
    const elements = {
      ...settingsStore.settings.customization.elements,
      [id]: { offsetX, offsetY, scale, bgVideo, bgImage, ...presetPatch },
    }
    settingsStore.patch({ customization: { ...settingsStore.settings.customization, elements } })
  }

  function reset() {
    const elements = { ...settingsStore.settings.customization.elements }
    delete elements[id]
    settingsStore.patch({ customization: { ...settingsStore.settings.customization, elements } })
  }

  return { override, cssVars, wrapperStyle, hasOverride, patch, applyPreset, reset }
}

/** Global Editor Mode on/off switch — toggled from Settings → Appearance or the floating button. */
export function useEditorMode() {
  const settingsStore = useSettingsStore()
  const enabled = computed(() => settingsStore.settings.customization.editorMode)
  const wiggleEnabled = computed(() => settingsStore.settings.customization.wiggleEnabled)
  function setEnabled(v: boolean) {
    settingsStore.patch({ customization: { ...settingsStore.settings.customization, editorMode: v } })
  }
  function setWiggleEnabled(v: boolean) {
    settingsStore.patch({ customization: { ...settingsStore.settings.customization, wiggleEnabled: v } })
  }
  return { enabled, setEnabled, wiggleEnabled, setWiggleEnabled }
}

export const AVAILABLE_FONTS = [
  { label: 'IBM Plex Sans (Standard)', value: '' },
  { label: 'Plus Jakarta Sans', value: "'Plus Jakarta Sans', sans-serif" },
  { label: 'Barlow Condensed', value: "'Barlow Condensed', sans-serif" },
  { label: 'Mojangles', value: "'Mojangles', monospace" },
  { label: 'System UI', value: 'system-ui, sans-serif' },
  { label: 'Georgia (Serif)', value: 'Georgia, serif' },
  { label: 'Monospace', value: "'IBM Plex Mono', monospace" },
]

export interface StylePreset {
  name: string
  patch: Partial<ElementOverride>
}

export const STYLE_PRESETS: StylePreset[] = [
  { name: 'Standard', patch: { radius: 'default', outline: false, shadow: 'none', blur: undefined, bgColor: undefined, color: undefined } },
  { name: 'Glas', patch: { radius: 'rounded', outline: true, outlineColor: 'rgba(255,255,255,0.35)', outlineWidth: 1, blur: 14, bgColor: 'rgba(255,255,255,0.08)', shadow: 'soft' } },
  { name: 'Neon', patch: { radius: 'rounded', outline: true, outlineColor: '#78aaff', outlineWidth: 2, shadow: 'glow' } },
  { name: 'Minimal', patch: { radius: 'sharp', outline: false, shadow: 'none', blur: undefined } },
  { name: 'Kontrast', patch: { radius: 'sharp', outline: true, outlineColor: '#ffffff', outlineWidth: 2, shadow: 'strong' } },
]
