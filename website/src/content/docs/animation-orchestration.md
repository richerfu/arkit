---
title: Stagger 与 Animatable
description: "列表错开入场、高频改目标，以及把一组动画收进同一个 scope。"
---

# Stagger 与 Animatable

列表要错落入场时用 stagger；需要每帧改目标时用 Animatable；一组控件的生命周期想一起收掉，放进 AnimationScope。

## Stagger

```rust
let distribution = stagger(36)
    .from_center()
    .grid(StaggerGrid::new(4, 3))
    .axis(StaggerAxis::Radial)
    .jitter(0.15, 42);

let delay = distribution.delay_span(index, total);
```

支持 1D/2D/3D grid、X/Y/Z/radial axis、origin、reverse、easing、modifier 和 seeded jitter。jitter 使用稳定 seed，避免每次 render 改变 item 时序。

## 列表使用原则

delay 由稳定 item index/id 推导，Timeline 仍是唯一时钟。不要为每个 item 启动独立 Tokio timer；大列表只为进入视口或实际挂载的 item 创建动画。

## Animatable

```rust
let progress = use_animatable_with_defaults(
    0.0_f32,
    AnimatableDefaults {
        duration: TimeSpan::from_millis(240),
        easing: Easing::Builtin(BuiltinEase::Cubic(EaseDirection::Out)),
        ..Default::default()
    },
);

progress.to(1.0);
progress.retarget(0.4, TimeSpan::from_millis(120));
```

Animatable 适合 Drawing 数值、持续手势反馈和频繁目标变化。`retarget` 从当前采样值继续，不跳回旧起点，也不创建私有 timer。

## AnimationScope

`use_animation_scope`/`use_scoped_animation` 把 controls 和命名方法集中到一个 scope。父组件可按 `ScopeMethodName` 触发局部编排，而不直接持有每条 Timeline。

scope drop 按 `ScopeCleanupPolicy` cancel 或 revert。选择依据是卸载后视觉值是否仍可能被复用节点观察；临时 overlay 通常 revert，永久离场节点可以 cancel。

## 选择 API

| 需求                   | API              |
| ---------------------- | ---------------- |
| 固定时序、多属性编排   | `Timeline`       |
| 多 item 延迟分布       | `Stagger`        |
| 高频变更单个目标值     | `Animatable`     |
| 暴露命名动作并集中清理 | `AnimationScope` |
