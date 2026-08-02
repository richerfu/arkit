---
title: Hooks 与生命周期
description: "Hook 的使用规则，以及 effect、memo 和清理时机。"
---

# Hooks 与生命周期

Hook 的规则和 React 类似：只在组件顶层调用，顺序保持稳定。effect 负责订阅和副作用，清理函数在依赖变了或组件卸载时跑。

## 常用 Hooks

| Hook            | 用途                        |
| --------------- | --------------------------- |
| `use_signal`    | 响应式可变状态              |
| `use_memo`      | 根据读取依赖缓存派生值      |
| `use_effect`    | 依赖变化后的同步副作用      |
| `use_future`    | scope 挂载时启动 future     |
| `use_resource`  | 依赖变化时重跑异步计算      |
| `use_coroutine` | 接收多次输入的长期 worker   |
| `use_hook`      | 创建非响应式 scope-owned 值 |
| `use_drop`      | scope 卸载清理              |

## 应用前后台生命周期

`#[entry]` 会把 Ability、WindowStage 和 Surface 回调归一化为根 Context。组件可直接读取响应式快照：

```rust
let lifecycle = use_application_lifecycle();

if lifecycle.is_foreground() {
    // 只在前台持有相机、传感器等独占或高功耗资源。
}
```

`use_app_foreground()` 是只关心前后台的简写。`Pause`、`Hidden`、window destroy 和 ability destroy 会立即变为 `false`；只有窗口仍然存在且可见的 `Resume`/`Shown` 才会恢复为 `true`。单独失焦不会被误判为后台，系统弹窗或临时焦点切换因此不会无故重建资源。

需要区分低内存、焦点、窗口和 Surface 事件时，使用 scope-owned 订阅：

```rust
use_application_lifecycle_event(move |event, state| {
    match event {
        ApplicationLifecycleEvent::LowMemory => release_memory_cache(),
        ApplicationLifecycleEvent::FocusLost => pause_input(),
        _ => {}
    }
    println!("lifecycle={event:?}, foreground={}", state.is_foreground());
});
```

订阅随组件 scope 自动注销，不保留悬空回调。

## 组件展示与隐藏

组件仍然挂载，但因 `visibility`、父级裁剪、Tab 切换或移出可见区域而不可见时，Dioxus 的 `use_drop` 不会执行。此时给目标元素绑定精确 ref：

```rust
let reference = use_native_element_ref();
let component = use_component_lifecycle(reference.clone());
let should_run = use_app_foreground() && component.is_visible();

use_effect(use_reactive(&should_run, move |active| {
    player.set_active(active);
}));

rsx! {
    stack {
        native_ref: reference,
        PlayerSurface {}
    }
}
```

快照包含 `visible` 和 ArkUI 上报的 `visible_fraction`；`use_component_visibility(reference)` 是布尔简写。同一精确元素的多个订阅共享 renderer 的 native event route，卸载时按 RAII token 清理。

组件创建和销毁已经由 scope 生命周期完整表达，因此不再增加一套重复的 `on_create`/`on_destroy` API：首次 `use_effect` 负责创建或订阅，`use_drop` 负责销毁；展示/隐藏由上述组件生命周期 Hook 补齐。

## Effect

```rust
let query = use_signal(String::new);

use_effect(move || {
    let current = query();
    tracing::debug!(%current, "query changed");
});
```

Effect 内读取的 Signal 建立依赖。Effect 用于把状态同步到外部系统，不用于计算本可由 `use_memo` 得到的 UI 值。

## 创建与清理资源

```rust
let subscription = use_hook(subscribe_to_native_source);
use_drop(move || subscription.unregister());
```

订阅、native handle、timer 和 abort handle 必须由同一 scope 完整清理。不要把清理注册在可能反复执行的条件分支。

## 自定义 Hook

自定义 Hook 把一组状态与生命周期约束封装成一个入口：

```rust
fn use_toggle(initial: bool) -> Signal<bool> {
    let value = use_signal(|| initial);
    value
}
```

名称使用 `use_` 前缀，并在内部遵守固定调用顺序。返回稳定 handle 或明确的数据结构，不暴露只能在内部安全操作的 native 裸指针。

## 组件卸载

卸载会丢弃 scope、取消 Dioxus 管理的任务并执行 drop hook。额外 spawn 到 Tokio 的任务不一定自动停止；其取消策略见“异步任务”。
