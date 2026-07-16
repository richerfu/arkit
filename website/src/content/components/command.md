---
title: Command
description: "命令搜索与执行列表。"
---

# Command

Command 是可检索的命令列表，适合命令面板、页面跳转和快捷操作，不用于普通表单值选择。

```rust
let mut query = use_signal(String::new);

Command {
    query: query(),
    options: vec!["打开设置".into(), "新建项目".into()],
    on_query_change: move |next| query.set(next),
}
```

| 属性              | 类型                           | 默认值   | 说明         |
| ----------------- | ------------------------------ | -------- | ------------ |
| `query`           | `String`                       | 空字符串 | 当前查询文本 |
| `options`         | `Vec<String>`                  | 必填     | 命令文案列表 |
| `on_query_change` | `Option<EventHandler<String>>` | `None`   | 查询变化回调 |

命令需要稳定 identity 时，在页面层维护 id 到文案/handler 的映射。执行后明确关闭外层 Dialog/Popover；破坏性命令应先进入 AlertDialog 确认。
