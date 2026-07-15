---
title: 属性与关键帧
---

# 属性与关键帧

动画属性按值域类型化。错误值类型、未知 property、单位不兼容和 baseline 读取失败在 build/resolve 阶段报告，不会在每帧退化为字符串转换。

## 内建属性

| 值类型       | 属性                                                                                                                         |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| `f32`        | `OPACITY`、`SCALE_X/Y`、`BRIGHTNESS`、`SATURATION`、`GRAYSCALE`、`INVERT`、`SEPIA`、`CONTRAST`、`ASPECT_RATIO`               |
| `Length`     | `TRANSLATE_X/Y`、`WIDTH/HEIGHT`、`POSITION_X/Y`、`BORDER_WIDTH/RADIUS`、`BLUR`、`FONT_SIZE`、`LINE_HEIGHT`、`LETTER_SPACING` |
| `Angle`      | `ROTATION`                                                                                                                   |
| `LinearRgba` | `BACKGROUND_COLOR`、`FOREGROUND_COLOR`、`FONT_COLOR`、`BORDER_COLOR`                                                         |

`Length` 保留单位，`LinearRgba` 在线性颜色空间插值，避免普通 sRGB 通道插值产生不自然的中间色。

## Tween

```rust
let fade = Animation::new(selector)
    .tween(
        &OPACITY,
        0.0,
        1.0,
        TimeSpan::from_millis(320),
    )
    .configure_last(
        Easing::Builtin(BuiltinEase::Cubic(EaseDirection::Out)),
        Composition::Replace,
        Modifier::Identity,
        TimeSpan::ZERO,
        0,
    );
```

`configure_last` 设置最后一个 track 的 easing、composition、modifier、delay 和 priority。

## Keyframes

```rust
let pulse = Animation::new(selector).keyframes(
    &SCALE_X,
    [
        PropertyKeyframe::new(0.0, 1.0),
        PropertyKeyframe::new(0.6, 1.15)
            .easing(Easing::Builtin(BuiltinEase::Sine(EaseDirection::Out))),
        PropertyKeyframe::new(1.0, 1.0),
    ],
    TimeSpan::from_millis(600),
)?;
```

offset 在归一化范围内严格递增。每个 segment 可以有独立 easing。

## Easing

支持 linear、内建 ease、irregular ease、spring 与合法 custom easing。Spring 描述物理响应；duration-based easing 描述固定时间映射。需要精确 terminal 时优先有限 duration，避免以视觉阈值猜测结束。

## Composition

| 模式         | 语义                                                        |
| ------------ | ----------------------------------------------------------- |
| `Replace`    | 按 timeline position、priority、insertion order 选择 winner |
| `Add`        | 在 baseline 上叠加 active contribution                      |
| `Accumulate` | 把已完成 iteration 的 terminal delta 累积到下一轮           |

对同一 target/property 混合多个动画前明确 composition。依赖隐式覆盖顺序会让插入新 track 后产生不可见的优先级变化。
