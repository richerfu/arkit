# async_task

路径：`examples/async_task`

展示 `use_resource` 依赖 request-id signal，并通过框架 Tokio runtime 等待定时器。future 完成后 Dioxus scheduler 会唤醒 OpenHarmony UI loop，不需要轮询或额外 Message 通道。

```sh
cd examples/async_task
ohrs build --arch aarch
```
