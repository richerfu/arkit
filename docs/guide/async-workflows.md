# 异步任务

`Task` 用于在 `update` 中描述后续工作。

## 无任务

```rust
Task::none()
```

## 立即发送消息

```rust
Task::done(Message::Load)
```

## 执行异步任务

```rust
Task::perform(fetch_user(id), Message::UserLoaded)
```

完整示例：

```rust
#[derive(Debug, Clone)]
enum Message {
    LoadUser(String),
    UserLoaded(Result<User, String>),
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::LoadUser(id) => {
            state.loading = true;
            Task::perform(fetch_user(id), Message::UserLoaded)
        }
        Message::UserLoaded(result) => {
            state.loading = false;
            match result {
                Ok(user) => state.user = Some(user),
                Err(error) => state.error = Some(error),
            }
            Task::none()
        }
    }
}
```

## 批量任务

```rust
Task::batch([
    Task::done(Message::LoadProfile),
    Task::done(Message::LoadOrders),
])
```

## 推荐状态

```rust
struct State {
    loading: bool,
    data: Option<Data>,
    error: Option<String>,
}
```

不要只用 `Option<Data>` 表示加载状态。

## API

| API | 说明 |
| --- | --- |
| `Task::none()` | 不产生后续消息。 |
| `Task::done(message)` | 立即产生消息。 |
| `Task::perform(future, map)` | future 完成后用 `map` 转成消息。 |
| `Task::batch(tasks)` | 合并多个任务。 |
