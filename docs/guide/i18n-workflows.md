# 国际化

Arkit 使用 `.ftl` 文件管理文案，并生成类型安全的 Rust API。

## 文案文件

```text
locales/
  zh-CN.ftl
  en-US.ftl
```

`zh-CN.ftl`：

```text
home-title = 商品
refresh = 刷新
load-failed = 加载失败：{$reason}
```

`en-US.ftl`：

```text
home-title = Products
refresh = Refresh
load-failed = Failed to load: {$reason}
```

## 声明模块

```rust
arkit::i18n::i18n! {
    pub mod tr {
        path: "locales",
        fallback: "zh-CN",
        locales: ["zh-CN", "en-US"],
    }
}
```

## 保存语言状态

```rust
struct State {
    i18n: tr::I18n,
}

impl Default for State {
    fn default() -> Self {
        Self {
            i18n: tr::I18n::default(),
        }
    }
}
```

## 使用文案

```rust
fn view(state: &State) -> Element<Message> {
    column_component()
        .children(vec![
            text(state.i18n.tr(tr::home_title())).into(),
            button(state.i18n.tr(tr::refresh()))
                .on_press(Message::Refresh)
                .into(),
        ])
        .into()
}
```

带参数：

```rust
text(state.i18n.tr(tr::load_failed(reason)))
```

## 切换语言

```rust
enum Message {
    LocaleChanged(tr::Locale),
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::LocaleChanged(locale) => {
            state.i18n.set_locale(locale);
            Task::none()
        }
    }
}
```

## 规则

- `fallback` 必须包含在 `locales` 中。
- 所有语言文件必须包含相同 key。
- 同一个 key 的参数必须一致。
- `home-title` 会生成 `tr::home_title()`。
- 缺少参数会在编译期报错。
