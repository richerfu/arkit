# WebView

WebView 是 native escape hatch：Dioxus 负责声明式占位与状态，`use_layout_frame_node` 在布局完成后把真实 WebView 挂到对应 ArkUI host node。

核心对象是 `EmbeddedWebViewController`：

```rust
let webview = use_context_provider(|| EmbeddedWebViewController::new("business-webview"));

use_layout_frame_node(move |mut host, frame| {
    if !frame.is_measured() {
        return;
    }
    let init = EmbeddedWebViewInit::url("business-webview", "https://example.com");
    let _ = webview.mount_or_sync(
        &mut host,
        init,
        Some(WebViewFrame { width: frame.width, height: frame.height }),
    );
});
```

controller 还提供 `load_url`、`load_html`、`reload`、`focus`、`set_zoom`、`dispose` 和可返回 teardown 错误的 `try_dispose`。涉及 native 回调的状态更新使用 facade 导出的 `arkit::queue_ui_loop` 回到 UI loop。URL/HTML 只有在 native navigation 成功后才提交到 controller snapshot；从 HTML 切回 URL 会执行真实 load，而不是只更新 Rust 状态，未提供新内容时保留当前页面 snapshot。

完整交互实现见 `examples/webview/src/lib.rs`。
