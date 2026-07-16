---
title: 动画概览
description: "统一时钟、target、timeline 与渲染路径。"
---

# 动画概览

启用 `animation` 后，一个 Arkit UI root 只有一个 AnimationHost、一个 Engine 和一次帧提交。ArkUI node、Drawing、Chart、Layout、Presence、Drag 与 Scroll 共用时钟和控制语义。

## 核心模型

```text
Target + Property + Value
          ↓
Animation / Keyframes
          ↓
Timeline
          ↓
AnimationControls
          ↓
sample → compose → adapter commit
```

`#[entry]` 自动安装 host，业务组件不要重复 provider。

## 最小动画

```rust
let _target = use_animation_target("card");
let fade = Animation::new(
    AnimationSelector::Target(TargetName::owned("card")),
)
.tween(&OPACITY, 0.0, 1.0, TimeSpan::from_millis(240));

let controls = use_animation(
    Timeline::new().add(fade, TimelinePosition::START),
);

rsx! {
    column {
        button { onclick: move |_| controls.restart(), "播放" }
    }
}
```

target 尚未挂载时实例进入 Scheduled，节点可用后开始；同一 resolve scope 中重复 target 名会返回错误。

## 为什么使用 Timeline

Animation 描述某个 target/property 的变化，Timeline 描述多个变化、timer、call 和 barrier 的相对时间。Controls 只操作已解析计划，所以 play、pause、seek、reverse 和 cancel 在不同 adapter 上有一致语义。

## 章节地图

- 属性与关键帧：值类型、easing、composition。
- Timeline 编排：位置、label、回调和循环。
- 播放控制：命令、快照、finished。
- Stagger 与 Animatable：批量编排和 imperative retarget。
- Layout 与 Presence：FLIP、进退场。
- Drag 与 Scroll：手势驱动 timeline。
- 后端与性能：sampled/native lowering 和诊断。
