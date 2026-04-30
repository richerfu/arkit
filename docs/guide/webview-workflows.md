# WebView

WebView 用于加载网页或 H5 页面。

## 启用

```toml
[dependencies]
arkit = { workspace = true, features = ["webview"] }
```

## 基本使用

```rust
use arkit::prelude::*;

fn view(_state: &State) -> Element<Message> {
    web_view("https://example.com")
        .percent_width(1.0)
        .percent_height(1.0)
        .into()
}
```

## 与页面状态结合

```rust
struct State {
    url: String,
}

fn view(state: &State) -> Element<Message> {
    column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![
            button("关闭")
                .on_press(Message::Close)
                .into(),
            web_view(&state.url)
                .percent_width(1.0)
                .height(Length::Fill)
                .into(),
        ])
        .into()
}
```

## API

| API | 说明 |
| --- | --- |
| `web_view(url)` | 创建 WebView。 |
| `web_view_component()` | 创建可继续配置的 WebView 组件。 |
| `WebViewController` | 控制 WebView。 |

## 建议

- 原生页面优先使用 Arkit 组件。
- H5、活动页、协议页使用 WebView。
- WebView 需要 `webview` feature。
