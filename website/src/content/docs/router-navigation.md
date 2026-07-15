---
title: 导航与历史栈
---

# 导航与历史栈

Arkit 的原生 `Link` 渲染 ArkUI `text`，点击后向当前 Navigator push typed route。它不是 Web anchor。

## Link

```rust
Link {
    to: Route::User { id: 42 },
    color: "#ff2563eb",
    font_size: 18.0,
    "用户 42"
}
```

需要卡片式点击区域时使用普通 ArkUI 事件，并调用 Navigator；Link 适合文本导航语义。

## 程序化导航

```rust
use arkit::router::dioxus_router::prelude::*;

let navigator = use_navigator();

rsx! {
    row {
        button {
            onclick: move |_| navigator.push(Route::Settings {}),
            "打开设置"
        }
        button {
            onclick: move |_| navigator.replace(Route::Login {}),
            "替换当前页"
        }
    }
}
```

`push` 增加 history entry；`replace` 替换当前 entry；`go_back` 返回；`can_go_back` 可用于按钮禁用态。

## 系统返回键

在 Router tree 靠近 root 安装一次：

```rust
#[component]
fn AppShell() -> Element {
    let _back = use_back_handler();
    rsx! { Outlet::<Route> {} }
}
```

有历史记录时 handler 调用 `go_back` 并消费事件；到根时返回 false，让系统继续处理。registration 跟随 scope，卸载旧 Router 不会误删新 handler。

## 避免重复导航

提交按钮在请求完成前禁用，防止连续 push 同一页面。外部 deep link 先解析为 Route，再选择 push 或 replace；不要让 ArkTS 直接维护另一份 Rust history。

## 页面状态

必须可分享/恢复的筛选条件进入 route params 或 query；短期输入、滚动位置保留在页面 scope。选择依据是“是否属于页面地址”，而不是数据大小。
