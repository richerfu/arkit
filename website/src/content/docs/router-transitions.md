---
title: 页面转场
description: "页面切换动画怎么配，前进和返回方向如何区分。"
---

# 页面转场

切页时可以挂过渡动画。配置跟在路由上，前进和返回可以走不同方向，生命周期也和页面挂载对齐。

## RouteTransition

```rust
#[component]
fn Home() -> Element {
    rsx! {
        RouteTransition::<Route> {
            preset: TransitionPreset::SlideLeft,
            duration_ms: 220,
            delay_ms: 0,
            fill: true,
            column { "Home" }
        }
    }
}
```

默认 preset 是 `SlideLeft`、duration 220ms、delay 0、fill true。调用方需要让 route change 产生稳定而不同的 component/key identity。

## AnimatedOutlet

```rust
AnimatedOutlet::<Route> {
    preset: TransitionPreset::Fade,
    duration_ms: 180,
}
```

它按当前 typed route 的字符串 identity 为 wrapper 建 key，nested route 改变时重新 mount transition。公共 layout 本身不会随 child 一起重建。

## 前进与返回

简单应用可以统一 Fade/Slide。需要区分 push 与 back 时，在 router history 变化处记录导航方向，再选择对应 preset；方向只是转场输入，不另存当前路径。

## 快速切页

新 route 到来时旧 transition 必须按 AnimationHost 的取消语义清理。不要用固定 timeout 删除页面，也不要在 animation complete callback 再 push route，否则会把视觉生命周期和 history 相互锁死。

## 可访问性与性能

尊重系统减少动态效果设置：把 duration 降为 0 或选择轻量 Fade。避免整页同时做 blur、shadow 和大量 layout animation；先保证导航输入立即生效，再让视觉动画跟进。

## 验证

`examples/router` 覆盖动态参数、Link、系统 back 与 RouteTransition。真机检查快速连续导航、返回到根、组件卸载清理和旋转过程中的转场 frame。
