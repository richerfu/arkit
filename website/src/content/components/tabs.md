---
title: Tabs
---

# Tabs

Tabs 表达同一页面内的视图切换。完整组件接收标签和面板列表，也可以用四个 primitive 自行组合。

```rust
let panels = vec![
    rsx! { ProfileForm {} },
    rsx! { SecuritySettings {} },
];

Tabs {
    labels: vec!["资料".into(), "安全".into()],
    panels,
    default_active: 0,
    on_change: move |index| tracing::debug!(index),
}
```

| 组件          | 主要属性                                                    | 用途                  |
| ------------- | ----------------------------------------------------------- | --------------------- |
| `Tabs`        | `labels`、`panels`、`active`、`default_active`、`on_change` | 完整受控/非受控标签页 |
| `TabsList`    | `children`                                                  | trigger 容器          |
| `TabsTrigger` | `label`、`active`、`on_press`                               | 单个标签              |
| `TabsContent` | `active`、`children`                                        | 单个面板              |

`labels` 与 `panels` 应保持相同顺序和数量。需要可分享 URL 或系统返回栈的页面切换，应使用 Router，而不是 Tabs。
