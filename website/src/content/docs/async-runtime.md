---
title: 异步任务
description: "异步任务如何回到 UI 线程，以及取消和依赖更新要注意什么。"
---

# 异步任务

Runtime 把 Dioxus 的调度器和 OpenHarmony 的 UI loop 接在一起。Future 完成后会通过 waker 把 UI 叫醒，不需要你自己轮询。

## 选择异步 Hook

| Hook            | 适用场景                                     |
| --------------- | -------------------------------------------- |
| `use_future`    | 挂载时启动一次，不直接返回缓存值             |
| `use_resource`  | 依赖 Signal/Context，保留 pending/value 状态 |
| `use_coroutine` | 长期 worker，通过 channel 接收多次输入       |
| `tokio_handle`  | timer、I/O 和要求 `Send` 的后台 future       |

## Resource 示例

```rust
let handle = arkit::tokio_handle();
let user_id = use_signal(|| 42_u64);
let profile = use_resource(move || {
    let handle = handle.clone();
    async move {
        let id = user_id();
        handle.spawn(async move { fetch_profile(id).await })
            .await
            .expect("profile task panicked")
    }
});
```

`tokio_handle()` 只在已挂载的 `ArkRuntime` 内可用。后台任务不应操作 UI-thread confined 的 ArkUI node、Drawing canvas 或 WebView 原生对象。

## 错误建模

```rust
let result = use_resource(move || async move {
    fetch_profile().await.map_err(|error| error.to_string())
});
```

在 RSX 中分别渲染 pending、success 和 error。`JoinError` 表示任务 panic/取消，不等同业务 I/O 错误。

## 取消与卸载

Dioxus 管理的 hook task 跟随 scope。直接 spawn 到 Tokio 的任务可能超出 scope，应保存 abort handle：

```rust
let task = use_hook(|| handle.spawn(async { run_worker().await }));
use_drop(move || task.abort());
```

如果任务允许自然完成，确保它只返回 owned 数据，不再访问已卸载组件的状态或 native handle。

## Native 回调回到 UI

原生 SDK callback 不在 Dioxus dispatch 内时，用 `queue_ui_loop` 安排下一 UI tick。它既避免 native patch 期间重入，也会唤醒 scheduler。

## 验证

`examples/async_task` 展示 Tokio timer 完成后自动刷新 UI：

```sh
cd examples/async_task
ohrs build --arch aarch
```
