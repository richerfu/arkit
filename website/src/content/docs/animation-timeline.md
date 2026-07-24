---
title: Timeline 编排
description: "用 label、call 和嵌套 timeline 把多段动画排进一条时间线。"
---

# Timeline 编排

Timeline 把多段动画、延时、回调和子时间线排成一条确定的时间计划。用 label 标记阶段，比满屏魔法数字好改。

## 位置表达式

节点可放在绝对时间、label offset、previous start/end、timeline end 或百分比位置。使用 label 表达业务阶段，比散落的毫秒常量更容易调整。

```rust
let timeline = Timeline::new()
    .label(LabelName::owned("intro"), TimelinePosition::START)
    .add(card, TimelinePosition::START)
    .add(
        badge,
        TimelinePosition::Label {
            label: LabelName::owned("intro"),
            offset: TimeOffset::from_millis(120),
        },
    )
    .call(
        || {},
        CallPolicy::ForwardOnly,
        Timeline::at(420).expect("valid constant time"),
    );
```

label 必须先定义再引用。重复定义和前向引用返回 typed resolve error。

## 同时刻顺序

同一时刻固定按 `set → call → barrier`；同类节点保持 source insertion order。依赖这一顺序的状态变更应写成显式节点，不通过多个异步 timer 猜执行次序。

## 循环与交替

```rust
let timeline = timeline
    .iterations(IterationCount::finite(2).expect("non-zero"))
    .alternate(true);
```

finite iteration 可以 `complete` 并产生 terminal outcome；infinite timeline 没有自然终点。alternate 反转每轮局部方向，不等于对 controls 调用 `reverse`。

## Call Policy

Call 节点可以限定正向、反向或 crossing 行为。普通 `seek` 只采样视觉值，不触发 crossing；需要模拟真实播放越过节点时使用 `seek_with_events`。

回调内发出的 controls 命令进入 FIFO queue，不重入当前 tick。这样 callback 可以安全 restart/cancel，但命令会在当前提交完成后处理。

## Nested Timeline

可复用的局部编排作为 nested timeline 放入父 timeline，再由父位置和 iteration 控制。子 timeline 不创建私有时钟；组合后仍由同一个 Engine 采样。

## 计划校验

在用户交互前构建并解析计划。处理 `AnimationBuildError`/`TimeError`，不要把非法 label、负时间或零 iteration 延迟到首帧。
