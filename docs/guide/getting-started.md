# 快速开始

## 安装

在业务 crate 中添加依赖：

```toml
[dependencies]
arkit = { workspace = true }
```

WebView 需要额外启用 feature：

```toml
arkit = { workspace = true, features = ["webview"] }
```

## 最小应用

```rust
use arkit::prelude::*;
use arkit::{application, Element, Task};

#[derive(Default)]
struct State {
    count: i32,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Increment => {
            state.count += 1;
        }
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

## 构建示例

```sh
cd examples/counter
ohrs build --arch aarch
```

## 文档预览

```sh
pnpm install
pnpm run docs:dev
```
