# arkit

`arkit` is a Dioxus 0.7 native renderer for OpenHarmony ArkUI. Applications use normal Dioxus components, `rsx!`, signals, hooks, async resources, and `dioxus-router`; the framework translates Dioxus mutations into ArkUI native nodes.

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
- `arkit_chart`: ECharts-compatible typed/JSON options rendered by an ArkUI native canvas and updated through Dioxus props.
- `arkit_router`, `arkit_i18n`, `arkit_animation`, `arkit_icon`, `arkit_shadcn`: Dioxus-native framework capabilities.

There is no parallel Element tree or message/update runtime. Dioxus owns component identity, hooks, diffing, task scheduling, and routing.

## Examples

- `examples/counter`: signals and native events.
- `examples/async_task`: `use_resource` with a Tokio-backed future.
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
```

OpenHarmony validation must use the project toolchain from the example directory:

```sh
cd examples/counter
ohrs build --arch aarch
```

`cargo check` is useful as a host-side diagnostic, but it does not replace an `ohrs` build. Package and deploy one example at a time through `app/run.sh` after its `ohrs` build succeeds.

Documentation uses VitePress:

```sh
pnpm install
pnpm run docs:dev
```
