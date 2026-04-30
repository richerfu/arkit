# async_task

路径：`examples/async_task`

该示例展示 `Task::perform`。它适合参考异步加载、延迟操作、网络请求完成后更新 UI 等场景。

## 使用方式

典型流程：

1. UI 事件产生 `Message::Start`。
2. `update` 设置 loading 状态。
3. `update` 返回 `Task::perform(future, Message::Finished)`。
4. runtime 后台线程执行 future。
5. future 完成后唤醒 UI loop。
6. `Message::Finished` 回到 update，写入结果并关闭 loading。

## 开发注意

- future 必须是 `Send + 'static`。
- 不要在 future 中直接改 State。
- 错误应映射进 Message，例如 `Message::Finished(Result<T, E>)`。
