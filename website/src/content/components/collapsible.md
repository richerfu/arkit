---
title: Collapsible
---

# Collapsible

Collapsible 控制单个区域的展开与收起，比 Accordion 更适合独立的高级选项区。

```rust
Collapsible {
    title: "高级设置",
    default_open: false,
    on_open_change: move |open| tracing::debug!(open),
    AdvancedSettings {}
}
```

| 属性             | 类型                 | 默认值     | 说明           |
| ---------------- | -------------------- | ---------- | -------------- |
| `title`          | `String`             | 必填       | 触发区域标题   |
| `children`       | `Element`            | 必填       | 展开内容       |
| `open`           | `Option<bool>`       | `None`     | 受控展开状态   |
| `default_open`   | `bool`               | `false`    | 非受控初始状态 |
| `on_open_change` | `EventHandler<bool>` | 默认空处理 | 状态变化回调   |

传入 `open` 后必须在回调中更新外部状态。Collapsible 只控制可见性，不负责持久化用户偏好。
