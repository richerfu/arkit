# Animation v2

Arkit 的动画系统由 root-owned `AnimationHost` 驱动。一个 UI root 只有一个时钟、一个 `AnimationEngine` 和一次帧提交；ArkUI 节点、Drawing/Canvas、Chart、Layout、Presence、Drag 与 Scroll 都使用同一套时间和 controls 语义。

应用通过 `#[entry]` 挂载时会自动安装 host。业务组件不要自行安装 provider。

## 基础动画

先给组件根节点注册稳定的 target 名，再用类型化属性构造 `Animation`，最后把动画加入多目标 `Timeline`：

```rust
use arkit::prelude::*;

#[component]
fn Card() -> Element {
    let _target = use_animation_target("card");
    let target = TargetName::owned("card");
    let animation = Animation::new(AnimationSelector::Target(target))
        .tween(&OPACITY, 0.0, 1.0, TimeSpan::from_millis(320))
        .configure_last(
            Easing::Builtin(BuiltinEase::Cubic(EaseDirection::Out)),
            Composition::Replace,
            Modifier::Identity,
            TimeSpan::ZERO,
            0,
        )
        .tween(
            &TRANSLATE_Y,
            Length::vp(24.0),
            Length::vp(0.0),
            TimeSpan::from_millis(320),
        );
    let controls = use_animation(
        Timeline::new().add(animation, TimelinePosition::START),
    );

    rsx! {
        column {
            button { onclick: move |_| controls.restart(), "Replay" }
        }
    }
}
```

属性类型在编译期固定值域。例如 `OPACITY` 接受 `f32`，`TRANSLATE_X/Y` 接受 `Length`，`ROTATION` 接受 `Angle`，颜色属性接受 `LinearRgba`。不同单位的 `Length` 不会被静默混合。

内建 ArkUI schema 包含 transform、opacity、position/size、颜色、字体、边框、圆角、blur 与 brightness/contrast/saturation 等视觉属性。未知属性、类型错误和 baseline 读取失败都会返回 typed error。

## Timeline 与 controls

`Timeline` 可以同时包含 animation、timer、set、call、label、barrier 和 nested timeline。位置可用绝对时间、label 偏移、上一段起点/终点、timeline 末尾或百分比表达。内部在 resolve/compile 阶段把名字转换为 dense ID；每帧不做 target/property 字符串查找。

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
        Timeline::at(420).expect("constant time is representable"),
    )
    .iterations(IterationCount::finite(2).expect("non-zero"))
    .alternate(true);
```

`AnimationControls` 提供 `play`、`pause`、`resume`、`restart`、`reverse`、`seek`、`seek_with_events`、`complete`、`cancel`、`reset`、`revert`、`stretch`、`refresh` 和运行时替换 timeline。`reset` 回到计划起点；`revert` 恢复开始动画前捕获的 baseline；`cancel` 保留最后已提交画面。详细 crossing 和 terminal 顺序见 [Animation v2 可执行语义](./animation-semantics.md)。

只有显式调用 `use_animation_snapshot` 或 `subscribe` 时才会产生响应式进度更新。普通动画帧不会驱动 Dioxus component 重渲染。

```rust
controls.on_begin(|| {});
controls.on_before_update(|logical_time| {});
controls.on_update(|progress| {});
controls.on_render(|| {}); // adapter 已成功提交这一帧之后
controls.on_loop(|completed_iterations| {});
controls.on_complete(|| {});

let outcome = controls.finished().await;
```

## Keyframes、composition 与 stagger

属性级 keyframe 使用严格递增的归一化 offset：

```rust
let pulse = Animation::new(AnimationSelector::Target(TargetName::owned("pulse")))
    .keyframes(
        &SCALE_X,
        [
            PropertyKeyframe::new(0.0, 1.0),
            PropertyKeyframe::new(0.6, 1.15),
            PropertyKeyframe::new(1.0, 1.0),
        ],
        TimeSpan::from_millis(600),
    )?;
```

同一属性的 animation 可以使用 `Replace`、`Add` 或 `Accumulate`。优先级在 compile 阶段稳定排序，baseline 只在 resolve/refresh 边界读取。

`Stagger` 支持 1D、2D、3D grid、X/Y/Z/radial axis、origin、reverse、easing、modifier 和带 seed 的 jitter：

```rust
let distribution = stagger(36)
    .from_center()
    .grid(StaggerGrid::new(4, 3))
    .axis(StaggerAxis::Radial)
    .jitter(0.15, 42);

let delay = distribution.delay_span(index, total);
let value = distribution.value(0.8, 1.0, index, total);
```

## Animatable 与 Scope

`Animatable<T>` 适合频繁 imperative retarget 或 Drawing/Canvas 数值。它使用 Drawing adapter 接入 root Engine，不创建 timer：

```rust
let progress = use_animatable_with_defaults(
    0.0_f32,
    AnimatableDefaults {
        duration: TimeSpan::from_millis(240),
        easing: Easing::Builtin(BuiltinEase::Cubic(EaseDirection::Out)),
        ..AnimatableDefaults::default()
    },
);
progress.to(1.0);
progress.retarget(0.4, TimeSpan::from_millis(120));
progress.set_invalidator(|| { /* request canvas redraw */ });
```

`AnimationScope` 统一拥有 controls、命名方法和外部事件清理。scope drop 时先注销事件源，再按策略 `revert` 或 `cancel` 所有实例。`WindowCondition` 使用 animation window metrics 匹配响应式条件。

## Layout 与 Presence

`use_animation_layout` 把节点注册进 root layout registry。`use_layout_snapshot` 生成带 parent topology、frame、visibility、z-order、window metrics 和 generation 的集中快照；`LayoutEngine` 比较前后快照并输出 Enter、Exit、Move、Resize、Reparent 或 Visibility delta。`LayoutDelta::timeline` 把 FLIP inverse transform 编译回同一个 Engine。

```rust
use_animation_layout(
    LayoutId::owned("card"),
    Some(LayoutId::owned("grid")),
    true,
    0,
);

layout_engine.record_old(before);
for delta in layout_engine.record_new(after) {
    if let Some(timeline) = delta.timeline(
        TargetName::owned(delta.id.as_str()),
        LayoutAnimationMode::PositionAndSize,
        TimeSpan::from_millis(280),
        Easing::Linear,
    ) {
        // Feed the generated timeline to use_animation/use_scoped_animation.
    }
}
```

`AnimatePresence<T>` 提供 `Sync`、`Wait` 和 `PopLayout`。leaving child 会一直保留到真实 animation terminal event 调用 `settle_exit`；没有固定 timeout。取消退出时可选择 re-enter 或完成退出。`SharedElementProjection` 在 settle/drop 时释放临时 projection cleanup。

## Draggable

`Draggable` 不拥有私有时钟。pointer 采样生成确定性 velocity，拖动位置映射为 linked animation seek；release 的 inertia、snap 和 spring 被编译成新的 Timeline 交给相同 Engine。

```rust
let drag = use_draggable(
    controls.clone(),
    TargetName::owned("sheet"),
    DraggableConfig {
        axis: DragAxis::Vertical,
        constraints: Some(DragConstraints {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(0.0, 480.0),
            padding: Vec2::default(),
        }),
        snap: DragSnap::Grid(Vec2::new(1.0, 120.0)),
        ..DraggableConfig::default()
    },
    DraggableCallbacks::default(),
);

drag.grab(event_time, pointer);
drag.drag(event_time, pointer);
drag.release();
```

`DragUpdate::auto_scroll_velocity` 是容器需要消费的 auto-scroll 指令；框架不会假定具体滚动容器实现。

## Scroll

`ScrollObserver` 支持 horizontal/vertical、typed threshold、enter/leave、forward/backward、repeat/once，以及 method/progress/eased/smooth sync。平台可以在一帧内多次调用 `update_at`，frame boundary 只通过 `flush_frame` 消费最后一个事件。

```rust
let observer = use_scroll_observer(
    controls.clone(),
    ScrollRange { start: 120.0, end: 720.0 },
    TimeSpan::from_millis(900),
    ScrollSync::Smooth { factor: 0.2, easing: Easing::Linear },
    ScrollCallbacks::default(),
);

observer.update_at(frame_time, content_offset);
let sample = observer.flush_frame();
```

`refresh` 在布局或 viewport 改变后替换 range 并触发 Engine resolution refresh；`revert` 清空 observer 状态并恢复 animation baseline。

## Backend lowering 与诊断

`ExecutionPolicy` 有四种：

| Policy | 行为 |
| --- | --- |
| `Auto` | 在语义可保持时选择 native，否则 sampled |
| `SampledOnly` | 强制 root Engine sampled backend |
| `NativePreferred` | 优先 native，所有 fallback 写入 report |
| `NativeOnly` | 任一能力或运行环境不满足时返回 typed error |

lowerer 会检查 seek、pause/resume、reverse、cancel、alternate、callbacks、per-property timing、composition、dynamic modifier、infinite iteration、layout invalidation、custom easing 和 property support。`controls.lowering_report()` 返回选中 backend、每个 native backend 的拒绝原因、target/property/tween 数和估算的每帧工作量。

当前 `#[entry]` root 没有来自 ETS 的 `ArkUIContext` 注入通道，因此普通 hook 路径会执行语义等价的 sampled backend，并在 `LoweringReport` 中记录 `BackendUnavailable`；`NativeOnly` 会报错，不会静默降级。`ArkUiImplicitInstance`、`ArkUiKeyframeInstance` 和 `ArkUiAnimatorInstance` 是真实 ArkUI handle 的所有权封装，供持有 `ArkUIContext` 的平台集成层使用。

debug build 可通过 `AnimationHost::performance_counters()` 读取 frame、dirty write、adapter failure、target miss、fallback、compute/apply 时间与 Engine 内部计数。计数器属于诊断接口，不是单独的 benchmark 页面。

## v1 升级

v2 是一次性 breaking cutover，没有 compatibility wrapper：

| v1 | v2 |
| --- | --- |
| `AnimationState` 固定字段 | `Animation` + `Property<T>` |
| `AnimationDelta` | typed from/to + `Composition`/`Modifier` |
| `TimelineGroup` | 一个原生多目标 `Timeline` |
| `use_timeline_group` / player | `use_animation` + `AnimationControls` |
| 每 player 的 frame clock | root-owned `AnimationHost` |
| Chart 私有 async clock | Engine-driven `Animatable` |
| fixed retention / timeout | terminal event + owner drop |

迁移时必须先为节点注册稳定 `TargetName`，把 state 字段替换为对应属性常量，把毫秒整数替换为 `TimeSpan`/`TimePoint`，再把 group 的 track 位置映射到 `TimelinePosition`。不要在业务层重建 v1 wrapper。

## 目标平台验证

```sh
cd examples/animation
ohrs build --arch aarch
```

host `cargo check` 只能验证 Rust 类型和平台无关逻辑，不能替代 OpenHarmony 构建与真机画面验收。
