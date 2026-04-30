# webview

路径：`examples/webview`

WebView 示例展示 `webview` feature 的使用。

## 启用 feature

```toml
arkit = { workspace = true, features = ["webview"] }
```

## 使用方式

- 创建 `WebViewController`。
- 用 `web_view` 或 `web_view_component` 创建节点。
- 通过 controller 同步 URL、加载状态和下载事件。
- 依赖入口宏 render 阶段设置的 OHOS helper/env。

## 构建

```sh
cd examples/webview
ohrs build --arch aarch
```
