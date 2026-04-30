# 页面路由

Router 用来管理页面跳转、返回栈、路由参数、404 页面和导航守卫。一个页面时不需要 Router；出现首页、详情页、设置页、登录页后再接入。

## 基本用法

应用 State 中保存一个 `Router`：

```rust
use arkit::router::Router;

#[derive(Clone)]
struct State {
    router: Router,
}

impl Default for State {
    fn default() -> Self {
        Self {
            router: Router::new("/"),
        }
    }
}
```

消息中增加 Router 分支：

```rust
use arkit::router::RouterMessage;

#[derive(Debug, Clone)]
enum Message {
    Router(RouterMessage),
}
```

在 `update` 中交给 Router 处理：

```rust
use arkit::router::RouterNavigationExt;

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Router(message) => {
            state.router.handle(message, Message::Router)
        }
    }
}
```

## 渲染当前路由

用 `RouterOutlet` 和 `Routes` 声明页面：

```rust
use arkit::router::{RouterOutlet, Routes};

fn view(state: &State) -> Element<Message> {
    RouterOutlet::new(
        state.router.clone(),
        Routes::new()
            .route("/", |_context| home_page())
            .route("/settings", |_context| settings_page())
            .route("/products/:id", |context| product_page(context))
            .fallback("*rest", |_context| not_found_page()),
    )
    .into()
}
```

`route` 的闭包返回一个 `Element<Message>`。`context` 可以读取当前路由信息。

## 跳转

在按钮或其他交互中发送 `RouterMessage`：

```rust
button("设置")
    .on_press(Message::Router(RouterMessage::push("/settings")))
```

跳到详情页：

```rust
button("查看")
    .on_press(Message::Router(RouterMessage::push(format!("/products/{id}"))))
```

常用导航：

```rust
RouterMessage::push("/settings")
RouterMessage::replace("/login")
RouterMessage::reset("/")
RouterMessage::back()
```

## 读取参数和查询

路由 `/products/:id` 可以读取 `id`：

```rust
fn product_page(context: RouteContext) -> Element<Message> {
    let id = context.param("id").unwrap_or_default();

    text(format!("product id = {id}")).into()
}
```

需要解析成数字：

```rust
let id: u32 = context.parse_param("id").unwrap_or_default();
```

查询参数：

```rust
let tab = context.query("tab").unwrap_or("overview");
```

## 携带导航状态

如果不想把所有数据放进 URL，可以使用 state：

```rust
#[derive(Clone)]
struct ProductNavState {
    source: String,
}

RouterMessage::push_with_state(
    "/products/42",
    ProductNavState {
        source: "home".into(),
    },
)
```

页面中读取：

```rust
let source = context
    .state::<ProductNavState>()
    .map(|state| state.source.as_str())
    .unwrap_or("unknown");
```

## 嵌套路由

嵌套路由适合“用户页 + 用户设置页”这类共享布局：

```rust
Routes::new().nest(
    "/users/:id",
    |context, outlet| user_layout(context, outlet.into()),
    |users| {
        users
            .index(|context| user_page(context))
            .route("settings", |context| user_settings_page(context))
    },
)
```

父级页面通过 `outlet` 显示子页面。

## 导航守卫

需要登录、权限或草稿保存判断时使用 guard：

```rust
Routes::new().guard(
    |context| {
        if is_logged_in() {
            RouteGuardDecision::Allow
        } else {
            RouteGuardDecision::Redirect("/login".into())
        }
    },
    |routes| routes.route("/profile", |context| profile_page(context)),
)
```

异步守卫：

```rust
Routes::new().guard_async(
    |context| async move {
        if check_permission(context.to.path()).await {
            RouteGuardDecision::Allow
        } else {
            RouteGuardDecision::Block("没有权限".into())
        }
    },
    |routes| routes.route("/admin", |context| admin_page(context)),
)
```

## 系统返回键

让系统返回键优先执行路由返回：

```rust
#[entry]
fn app() -> impl arkit::EntryPoint {
    application(State::default, update, view)
        .on_back_press(|state| state.router.handle_system_back(Message::Router))
}
```

如果路由不能再返回，会交给系统处理。

## API 摘要

| API | 用途 |
| --- | --- |
| `Router::new("/")` | 创建路由状态。 |
| `RouterOutlet::new(router, routes)` | 渲染当前路由匹配的页面。 |
| `Routes::new().route(path, render)` | 声明页面路由。 |
| `Routes::new().fallback("*rest", render)` | 声明 404 页面。 |
| `RouterMessage::push(path)` | 进入新页面。 |
| `RouterMessage::replace(path)` | 替换当前页面。 |
| `RouterMessage::reset(path)` | 重置页面栈。 |
| `RouterMessage::back()` | 返回上一页。 |
