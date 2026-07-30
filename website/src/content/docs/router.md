---
title: 路由概览
description: "类型化路由和 RouterProvider：页面导航从这里开始。"
---

# 路由概览

路由建立在 dioxus-router 上，并接好了 ArkUI 的 Link 和系统返回。先声明类型化 Route，再用 RouterProvider 包住应用。

## 定义 Route

```rust
use arkit::prelude::*;
use arkit::router::dioxus_router;

#[derive(Routable, Clone, PartialEq, Debug)]
enum Route {
    #[route("/")]
    Home {},

    #[route("/users/:id")]
    User { id: u32 },
}

#[entry]
fn app() -> Element {
    rsx! { Router::<Route> {} }
}
```

derive 生成 URL 解析与序列化。动态参数按字段类型解析，失败返回路由解析错误。

## 路由的所有权

Route enum 是页面身份与 URL 的单一来源。页面选择不要再复制成 `current_page: Signal<String>`；导航 UI 只派发 typed route，当前态从 router context 读取。

## API 分层

Arkit facade 直接导出稳定常用面：

- `Router`、`Routable`
- 原生 `Link`
- `use_back_handler`
- `RouteTransition`、`AnimatedOutlet`

`RouterConfig`、`RouterContext`、`Outlet`、`Navigator`、`use_route`、`use_navigator` 等完整 upstream API 从 `arkit::router::dioxus_router` 使用。

## 后续章节

“导航与历史栈”处理 Link、push/replace/back；“嵌套路由”处理 Outlet 和参数；“页面转场”说明视觉生命周期。转场不拥有 history，系统返回键和视觉动画始终围绕同一个 router state 工作。
