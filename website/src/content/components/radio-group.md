---
title: Radio Group
description: "单选选项组。"
---

# Radio Group

RadioGroup 在少量选项中选择一个值。当前 API 使用同一个字符串作为显示文本和值。

```rust
let mut plan = use_signal(|| "基础版".to_string());

RadioGroup {
    options: vec!["基础版".into(), "专业版".into()],
    selected: plan(),
    on_select: move |next| plan.set(next),
}
```

| 属性               | 类型                   | 默认值     | 说明         |
| ------------------ | ---------------------- | ---------- | ------------ |
| `options`          | `Vec<String>`          | 必填       | 候选项       |
| `selected`         | `Option<String>`       | `None`     | 受控选中值   |
| `default_selected` | `String`               | 空字符串   | 非受控初始值 |
| `on_select`        | `EventHandler<String>` | 默认空处理 | 选择回调     |

复杂领域对象应在页面层维护稳定 id 与文案的映射。选项较多或需要过滤时使用 Select/Combobox。
