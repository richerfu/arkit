---
title: Hooks 与生命周期
description: "hook 规则、effect、memo 与清理。"
---

# Hooks 与生命周期

Hook 把状态和资源绑定到组件 scope。调用顺序必须在每次 render 中保持一致，因此 Hook 不能放进条件、循环或提前返回之后。

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
