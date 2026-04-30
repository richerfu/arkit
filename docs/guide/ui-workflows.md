# UI 组件

## 基础组件

常用 API：

```rust
text("标题")
button("提交")
image("...")
text_input("")
checkbox(false)
list_component()
grid_component()
```

布局：

```rust
column_component()
row_component()
stack_component()
scroll_component()
```

示例：

```rust
column_component()
    .percent_width(1.0)
    .padding(16.0)
    .children(vec![
        text("登录").font_size(24.0).into(),
        text_input("")
            .margin_top(16.0)
            .into(),
        button("提交")
            .margin_top(16.0)
            .on_press(Message::Submit)
            .into(),
    ])
    .into()
```

## 样式

```rust
text("商品")
    .font_size(20.0)
    .font_color(0xFF111827)
    .line_height(28.0)
```

```rust
button("刷新")
    .padding([8.0, 12.0, 8.0, 12.0])
    .margin_top(12.0)
    .border_radius(8.0)
```

## 抽组件

重复 UI 可抽成普通函数：

```rust
fn section_title<Message: 'static>(title: impl Into<String>) -> Element<Message> {
    text(title.into())
        .font_size(20.0)
        .line_height(28.0)
        .into()
}
```

带事件：

```rust
fn primary_button<Message>(label: impl Into<String>, message: Message) -> Element<Message>
where
    Message: Clone + 'static,
{
    button(label.into())
        .padding([8.0, 12.0, 8.0, 12.0])
        .on_press(message)
        .into()
}
```

## arkit_shadcn

引入：

```rust
use arkit_shadcn::prelude::*;
```

按钮：

```rust
Button::new("提交")
    .variant(ButtonVariant::Default)
    .size(ButtonSize::Default)
    .on_press(Message::Submit)
```

带图标：

```rust
Button::with_icon("保存", "save")
    .on_press(Message::Save)
```

卡片：

```rust
Card::new(vec![
    CardHeader::new("标题", "描述").into(),
    CardContent::new(vec![
        Text::new("内容").into(),
    ]).into(),
])
```

主题：

```rust
arkit_shadcn::theme::with_theme(
    arkit_shadcn::theme::Theme::default(),
    || {
        Card::new(vec![
            Text::new("商品").into(),
            Button::new("刷新")
                .on_press(Message::Refresh)
                .into(),
        ])
        .into()
    },
)
```

## 选择

| 需求 | 使用 |
| --- | --- |
| 快速写页面 | 基础组件 |
| 复用 UI | 函数组件 |
| 统一视觉风格 | `arkit_shadcn` |
