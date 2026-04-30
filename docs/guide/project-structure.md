# 项目组织

Arkit 不要求固定目录。下面是推荐结构。

## 单页面

```text
src/
  lib.rs
```

适合 Demo 或简单页面。

## 多页面

```text
src/
  lib.rs
  app.rs
  message.rs
  pages/
    mod.rs
    home.rs
    detail.rs
  components/
    mod.rs
    product_card.rs
  services/
    mod.rs
    product.rs
  i18n.rs
locales/
  zh-CN.ftl
  en-US.ftl
```

| 文件 | 内容 |
| --- | --- |
| `lib.rs` | `#[entry]`。 |
| `app.rs` | 顶层 `State`、`update`、`view`。 |
| `message.rs` | 顶层 `Message`。 |
| `pages/` | 页面。 |
| `components/` | 可复用 UI。 |
| `services/` | 请求、存储、数据转换。 |
| `i18n.rs` | 国际化声明。 |

## lib.rs

```rust
mod app;
mod components;
mod message;
mod pages;
mod services;

use arkit::prelude::*;

#[entry]
fn app() -> impl arkit::EntryPoint {
    arkit::application(app::State::default, app::update, app::view)
}
```

## app.rs

```rust
use arkit::{Element, Task};
use crate::message::Message;

#[derive(Default)]
pub struct State {
    home: crate::pages::home::State,
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Home(message) => {
            crate::pages::home::update(&mut state.home, message)
                .map(Message::Home)
        }
    }
}

pub fn view(state: &State) -> Element<Message> {
    crate::pages::home::view(&state.home)
}
```

## message.rs

```rust
#[derive(Debug, Clone)]
pub enum Message {
    Home(crate::pages::home::Message),
}
```

## pages/home.rs

```rust
use arkit::prelude::*;
use arkit::{Element, Task};

#[derive(Default)]
pub struct State {
    count: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Increment => {
            state.count += 1;
        }
    }

    Task::none()
}

pub fn view(state: &State) -> Element<crate::message::Message> {
    column_component()
        .children(vec![
            text(format!("count = {}", state.count)).into(),
            button("increment")
                .on_press(crate::message::Message::Home(Message::Increment))
                .into(),
        ])
        .into()
}
```

## 拆分规则

| 情况 | 放到 |
| --- | --- |
| 页面变长 | `pages/` |
| UI 被复用 | `components/` |
| 请求或数据转换 | `services/` |
| 多语言文案 | `i18n.rs` + `locales/` |
| 多页面跳转 | Router |
