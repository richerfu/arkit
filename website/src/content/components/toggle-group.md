---
title: Toggle Group
description: "成组互斥或多选切换。"
---

# Toggle Group

ToggleGroup 管理一组互斥或多选的 Toggle，适合文本对齐、视图模式等紧凑工具栏。

```rust
let mut align = use_signal(|| vec!["left".to_string()]);

ToggleGroup {
    options: vec!["left".into(), "center".into(), "right".into()],
    selected: align(),
    multi: false,
    icons: true,
    on_change: move |next| align.set(next),
}
```

| 属性               | 类型                        | 默认值     | 说明                 |
| ------------------ | --------------------------- | ---------- | -------------------- |
| `options`          | `Vec<String>`               | 必填       | 候选项               |
| `selected`         | `Option<Vec<String>>`       | `None`     | 受控选择集合         |
| `default_selected` | `Vec<String>`               | 空         | 非受控初始集合       |
| `multi`            | `bool`                      | `false`    | 是否允许多选         |
| `icons`            | `bool`                      | `false`    | 将选项按图标名称渲染 |
| `on_change`        | `EventHandler<Vec<String>>` | 默认空处理 | 选择集合变化         |

`multi: false` 时最多保留一个值。业务数据应使用稳定 id；若选项需要说明文字或数量较多，改用 RadioGroup、Checkbox 列表或 Select。
