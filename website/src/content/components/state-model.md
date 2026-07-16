---
title: 状态模型
description: "受控、非受控、事件与表单状态。"
---

# 状态模型

交互组件采用受控/非受控两种状态模型。业务状态、路由状态和服务器状态优先受控；纯局部展开、临时选择可以非受控。

## 受控模式

```rust
let mut checked = use_signal(|| false);

Checkbox {
    checked: checked(),
    on_change: move |next| checked.set(next),
}
```

组件读取 `checked`，回调只报告 next value。调用方必须更新 prop，否则画面继续保持外部值。

## 非受控模式

```rust
Checkbox {
    default_checked: true,
    on_change: move |next| tracing::info!(next),
}
```

`default_*` 只初始化内部 Signal，不会在后续 prop 改变时重置。要从外部重置组件，应改为受控或更换稳定 key。

## 常见字段

| 状态 | 受控字段         | 初始字段                         | 回调                      |
| ---- | ---------------- | -------------------------------- | ------------------------- |
| 开关 | `checked`        | `default_checked`                | `on_change`               |
| 展开 | `open`           | `default_open`                   | `on_open_change/on_close` |
| 选择 | `selected/value` | `default_selected/default_value` | `on_select/on_change`     |

## 状态所有权

- Input/表单值由表单页面持有。
- Sidebar、BottomNavigation 的 active route 从 Router 派生。
- Menu 的 checkbox/radio 值由业务 Signal 持有。
- Sonner 的 toast Vec 和稳定 id 由调用方持有。
- 动画进度由 AnimationHost 持有，不要每帧复制到全局 Signal。

## 卸载与清理

组件内部 Signal 随 scope 卸载。Overlay token、timer、订阅和动画 handle 也必须在所属 scope 清理；调用方不要保存已卸载组件的 controller/overlay handle。
