// Real elbow/knee bend for skinview3d's player model. skinview3d's own
// SkinObject (node_modules/skinview3d/libs/model.js) builds each limb as a
// single rigid box per layer (inner+outer) - no forearm/shin joint exists to
// rotate. This module replaces those single boxes with two half-height boxes
// (upper/lower) plus a rotatable Group ("joint") at the midpoint, so the
// lower half can actually bend independently of the upper half.
//
// UV mapping for the two halves reuses skinview3d's own setSkinUVs formula
// (reimplemented below - it's a private, unexported function in model.js)
// with the same u/v/width/depth constants skinview3d itself uses per limb,
// just with height halved and v shifted for the lower half. The two new cut
// faces at the joint (visible only once actually bent) get whatever that
// formula computes for their row rather than a hand-matched cap texture -
// it lands within the same limb's skin-texture region so it reads as
// plausible skin/sleeve color, not a hand-picked "correct" cap. The lower
// half's outward-facing cap (bottom of a hand/foot) is very slightly off for
// the same reason - minor, low-visibility trade-off accepted for scope.
//
// Installed once per viewer (see HeroSkinViewer.vue) using whichever
// slim/classic model was active at install time; if the skin's arm width
// later changes (slim ⇄ classic) the split geometry does NOT resize -
// same limitation would apply to any from-scratch rig like this without a
// full rebuild-on-model-change path, and out of scope here.

import { BoxGeometry, Group, Mesh, MeshStandardMaterial, Vector2 } from 'three'

function setSkinUVs(box: BoxGeometry, u: number, v: number, width: number, height: number, depth: number) {
  const tw = 64
  const th = 64
  const toFaceVertices = (x1: number, y1: number, x2: number, y2: number) => [
    new Vector2(x1 / tw, 1 - y2 / th),
    new Vector2(x2 / tw, 1 - y2 / th),
    new Vector2(x2 / tw, 1 - y1 / th),
    new Vector2(x1 / tw, 1 - y1 / th),
  ]
  const top    = toFaceVertices(u + depth, v, u + width + depth, v + depth)
  const bottom = toFaceVertices(u + width + depth, v, u + width * 2 + depth, v + depth)
  const left   = toFaceVertices(u, v + depth, u + depth, v + depth + height)
  const front  = toFaceVertices(u + depth, v + depth, u + width + depth, v + depth + height)
  const right  = toFaceVertices(u + width + depth, v + depth, u + width + depth * 2, v + height + depth)
  const back   = toFaceVertices(u + width + depth * 2, v + depth, u + width * 2 + depth * 2, v + height + depth)
  const uvAttr = box.attributes.uv as any
  uvAttr.copyVector2sArray([
    right[3], right[2], right[0], right[1],
    left[3], left[2], left[0], left[1],
    top[3], top[2], top[0], top[1],
    bottom[0], bottom[1], bottom[3], bottom[2],
    front[3], front[2], front[0], front[1],
    back[3], back[2], back[0], back[1],
  ])
  uvAttr.needsUpdate = true
}

interface LayerSpec { u: number; v: number; uvWidth: number; meshWidth: number; depth: number; height: number }
interface LimbSpec { inner: LayerSpec; outer: LayerSpec }

// Constants copied from skinview3d's SkinObject constructor (model.js).
// Note: outer layer's UV *width* intentionally reuses the inner layer's
// slim/classic width even though the outer mesh itself renders ~0.5 wider -
// that's skinview3d's own convention (the "puffy sleeve" overlay samples the
// same relative texture rectangle as the base layer), mirrored here exactly.
function limbConfigs(slim: boolean) {
  const armW = slim ? 3 : 4
  const rightArm: LimbSpec = {
    inner: { u: 40, v: 16, uvWidth: armW, meshWidth: armW,          depth: 4,   height: 12 },
    outer: { u: 40, v: 32, uvWidth: armW, meshWidth: slim ? 3.5 : 4.5, depth: 4.5, height: 12.5 },
  }
  const leftArm: LimbSpec = {
    inner: { u: 32, v: 48, uvWidth: armW, meshWidth: armW,          depth: 4,   height: 12 },
    outer: { u: 48, v: 48, uvWidth: armW, meshWidth: slim ? 3.5 : 4.5, depth: 4.5, height: 12.5 },
  }
  const rightLeg: LimbSpec = {
    inner: { u: 0,  v: 16, uvWidth: 4, meshWidth: 4,   depth: 4,   height: 12 },
    outer: { u: 0,  v: 32, uvWidth: 4, meshWidth: 4.5, depth: 4.5, height: 12.5 },
  }
  const leftLeg: LimbSpec = {
    inner: { u: 16, v: 48, uvWidth: 4, meshWidth: 4,   depth: 4,   height: 12 },
    outer: { u: 0,  v: 48, uvWidth: 4, meshWidth: 4.5, depth: 4.5, height: 12.5 },
  }
  return { rightArm, leftArm, rightLeg, leftLeg }
}

function buildHalf(layer: LayerSpec, half: 'upper' | 'lower', material: MeshStandardMaterial): Mesh {
  const h = layer.height / 2
  const v = half === 'upper' ? layer.v : layer.v + h
  const box = new BoxGeometry(layer.meshWidth, h, layer.depth)
  setSkinUVs(box, layer.u, v, layer.uvWidth, h, layer.depth)
  return new Mesh(box, material)
}

// `bodyPart` is skinview3d's BodyPart (skin.rightArm / .leftArm / .rightLeg /
// .leftLeg) - a Group whose only child is a pivot Group holding the original
// single-box inner/outer meshes (BodyPart.innerLayer / .outerLayer).
// Returns the new joint Group; rotate its .rotation.x to bend.
function splitLimb(bodyPart: any, spec: LimbSpec): Group {
  const pivot: Group = bodyPart.children[0]
  const innerMat = (bodyPart.innerLayer as Mesh).material as MeshStandardMaterial
  const outerMat = (bodyPart.outerLayer as Mesh).material as MeshStandardMaterial

  pivot.remove(bodyPart.innerLayer, bodyPart.outerLayer)
  ;(bodyPart.innerLayer as Mesh).geometry.dispose()
  ;(bodyPart.outerLayer as Mesh).geometry.dispose()

  const upperInner = buildHalf(spec.inner, 'upper', innerMat)
  const upperOuter = buildHalf(spec.outer, 'upper', outerMat)
  const lowerInner = buildHalf(spec.inner, 'lower', innerMat)
  const lowerOuter = buildHalf(spec.outer, 'lower', outerMat)

  // Original single box was centred on the pivot's own local origin (0,0,0)
  // - confirmed from model.js: the mesh gets no explicit .position, only
  // .scale, and the pivot Group itself already carries the shoulder/hip
  // offset. Midpoint (the joint) is therefore pivot-local y = 0.
  const innerHalfH = spec.inner.height / 2
  const outerHalfH = spec.outer.height / 2
  upperInner.position.y = innerHalfH / 2
  upperOuter.position.y = outerHalfH / 2

  const joint = new Group()
  pivot.add(upperInner, upperOuter, joint)

  lowerInner.position.y = -innerHalfH / 2
  lowerOuter.position.y = -outerHalfH / 2
  joint.add(lowerInner, lowerOuter)

  return joint
}

export interface LimbBendJoints {
  rightArm: Group
  leftArm:  Group
  rightLeg: Group
  leftLeg:  Group
}

export function installLimbBendRig(skin: any, slim: boolean): LimbBendJoints {
  const cfg = limbConfigs(slim)
  return {
    rightArm: splitLimb(skin.rightArm, cfg.rightArm),
    leftArm:  splitLimb(skin.leftArm,  cfg.leftArm),
    rightLeg: splitLimb(skin.rightLeg, cfg.rightLeg),
    leftLeg:  splitLimb(skin.leftLeg,  cfg.leftLeg),
  }
}
