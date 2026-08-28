# Downscaled wallpaper sheet

`src/assets/pass-bg.png` is a baked video-frame texture atlas at **21099×12335**
(33.3 MB on disk, **~993 MB decoded in RAM**). That is far larger than a
background needs and is a real load-time and memory cost.

These are aspect-preserving downscales of that sheet, resampled with `INTER_AREA`
(the correct filter for shrinking — no aliasing), intended to be loaded
**server-side** rather than bundled, so the client pulls only the resolution it
needs. Because the scale is uniform, every frame's position in the atlas stays
proportional and existing UV math is unchanged.

| Variant | Dimensions | File size | vs. original | Decoded RAM (RGBA) |
| --- | --- | --- | --- | --- |
| original (`src/assets/pass-bg.png`) | 21099×12335 | 33.34 MB | — | ~993 MB |
| `pass-bg@8192.png` | 8192×4789 | 3.03 MB | 9% | ~150 MB |
| `pass-bg@4096.png` | 4096×2395 | 0.41 MB | 1% | ~37 MB |
| `pass-bg@2048.png` | 2048×1197 | 0.12 MB | 0% | ~9 MB |

`@4096` is the recommended default: 4096 px wide is ample for a full-screen
background at ~27× less RAM than the original. `@2048` for the lightest
footprint, `@8192` only if softness shows at native resolution.

The original in `src/assets/` is left untouched.
