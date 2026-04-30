# arkit

`arkit` is an ArkUI framework for OpenHarmony.

It is built on top of local `ohos-native-bindings`, integrates with
`openharmony-ability`, and uses an `iced`-style programming model:

- application state lives in `State`
- rendering is `view(&State) -> Element`
- side effects flow through `Task<Message>` and `Subscription<Message>`

## Taste

```rust
use arkit::prelude::*;
use arkit::{application, Element, Task};

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

#[derive(Default)]
struct State {
    count: i32,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Increment => state.count += 1,
    }

    Task::none()
}

fn view(state: &State) -> Element<Message> {
    column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .align_items_center()
        .justify_content_center()
        .children(vec![
            text(format!("count = {}", state.count))
                .font_size(28.0)
                .line_height(32.0)
                .into(),
            button("increment")
                .margin_top(12.0)
                .padding([8.0, 12.0, 8.0, 12.0])
                .on_press(Message::Increment)
                .into(),
        ])
        .into()
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(State::default, update, view)
}
```

The complete runnable version is in [examples/counter](examples/counter/src/lib.rs).

## Feature Flags

- `api-22`: baseline OHOS API level, enabled by default
- `webview`: enables embedded webview support through `openharmony-ability`

## Examples

- `examples/counter`: minimal state + button update example
- `examples/async_task`: `Task::perform` example
- `examples/webview`: embedded webview example behind the `webview` feature
- `examples/shadcn_showcase`: UI showcase built with `arkit_shadcn`

## Workspace

- `crates/arkit`: facade crate and public re-exports
- `crates/arkit_widget`: widget tree, renderer, overlays, and ArkUI bindings glue
- `crates/arkit_runtime`: application runtime, task execution, and subscriptions
- `crates/arkit_derive`: `#[entry]` and `#[component]`
- `crates/arkit_shadcn`: shadcn-style components on top of `arkit`

## Building

Build an example from its crate directory with `ohrs`:

```sh
cd examples/counter
ohrs build --arch aarch
```

For webview examples, enable the `webview` feature in the crate dependency:

```toml
arkit = { workspace = true, features = ["webview"] }
```

## Documentation

The framework documentation is built with VitePress:

```sh
pnpm install
pnpm run docs:dev
```

Static output:

```sh
pnpm run docs:build
```
