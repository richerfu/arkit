# 应用模型

Arkit 应用由 `State`、`Message`、`update`、`view` 组成。

## State

保存页面数据。

```rust
#[derive(Default)]
struct State {
    loading: bool,
    items: Vec<Item>,
    error: Option<String>,
}
```

## Message

描述事件。

```rust
#[derive(Debug, Clone)]
enum Message {
    Load,
    Loaded(Result<Vec<Item>, String>),
    ItemPressed(String),
}
```

## update

处理事件，修改 State，返回 Task。

```rust
fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Load => {
            state.loading = true;
            Task::perform(load_items(), Message::Loaded)
        }
        Message::Loaded(result) => {
            state.loading = false;
            match result {
                Ok(items) => state.items = items,
                Err(error) => state.error = Some(error),
            }
            Task::none()
        }
        Message::ItemPressed(_id) => Task::none(),
    }
}
```

## view

根据 State 生成页面。

```rust
fn view(state: &State) -> Element<Message> {
    if state.loading {
        return text("loading").into();
    }

    column_component()
        .children(
            state
                .items
                .iter()
                .map(|item| {
                    button(&item.title)
                        .on_press(Message::ItemPressed(item.id.clone()))
                        .into()
                })
                .collect(),
        )
        .into()
}
```

## 启动任务

```rust
#[entry]
fn app() -> impl arkit::EntryPoint {
    application(State::default, update, view)
        .boot(|| (State::default(), Task::done(Message::Load)))
}
```

## 返回键

```rust
#[entry]
fn app() -> impl arkit::EntryPoint {
    application(State::default, update, view)
        .on_back_press(|state| {
            if state.dialog_open {
                BackPressDecision::message(Message::CloseDialog)
            } else {
                BackPressDecision::pass_through()
            }
        })
}
```

## API

| API | 说明 |
| --- | --- |
| `application(boot, update, view)` | 创建应用。 |
| `Task::none()` | 无后续消息。 |
| `Task::done(message)` | 立即发送一个消息。 |
| `Task::perform(future, map)` | 执行异步任务。 |
| `BackPressDecision::pass_through()` | 返回键交给系统。 |
| `BackPressDecision::message(message)` | 拦截返回键并发送消息。 |
