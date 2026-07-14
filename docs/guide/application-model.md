# 应用模型

Arkit 应用就是 Dioxus 应用：组件返回 `Element`，signal 保存响应式状态，event handler 修改状态，hooks 管理副作用。

```rust
use arkit::prelude::*;

#[entry]
fn app() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        button {
            onclick: move |_| count += 1,
            "count = {count}"
        }
    }
}
```

## 组件

使用 `#[component]` 建立组件边界。不要直接调用包含 hooks 的组件函数；在 `rsx!` 中挂载组件，让 Dioxus 管理 scope、props memoization 和生命周期。

## 状态

- 局部可变状态：`use_signal`
- 派生值：`use_memo`
- 跨树共享：`use_context_provider` / `use_context`
- 副作用与清理：`use_effect` / `use_drop`
- 异步：`use_resource` / `use_future` / `use_coroutine`

Arkit 不再提供 `State + Message + update + Task` 的第二套 runtime。

## 安全区

应用默认由框架根节点避让系统安全区，不需要在页面或组件中硬编码状态栏、导航栏高度。需要读取当前窗口状态时使用：

```rust
let metrics = use_window_metrics();
let safe = use_safe_area();
```

嵌套区域可以使用 `SafeArea { edges: SafeAreaEdges::BOTTOM, ... }`。沉浸式应用使用 `#[entry(edge_to_edge)]` 取消业务根节点的默认安全 padding；系统浮层仍会避让安全区。
