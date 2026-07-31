---
title: 嵌入 WebView
description: "在页面里嵌原生 WebView：打开页面、收消息、离开时释放。"
---

# 嵌入 WebView

需要嵌网页时用原生 WebView，而不是再开一套 H5 容器框架。挂载、导航和消息回调都有对应 API；页面走了记得释放。

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

`EmbeddedWebViewController::set_visible` 可在 native WebView 创建前调用，期望值会保留到首次挂载。嵌入组件应组合应用与组件可见性，避免后台或隐藏页面继续展示/播放 Web 内容：

```rust
let host_ref = use_native_element_ref();
let visible = use_app_foreground() && use_component_visibility(host_ref.clone());
let lifecycle_controller = controller.clone();

use_effect(use_reactive(&visible, move |visible| {
    let _ = lifecycle_controller.set_visible(visible);
}));
```

## 挂载与同步

```rust
#[component]
fn WebViewArea() -> Element {
    let controller: EmbeddedWebViewController = use_context();
    let host_ref = use_native_element_ref();
    let controller_for_layout = controller.clone();
    let lease_ref = host_ref.clone();

    use_layout_frame(host_ref.clone(), move |frame| {
        if !frame.is_measured() {
            return;
        }
        let Some(host) = lease_ref.current() else {
            return;
        };

        let mut init = EmbeddedWebViewInit::url(
            "article-webview",
            "https://example.com",
        );
        init.javascript_enabled = Some(true);

        let _ = controller_for_layout.mount_or_sync(&host, init);
    });

    let controller_for_drop = controller.clone();
    use_drop(move || controller_for_drop.dispose());

    rsx! {
        stack {
            native_ref: host_ref,
            width: "100%",
            height: 400.0,
        }
    }
}
```

`mount_or_sync` 首次创建并 attach；后续调用会幂等确保 child attachment，并按需导航 URL/HTML。WebView 使用 `100%` 宽高跟随 native host 的布局约束；ArkTS 创建的外部节点不能通过 ArkUI Native Node API 写布局属性。

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
let runtime = use_runtime_handle();

init.on_title_change = Some(Rc::new(move |new_title| {
    runtime.queue_ui(move || {
        title.set(new_title);
    });
}));
```

回调来自 native/ArkTS 边界，不要在其中直接触发 Dioxus render。传给 `RuntimeHandle::queue_ui` 的 payload 必须 owned。

## Controller API

| 方法                      | 说明                        |
| ------------------------- | --------------------------- |
| `id`、`is_mounted`        | 查询 controller 状态        |
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
