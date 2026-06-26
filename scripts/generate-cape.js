#!/usr/bin/env node
'use strict'
/**
 * Generates resources/capes/beja-default.png
 * 64x32 Minecraft cape texture — solid black with pixel-art "B" on front+back faces.
 * Run with: node scripts/generate-cape.js
 */
const zlib = require('zlib')
const fs   = require('fs')
const path = require('path')

const W = 64, H = 32

// RGBA pixel buffer
const buf = Buffer.alloc(W * H * 4, 0)

function px(x, y, r, g, b, a = 255) {
  if (x < 0 || x >= W || y < 0 || y >= H) return
  const i = (y * W + x) * 4
  buf[i] = r; buf[i + 1] = g; buf[i + 2] = b; buf[i + 3] = a
}

// ── Fill all black ─────────────────────────────────────────────────────────────
for (let y = 0; y < H; y++)
  for (let x = 0; x < W; x++)
    px(x, y, 0, 0, 0, 255)

// ── Pixel-art "B" (7 wide × 9 tall) ──────────────────────────────────────────
//  Styled to roughly match the BejaClient logo mark
const LOGO = [
  [0,1,1,1,1,1,0],
  [0,1,0,0,0,1,1],
  [0,1,0,0,0,0,1],
  [0,1,1,1,1,1,0],
  [0,1,0,0,0,0,1],
  [0,1,0,0,0,1,1],
  [0,1,1,1,1,1,0],
  [0,0,0,0,0,0,0],
  [0,0,0,1,1,0,0], // tiny "C" hint below — optional dot accent
]
const LW = 7, LH = 9

function drawLogo(faceOriginX, faceOriginY, faceW = 10, faceH = 16) {
  // Center the logo in the face
  const offX = faceOriginX + Math.floor((faceW - LW) / 2)
  const offY = faceOriginY + Math.floor((faceH - LH) / 2)
  for (let r = 0; r < LH; r++) {
    for (let c = 0; c < LW; c++) {
      if (LOGO[r][c]) px(offX + c, offY + r, 220, 220, 220, 255)
    }
  }
}

// Front face: texture x=[1..10], y=[1..16]
drawLogo(1, 1)

// Back face: texture x=[12..21], y=[1..16]
drawLogo(12, 1)

// ── Subtle 1-px border on both faces ─────────────────────────────────────────
function drawBorder(ox, oy, w = 10, h = 16) {
  const c = 40 // dark gray
  for (let i = 0; i < w; i++) { px(ox + i, oy,         c, c, c, 255); px(ox + i, oy + h - 1, c, c, c, 255) }
  for (let j = 0; j < h; j++) { px(ox,     oy + j,     c, c, c, 255); px(ox + w - 1, oy + j, c, c, c, 255) }
}
drawBorder(1, 1)
drawBorder(12, 1)

// ── PNG encoding (pure Node, no deps) ─────────────────────────────────────────
function crc32(data) {
  const t = new Uint32Array(256)
  for (let i = 0; i < 256; i++) {
    let c = i
    for (let k = 0; k < 8; k++) c = (c & 1) ? 0xEDB88320 ^ (c >>> 1) : c >>> 1
    t[i] = c
  }
  let crc = 0xFFFFFFFF
  for (const b of data) crc = t[(crc ^ b) & 0xFF] ^ (crc >>> 8)
  return (crc ^ 0xFFFFFFFF) >>> 0
}

function chunk(type, payload) {
  const tb = Buffer.from(type, 'ascii')
  const lb = Buffer.alloc(4); lb.writeUInt32BE(payload.length)
  const cb = Buffer.alloc(4); cb.writeUInt32BE(crc32(Buffer.concat([tb, payload])))
  return Buffer.concat([lb, tb, payload, cb])
}

const ihdr = Buffer.alloc(13)
ihdr.writeUInt32BE(W, 0); ihdr.writeUInt32BE(H, 4)
ihdr[8] = 8; ihdr[9] = 6  // 8-bit RGBA

const rows = []
for (let y = 0; y < H; y++) {
  rows.push(Buffer.alloc(1, 0)) // filter: None
  rows.push(buf.slice(y * W * 4, (y + 1) * W * 4))
}
const idat = zlib.deflateSync(Buffer.concat(rows), { level: 9 })

const png = Buffer.concat([
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
  chunk('IHDR', ihdr),
  chunk('IDAT', idat),
  chunk('IEND', Buffer.alloc(0)),
])

const outDir  = path.join(__dirname, '../resources/capes')
const outPath = path.join(outDir, 'beja-default.png')
if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true })
fs.writeFileSync(outPath, png)
console.log('✓ Cape written to:', outPath)
