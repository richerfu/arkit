---
title: Drag 与 Scroll
---

# Drag 与 Scroll

手势模块不建立另一套动画系统。Drag 把 pointer sample 映射为 controls seek，release 的惯性、snap、spring 编译为 Timeline；Scroll observer 也在帧边界驱动同一 Engine。

## Draggable

```rust
let drag = use_draggable(
    controls.clone(),
    TargetName::owned("sheet"),
    DraggableConfig {
        axis: DragAxis::Vertical,
        snap: DragSnap::Grid(Vec2::new(1.0, 120.0)),
        ..Default::default()
    },
    DraggableCallbacks::default(),
);
```

配置包括 axis、constraints、mapping、snap、inertia、auto-scroll 和 callback。pointer down/move/up/cancel 必须完整交给 handle，避免丢失终止阶段。

## 约束与速度

`DragConstraints` 限制有效位置；`VelocityTracker` 用最近 samples 估计 release velocity。坐标统一到 target/window 空间后再约束，不混用物理 px 和 vp。

Snap 可以是 grid 或候选点。惯性预测落点，再选择 snap，最后以 spring/tween 收敛；所有阶段仍可 pause/cancel/revert。

## Scroll Observer

`use_scroll_observer` 支持：

- horizontal / vertical axis
- typed threshold
- enter / leave
- forward / backward
- repeat / once
- method / progress / eased / smooth sync

一帧内多次 `update_at` 后，`flush_frame` 只消费最后 sample，降低 native scroll 高频回调对 Engine 的压力。

## Scroll 联动

进入视口只触发一次用 once threshold；随滚动连续变化用 progress sync；需要阻尼用 eased/smooth。不要在每个 scroll event 写 Dioxus Signal 来驱动 60fps 属性，除非 UI 真的需要重渲染文本状态。

## 手势冲突

明确 drag axis、hit-test 区域和滚动容器方向。嵌套纵向 drag 与纵向 scroll 时由最内层可操作区域决定是否接受手势，并在 cancel 阶段恢复 controls 状态。

## 验证

测试慢拖、快速甩动、越界、取消、旋转后重算约束、嵌套滚动和多指输入。帧率问题先检查是否误把 pointer progress 接入了全树 Signal。
