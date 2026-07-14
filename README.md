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

## Architecture

- `arkit`: public facade, prelude, and `#[entry]` launch wrapper.
- `arkit_runtime`: owns the Dioxus `VirtualDom`, connects the scheduler to the OpenHarmony event loop, queues native events before Dioxus dispatch, and hosts embedded WebView state.
- `arkit_arkui`: owns the HostTree projection, declarative attribute encoding, native node-event/gesture bridge, ArkUI node creation, image resources, and virtual adapters.
- `arkit_elements`: the ArkUI `dioxus_elements` registry used by `rsx!`.
- `arkit_hooks`: Dioxus hooks for native-node access, layout, overlays, and virtual lists.
- `arkit_animation_core`: platform-independent resolve/compile/sample/state engine; it has no ArkUI dependency.
- `arkit_animation`: root-owned frame driver, ArkUI/Drawing adapters, native lowering, layout/presence/drag/scroll integration.
- `arkit_chart`: ECharts-compatible typed/JSON options rendered by an ArkUI native canvas and updated through Dioxus props.
- `arkit_router`, `arkit_i18n`, `arkit_icon`, `arkit_shadcn`: Dioxus-native framework capabilities.

There is no parallel Element tree or message/update runtime. Dioxus owns component identity, hooks, diffing, task scheduling, and routing.

The `arkit` facade has an intentionally small default feature set. Core renderer/runtime APIs are always available; domain features are opt-in:

| Feature | Adds |
| --- | --- |
| `animation` | Animation engine and interaction hooks |
| `chart` | Native charts; also enables `animation` |
| `i18n` | Typed Fluent catalogs and reactive locale context |
| `icon` | Embedded SVG icon catalog |
| `router` | Dioxus Router integration; also enables `animation` |
| `shadcn` | Component library; also enables `animation` and `icon` |
| `full` | All domain features |

For example: `arkit = { path = "crates/arkit", features = ["animation", "i18n"] }`.

## Examples

- `examples/counter`: signals and native events.
- `examples/async_task`: `use_resource` with a Tokio-backed future.
- `examples/animation`: timeline, easing, lifecycle, orchestration, drag/scroll, layout/presence, native lowering, and diagnostics labs.
- `examples/chart`: native ECharts-compatible chart types and signal-driven realtime updates.
- `examples/router`: typed `dioxus-router` routes and ArkUI links.
- `examples/i18n`: reactive locale context.
- `examples/complex_cases`: ArkUI NodeAdapter virtualization.
- `examples/shadcn_showcase`: component and theme showcase.
- `examples/webview`: embedded WebView controlled from Dioxus.

## Building

Format the Rust workspace:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

OpenHarmony validation must use the project toolchain from the example directory:

```sh
cd examples/counter
ohrs build --arch aarch
```

`cargo check` is useful as a host-side diagnostic, but it does not replace an `ohrs` build. Package and deploy one example at a time through `app/run.sh` after its `ohrs` build succeeds.

`openharmony-ability` is pinned to commit `edc4e49d0d431035c6c001fc5e583abf62a998e3`, whose ArkUI, XComponent, Display, and resource-manager dependency ranges match this workspace. Keep the lockfile on one binding/sys generation; no local Cargo patch adapters are required.

Documentation uses VitePress:

```sh
pnpm install
pnpm run docs:dev
```
