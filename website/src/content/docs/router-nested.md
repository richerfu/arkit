---
title: 嵌套路由
description: "嵌套路由、动态参数和查询串，以及页面外壳怎么拆。"
---

# 嵌套路由

复杂应用常常要嵌套布局：外壳不变，内层 Outlet 换页。参数和查询串也能从路由里类型化读出。

## 定义 Layout

```rust
#[derive(Routable, Clone, PartialEq, Debug)]
enum Route {
    #[layout(AppShell)]
        #[route("/")]
        Home {},
        #[route("/users/:id")]
        User { id: u32 },
    #[end_layout]

    #[route("/login")]
    Login {},
}

#[component]
fn AppShell() -> Element {
    rsx! {
        column {
            Navigation {}
            Outlet::<Route> {}
        }
    }
}
```

实际 derive attribute 语法跟随当前 `dioxus-router` 0.7；Arkit 不另建路由 DSL。

## 读取参数

route variant 字段就是已解析参数：

```rust
let route = use_route::<Route>();

if let Route::User { id } = route {
    // id 已是 u32
}
```

解析失败不会以一个半合法 Route 进入页面。对可选或复杂查询使用 upstream query API，并在页面入口转换为领域类型。

## Outlet Context

父 layout 可通过 `use_outlet_context` 向 child subtree 传递只属于该 shell 的依赖。应用级会话或主题仍应使用普通 Provider；不要把所有 Context 都绑到 Router。

## Key 与状态保留

父 layout identity 不变时，其 sidebar、缓存和 Hook 状态保留；child route 切换只替换 Outlet。需要按参数重新创建页面资源时，让 child component key 包含稳定 route identity，或让 `use_resource` 显式读取参数。

## Not Found

为不可解析路径和业务对象不存在分别设计页面。前者属于 router fallback，后者是合法 Route 的数据状态；不要把服务端 404 伪装成路由解析失败。
