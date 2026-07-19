# `@napi-rs/canvas` example coverage

Audit source: [`Brooooooklyn/canvas/example`](https://github.com/Brooooooklyn/canvas/tree/dc3377f5673afe3d843db761cdfaf999f12ac75e/example).

This matrix maps the upstream Node APIs to the typed Arkit owners that provide
the same capability. Codec and font operations deliberately live on
`CanvasImage`, `OffscreenCanvas`, and `CanvasFontRegistry`, rather than adding
file I/O or lifecycle state to `CanvasRenderingContext2D`.

| Upstream example | Arkit implementation | Status |
| --- | --- | --- |
| `simple.js` | `OffscreenCanvas` + `convert_to_blob(Png)` | Complete. |
| `tiger.js` | Cached `Path2D` layers | Complete; the `Canvas Tiger` page renders all 240 vector layers. |
| `path-empty-line-to.js` | W3C current-path behavior + PNG output | Complete. |
| `draw-text-with-baseline.js` | `CanvasFontFace` + all Canvas baselines | Complete. |
| `draw-text.js` | `CanvasFontRegistry`, conic-gradient text, PNG output | Complete. |
| `measure-text.js` | Native typeface metrics and all six baselines | Complete. |
| `round-path.js` | `Path2D::round()` | Complete; retained as an explicitly documented upstream extension. |
| `image-data.js` | `ImageData`, PNG decode, `putImageData`, PNG output | Complete. |
| `image.js` | `CanvasImage::decode`, all three `drawImage` forms, encoded output | Complete. |
| `resize-svg.js` | Device SVG decoder with `desired_size` | Complete when SVG is listed by `supported_decode_formats()`. |
| `anime-girl.js` | SVG decode + PNG/AVIF encoder | Complete for formats reported by the device image framework. OHOS exposes quality but not the upstream AVIF `speed` knob. |
| `anime-girl-quality.js` | SVG decode + quality-controlled WebP output | Complete for formats reported by the device image framework. |
| `draw-emoji.js` | Runtime font bytes + native color-glyph drawing | Implemented; exact COLRv1 coverage follows the OHOS native drawing/font version on the device. |
| `export-svg.js` | `OffscreenCanvas::convert_to_svg()` | Pixel-exact SVG with an embedded PNG. True vector recording and `ConvertTextToPaths` are unavailable in the OHOS native drawing API. |
| `lottie-to-video.ts` | `LottieFrameRenderer` emits reusable, timed RGBA frames consumable by Canvas or an encoder | Frame rendering is complete. H.264 encoding and MP4 muxing remain unavailable because this workspace has no typed OHOS AVCodec/muxer binding; they are not emulated in Canvas. |

## Result

Every upstream scene can now be rendered. The Pipeline page exercises SVG
decode, a Lottie frame, `Path2D::round()`, off-screen composition, PNG encode,
and PNG decode as one round trip. Remaining device-dependent differences are
queried at runtime instead of being claimed unconditionally: AVIF/WebP/SVG
support comes from the OHOS image framework, color-font coverage comes from
native drawing, and true vector SVG plus MP4 require native APIs that OHOS does
not currently expose through this workspace.
