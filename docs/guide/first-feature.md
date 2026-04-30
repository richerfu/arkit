# 业务功能示例

示例：商品列表。

## 数据类型

```rust
#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub title: String,
    pub price_text: String,
}
```

## State

```rust
#[derive(Default)]
pub struct State {
    products: Vec<Product>,
    loading: bool,
    error: Option<String>,
}
```

## Message

```rust
#[derive(Debug, Clone)]
pub enum Message {
    Load,
    Refresh,
    Loaded(Result<Vec<Product>, String>),
    ProductPressed(String),
}
```

## update

```rust
pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Load | Message::Refresh => {
            state.loading = true;
            state.error = None;
            Task::perform(load_products(), Message::Loaded)
        }
        Message::Loaded(Ok(products)) => {
            state.loading = false;
            state.products = products;
            Task::none()
        }
        Message::Loaded(Err(error)) => {
            state.loading = false;
            state.error = Some(error);
            Task::none()
        }
        Message::ProductPressed(_id) => Task::none(),
    }
}
```

## 数据加载

```rust
async fn load_products() -> Result<Vec<Product>, String> {
    Ok(vec![
        Product {
            id: "p-1".into(),
            title: "ArkUI 开发指南".into(),
            price_text: "¥ 99".into(),
        },
    ])
}
```

## view

```rust
pub fn view(state: &State) -> Element<Message> {
    if state.loading {
        return text("加载中...").into();
    }

    if let Some(error) = &state.error {
        return text(format!("加载失败：{error}")).into();
    }

    let mut children = vec![
        button("刷新").on_press(Message::Refresh).into(),
    ];

    for product in &state.products {
        children.push(product_row(product).into());
    }

    column_component()
        .padding(16.0)
        .children(children)
        .into()
}
```

## 列表项

```rust
fn product_row(product: &Product) -> arkit::RowElement<Message> {
    row_component()
        .percent_width(1.0)
        .padding(12.0)
        .children(vec![
            text(&product.title).into(),
            text(&product.price_text).margin_left(8.0).into(),
            button("查看")
                .margin_left(12.0)
                .on_press(Message::ProductPressed(product.id.clone()))
                .into(),
        ])
}
```

## 启动加载

```rust
#[entry]
fn app() -> impl arkit::EntryPoint {
    application(State::default, update, view)
        .boot(|| (State::default(), Task::done(Message::Load)))
}
```
