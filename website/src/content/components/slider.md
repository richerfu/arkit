---
title: Slider
description: "单值、范围和多 thumb 滑块。"
---

# Slider

Slider 家族用于连续或离散数值选择，包括单值 `Slider`、区间 `RangeSlider` 和多节点 `MultiSlider`。

```rust
let mut volume = use_signal(|| 40.0_f32);

Slider {
    value: volume(),
    min: 0.0,
    max: 100.0,
    step: 5.0,
    show_steps: true,
    on_change: move |next| volume.set(next),
}
```

三种组件共享以下配置：`min`、`max`、`step`、`orientation`、`reversed`、`disabled`、`show_steps`、`width`、`height` 和 `SliderStyle`。

| 组件          | 值类型     | 回调                     |
| ------------- | ---------- | ------------------------ |
| `Slider`      | `f32`      | `EventHandler<f32>`      |
| `RangeSlider` | `[f32; 2]` | `EventHandler<[f32; 2]>` |
| `MultiSlider` | `Vec<f32>` | `EventHandler<Vec<f32>>` |

区间端点应保持从小到大，多节点值应排序。界面同时展示当前数值或无障碍描述，不能只依赖 thumb 位置传达结果。
