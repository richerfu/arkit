# animation

路径：`examples/animation`

这个示例使用 `Timeline` 组合 fade、四向 slide、zoom in/out 和顺/逆时针 rotate，并同时演示颜色、圆角、尺寸插值以及列表 stagger。选择 preset 会自动播放对应动效；点击 Replay 会替换时间线并重新播放，无需重新挂载子树。

核心 API 是 frame-driven `Timeline`，支持任意时间点 keyframe、每段 easing（预设、cubic Bézier、steps、spring）、delay、iterations、alternate、reverse、playback rate，以及 play/pause/resume/restart/seek/finish/cancel/reset 控制：

```rust
let timeline = Timeline::new(
    AnimationState::new().opacity(0.0).translate(0.0, 40.0),
)
.to_with(
    AnimationState::new().uniform_scale(1.08),
    420,
    Easing::EaseOutBack,
)
.to_with(AnimationState::default(), 120, Easing::EaseInOutQuad)
.iterations(2)
.alternate(true);

let player = use_timeline(timeline);

// Dioxus event syntax
button {
    onclick: move |_| player.restart(),
    "Replay"
}
```

## 多目标时间线

`TimelineGroup` 使用一个 frame clock 同步驱动多个 Dioxus 组件。父组件创建 group，子组件只需要注册稳定 target id；播放、暂停、seek、reverse 和循环状态都由同一个 `TimelineGroupControls` 管理：

```rust
let card = Timeline::new(card_hidden).to(card_visible, 420);
let badge = Timeline::new(badge_hidden).to(badge_visible, 280);

let group = TimelineGroup::new()
    .label_at("intro", 0)
    .add_at("card", card, 0)
    .add_at_label("badge", badge, "intro", 120)?
    .iterations(2)
    .loop_delay_ms(300)
    .alternate(true);

let player = use_timeline_group(group);

// 后代组件的根 ArkUI 节点成为该 target：
let _target = use_animation_target("card");
```

可使用 `add` 顺序追加、`add_at` 绝对定位、`add_with_previous` 从上一段开头偏移、`add_after_previous` 从上一段结尾偏移，以及 `label` / `add_at_label` 命名定位。同一 target 可以添加多段 track，后开始的 track 接管该 target。

相对关键帧使用 `AnimationDelta`。位移、旋转和尺寸为加法，scale 为乘法：

```rust
let timeline = Timeline::new(initial).to_relative(
    AnimationDelta::new()
        .translate_by(24.0, 0.0)
        .rotate_by(90.0)
        .uniform_scale_by(1.2),
    300,
    Easing::EaseOutCubic,
);
```

`AnimationState` 当前支持 opacity、translate、scale、rotate、background/font color、border radius、blur、width 和 height。颜色使用 ArkUI 的 `0xAARRGGBB` 格式；可选属性只会在显式设置时写入节点：

```rust
let timeline = Timeline::new(
    AnimationState::new()
        .background_color(0xff0f766e)
        .border_radius(8.0)
        .size(120.0, 80.0),
)
.to_with(
    AnimationState::new()
        .background_color(0xff7c3aed)
        .border_radius(24.0)
        .size(180.0, 120.0),
    400,
    Easing::CubicBezier { x1: 0.2, y1: 0.8, x2: 0.2, y2: 1.0 },
);
```

在 Dioxus 列表中，`stagger()` 分配 target 的 group position，因此不依赖 DOM selector：

```rust
for index in 0..items.len() {
    let delay = stagger(45).from_center().delay(index, items.len());
    group = group.add_at(format!("item-{index}"), item_timeline.clone(), delay);
    // 子组件：let _target = use_animation_target(format!("item-{index}"));
}
```

播放器还提供 `current_time_ms()`、`progress()`、`status()`，以及 `on_begin`、`on_update`、`on_loop`、`on_pause`、`on_complete` 生命周期回调。

单段 ArkUI 原生动效仍可用 `AnimationState` 配合 `AnimationControls`：

```rust
let controls = use_animation(Motion::new().duration_ms(300));
use_effect(move || {
    if controls.is_ready() {
        controls.set(AnimationState::new().opacity(0.0).translate(0.0, 24.0));
        controls.animate_to_next_frame(AnimationState::default());
    }
});
```

目标平台构建：

```sh
cd examples/animation
ohrs build --arch aarch
```
