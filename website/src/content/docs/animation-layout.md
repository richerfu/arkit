---
title: Layout 与 Presence
---

# Layout 与 Presence

Layout animation 比较前后布局快照，把几何变化转换为 FLIP timeline。Presence 保留离场 child，直到真实动画 terminal event。

## 注册 Layout

`use_animation_layout` 注册稳定 layout id；`use_layout_snapshot` 收集 parent topology、frame、visibility、z-order、window metrics 和 generation。

`LayoutEngine` 比较快照，产生：

- `Enter` / `Exit`
- `Move` / `Resize`
- `Reparent`
- `Visibility`

随后可生成 FLIP Timeline：先用 transform 抵消新布局，再动画到 identity。最终布局始终由 ArkUI 决定，动画不保存第二份永久 frame。

## 稳定身份

layout id 表达业务元素身份，跨重排保持稳定。使用列表 index 会在插入项时把旧节点错误匹配到新数据。

## AnimatePresence

```rust
let presence = use_animate_presence::<Item>(PresenceMode::Wait);
```

| Mode        | 语义                             |
| ----------- | -------------------------------- |
| `Sync`      | 进入与退出同时进行               |
| `Wait`      | 退出完成后再进入                 |
| `PopLayout` | 退出项脱离布局，其余内容立即重排 |

leaving child 保留到调用 `settle_exit`。没有固定 timeout，因此 spring、reverse、pause 或 backend fallback 都不会提前卸载。

## 取消退出

同 key 在退出期间重新出现时，按 `ExitCancelPolicy` 选择 re-enter 或完成退出。业务应保持 key 和领域身份一致，避免把两个不同对象误认为取消退出。

## Shared Element

`SharedElementProjection` 可表达跨 parent 的投影。前后页面必须在同一 AnimationHost 和可比较 window metrics 下；窗口 resize 期间可 `refresh` 重新解析，不能沿用旧物理 frame。

## 性能

快照只注册真正需要 layout animation 的节点。大列表不要全量采集；结合虚拟范围，只动画已挂载的可见 item。
