---
title: 异步任务
description: "通过当前 root 的 RuntimeHandle 调度异步任务与 UI 回调。"
---

# 异步任务

每个 `ArkRuntime` root 都有独立 `RuntimeHandle`。它把 Dioxus scheduler、OpenHarmony UI loop、Tokio runtime、back handler 和 embedded item runtime 收敛到同一个生命周期边界。

## 选择异步 Hook

| Hook / API              | 适用场景                                    |
| ----------------------- | ------------------------------------------- |
| `use_future`            | 挂载时启动一次，不直接返回缓存值            |
| `use_resource`          | 依赖 Signal / Context，保留 pending/value   |
| `use_coroutine`         | 长期 worker，通过 channel 接收多次输入      |
| `runtime.tokio()`       | timer、I/O 和要求 `Send` 的后台 future      |
| `runtime.queue_ui(...)` | 非 Dioxus callback 回到当前 root 的 UI tick |

## Resource 示例

```rust
let runtime = use_runtime_handle();
let async_runtime = runtime.tokio();
let user_id = use_signal(|| 42_u64);

let profile = use_resource(move || {
    let async_runtime = async_runtime.clone();
    async move {
        let id = user_id();
        async_runtime
            .spawn(async move { fetch_profile(id).await })
            .await
            .expect("profile task panicked")
    }
});
```

后台任务不应操作 UI-thread confined 的 ArkUI node、Drawing canvas 或 WebView 原生对象。

## 取消与卸载

Dioxus hook task 跟随 scope。直接 spawn 到 Tokio 的任务可能超出 scope，应保存 abort handle：

```rust
let task = use_hook(|| async_runtime.spawn(async { run_worker().await }));
use_drop(move || task.abort());
```

如果任务允许自然完成，确保它只返回 owned 数据，不再访问已卸载组件状态或 native handle。

## Native callback 回到 UI

```rust
let runtime = use_runtime_handle();
let mut value = use_signal(String::new);

register_native_callback(move |next| {
    runtime.queue_ui(move || value.set(next));
});
```

callback 只复制 owned payload 并入当前 root 队列，避免 native mutation 期间重入 Dioxus。root 卸载后队列会清空，handle 也不会唤醒其他 root。

`RuntimeHandle::register_back_handler` 返回 RAII registration；component 卸载即注销，不使用 process-global handler stack。
