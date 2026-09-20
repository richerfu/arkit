# arkit

`arkit` is a Dioxus 0.7 native renderer for OpenHarmony ArkUI. Applications use normal Dioxus components, `rsx!`, signals, hooks, async resources, and `dioxus-router`; the framework translates Dioxus mutations into ArkUI native nodes.

The workspace MSRV is Rust 1.88, matching the resolved OpenHarmony N-API toolchain dependencies.

```rust
use arkit::prelude::*;

#[entry]
fn app() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        column {
            width: "100%",
            height: "100%",
            align_items: "center",
            justify_content: "center",

            text { font_size: 28.0, "count = {count}" }
            button {
                margin_top: 12.0,
                onclick: move |_| count += 1,
                "increment"
            }
        }
    }
}
```

The complete runnable version is in [examples/counter](examples/counter/src/lib.rs).

## Adaptive Phone and PC layouts

Arkit resolves adaptive styles from the current application-window width, so
freeform and split-screen windows can change style without relying on the
physical device category. `Auto` switches to the PC style at 840vp by default;
applications can override the mode or breakpoint for a subtree.

```rust
use arkit::prelude::*;

#[component]
fn App() -> Element {
    rsx! {
        AdaptiveProvider {
            config: AdaptiveConfig::new(AdaptiveMode::Auto)
                .with_pc_min_width(840.0),
            ResponsiveContent {}
        }
    }
}

#[component]
fn ResponsiveContent() -> Element {
    let layout = use_adaptive_layout();

    rsx! {
        column {
            padding: layout.select(16.0, 32.0),
            AdaptiveView {
                phone: rsx! { text { "Phone navigation" } },
                pc: rsx! { text { "PC navigation" } },
            }
        }
    }
}
```

Use `AdaptiveMode::Phone` or `AdaptiveMode::Pc` to provide an explicit user or
application preference. Components without a structural difference can read
`use_adaptive_layout()` and select only the affected dimensions or interaction
states.

Applications can register their own openharmony-ability bridge plugins
(`BridgePlugin` facades) through `#[entry(plugins = [...])]` or by taking an
`OpenHarmonyApp` handle in the entry function — see [examples/plugins](examples/plugins/src/lib.rs) and the [getting-started guide](website/src/content/docs/getting-started.md).

Optional domain APIs are feature-gated. For example, native CameraKit preview
and JPEG capture are enabled with `arkit = { features = ["camera"] }`; configurable
barcode scanning is added by `camera-scan`. CameraKit and scan-decoder dependency edges
are absent from the default graph and follow their respective features.
See [examples/camera](examples/camera/src/lib.rs).

The `canvas` feature provides a W3C/WHATWG-aligned Canvas 2D context rendered
by an ArkUI custom-draw node with a persistent high-DPI native backing store.
It covers Path2D, gradients/patterns, shadows/filters, text, images and
ImageData. `OffscreenCanvas`, device image codecs, runtime font registration,
SVG rasterization, and timed Lottie frame export are available through typed
owners without putting file or codec state on `CanvasRenderingContext2D`.
The default dependency graph remains free of native drawing APIs and the
on-screen component does not require a separate XComponent surface. See
[examples/canvas](examples/canvas/src/lib.rs).

High-performance Lottie rendering is enabled independently with the `lottie`
feature; cancellable HTTP/HTTPS URL sources are added by `lottie-network` so
embedded-only apps do not pay for Reqwest/Rustls. ThorVG runs on a render worker
and writes directly into an ArkUI XComponent native window; the default
dependency graph contains neither the renderer nor network stack. See
[examples/lottie](examples/lottie/src/lib.rs).

## License

[MIT](./LICENSE-MIT) or [Apache2.0](./LICENSE-APACHE)
