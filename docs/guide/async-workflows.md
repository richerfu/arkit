# 异步任务

使用 Dioxus 的异步 hooks，不使用独立 Task/Message 调度器。

```rust
let handle = arkit::tokio_handle();
let result = use_resource(move || {
    let handle = handle.clone();
    async move {
        handle.spawn(async {
            tokio::time::sleep(Duration::from_millis(800)).await;
        }).await.ok();
        "done".to_string()
    }
});
```

`arkit_runtime` 把 Dioxus scheduler waker 接到 OpenHarmony UI loop。future 在后台线程完成后会主动唤醒 UI，随后 `VirtualDom::render_immediate` 应用 ready mutations。

选择原则：

- 依赖响应式输入并缓存结果：`use_resource`
- 启动一次 future：`use_future`
- 接收多次输入：`use_coroutine`
- Tokio timer/I/O：通过 `arkit::tokio_handle()` 进入框架 Tokio runtime

完整代码见 `examples/async_task/src/lib.rs`。
