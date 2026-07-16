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
            percent_width: 1.0,
            percent_height: 1.0,
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

Optional domain APIs are feature-gated. For example, native CameraKit preview
and JPEG capture are enabled with `arkit = { features = ["camera"] }`; configurable
barcode scanning is added by `camera-scan`. CameraKit and scan-decoder dependency edges
are absent from the default graph and follow their respective features.
See [examples/camera](examples/camera/src/lib.rs).

## License

[MIT](./LICENSE-MIT) or [Apache2.0](./LICENSE-APACHE)
