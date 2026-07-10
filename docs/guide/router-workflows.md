# 页面路由

Arkit 直接复用 `dioxus-router`。路由 enum、Outlet、navigator 和组件 scope 均由 Dioxus 管理。

```rust
#[derive(Routable, Clone, PartialEq, Debug)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/users/:id")]
    Users { id: u32 },
}

#[entry]
fn app() -> Element {
    rsx! { Router::<Route> {} }
}
```

页面使用 ArkUI 原生 `Link`：

```rust
Link { to: Route::Users { id: 42 }, "User 42" }
```

`RouteTransition` 和 `AnimatedOutlet` 只包装页面过渡，不接管路由状态。需要系统返回键时，在 Router 树内调用 `use_back_handler()`；hook 卸载时会自动清理 handler。

完整代码见 `examples/router/src/lib.rs`。
