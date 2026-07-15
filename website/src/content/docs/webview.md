---
title: 嵌入 WebView
---

# 嵌入 WebView

WebView 是 native escape hatch：Dioxus 声明占位与业务状态，`use_layout_frame_node` 在布局完成后把 ArkTS 创建的 WebView 原生节点挂到对应 ArkUI host。

## Controller 生命周期

```rust
const WEBVIEW_ID: &str = "article-webview";

#[entry]
fn app() -> Element {
    use_context_provider(|| EmbeddedWebViewController::new(WEBVIEW_ID));
    rsx! { WebViewArea {} }
}
```

controller id 在首次 mount 后不可变化。一个 controller 对应一个 embedded WebView；clone 共享同一 mounted state。

## 挂载与同步

```rust
#[component]
fn WebViewArea() -> Element {
    let controller: EmbeddedWebViewController = use_context();
    let controller_for_layout = controller.clone();

    use_layout_frame_node(move |mut host, frame| {
        if !frame.is_measured() {
            return;
        }

        let mut init = EmbeddedWebViewInit::url(
            "article-webview",
            "https://example.com",
        );
        init.javascript_enabled = Some(true);

        let _ = controller_for_layout.mount_or_sync(
            &mut host,
            init,
            Some(WebViewFrame {
                width: frame.width,
                height: frame.height,
            }),
        );
    });

    let controller_for_drop = controller.clone();
    use_drop(move || controller_for_drop.dispose());

    rsx! {
        stack {
            percent_width: 1.0,
            height: 400.0,
        }
    }
}
```

`mount_or_sync` 首次创建并 attach；后续调用会幂等确保 child attachment、按需导航 URL/HTML 并同步 frame。`WebViewFrame::is_valid` 要求有限且宽高大于零。

## 初始化选项

`EmbeddedWebViewInit` 支持：

| 字段                     | 说明                           |
| ------------------------ | ------------------------------ |
| `id`                     | 稳定、非空的 controller id     |
| `url` / `html`           | 初始 URL 或 HTML               |
| `style`                  | `WebViewStyle`，可见性、背景等 |
| `javascript_enabled`     | JavaScript 开关                |
| `devtools`               | 调试开关                       |
| `user_agent`             | 自定义 UA                      |
| `autoplay`               | 媒体自动播放                   |
| `initialization_scripts` | 页面初始化脚本                 |
| `headers`                | 初始请求 header                |
| `transparent`            | 透明背景                       |
| `on_navigation_request`  | 返回 bool 的导航拦截           |
| `on_title_change`        | 标题变化回调                   |

`EmbeddedWebViewInit::url(id, url)` 提供安全默认值。需要 HTML 时构造 struct 并令 `url = None`。

## Native callback 回到 Dioxus

```rust
let mut title = use_signal(|| String::from("loading"));

init.on_title_change = Some(Rc::new(move |new_title| {
    queue_ui_loop(move || {
        title.set(new_title);
    });
}));
```

回调来自 native/ArkTS 边界，不要在其中直接触发 Dioxus render。传给 `queue_ui_loop` 的 payload 必须 owned。

## Controller API

| 方法                      | 说明                        |
| ------------------------- | --------------------------- |
| `id`、`is_mounted`        | 查询 controller 状态        |
| `sync_frame`              | 只更新 native node frame    |
| `load_url`                | 导航 URL                    |
| `load_html`               | 加载 HTML                   |
| `reload`、`focus`         | 页面控制                    |
| `set_zoom`                | 设置缩放                    |
| `clear_all_browsing_data` | 清理 WebView 数据           |
| `with_webview`            | 对底层 `Webview` 执行窄操作 |
| `dispose`                 | best-effort 清理并记录错误  |
| `try_dispose`             | 返回 teardown 错误          |

URL/HTML snapshot 只在 native load 成功后更新。从 HTML 切回 URL 会真实调用 `load_url`；没有提供新内容时保留当前页面。

## 错误与清理

- attach native child 失败时立即 dispose ArkTS WebView controller。
- `try_dispose` 只有在 native cleanup 成功后才清空 Rust mounted snapshot，便于诊断和重试。
- controller 最后一个 clone drop 会 best-effort dispose，但业务仍应在 `use_drop` 明确清理。
- 不把 controller 移到后台线程。
- WebView 创建依赖 `NativeAbility` render 流程安装的 helper 和 main-thread N-API env。

## 验证

`examples/webview` 覆盖 URL 输入、reload、focus、zoom、title callback 和页面切换：

```sh
cd examples/webview
ohrs build --arch aarch
```

真机还应检查离开页面后 WebView 销毁、返回页面重新 mount、旋转/resize 后 frame 同步，以及导航拦截不会重入 Dioxus。
