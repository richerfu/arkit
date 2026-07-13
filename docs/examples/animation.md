# animation

路径：`examples/animation`

这是统一 Animation v2 的完整交互式能力展厅，不是单一过渡效果演示。应用按能力域拆成五个可切换页面，每个页面都直接调用公开 API，并提供 replay、切换、暂停、seek、reverse、revert 等可观察操作。

| 页面 | 可操作能力 |
| --- | --- |
| Timeline | 多目标 Timeline、label/相对位置、nested、timer、set/call、keyframes、iterations/alternate/loop delay、完整 controls、snapshot、callback 和 lowering report |
| Easing | builtin、spring、steps、cubic Bézier、linear points、seeded irregular，以及 grid/radial/axis/reverse/jitter stagger |
| Lifecycle | 9 种 `TransitionPreset`、`AnimatePresence` 的 Sync/Wait/PopLayout、enter/exit 协调、FLIP layout projection 和 layout registry |
| Input | 真 touch draggable、constraints、velocity/inertia、grid snap、auto-scroll、scroll-linked seek、`Animatable<f32>` drawing value |
| Scope | `AnimationScope`、共享 playback defaults、named methods、cleanup、composition/modifier、完整 typed property schema、capability lowering 和 window conditions |

## 代码结构

```text
examples/animation/src/
├── lib.rs                 # 展厅 shell、分页和共享 UI
├── timeline_lab.rs        # Timeline / controls / callbacks
├── easing_lab.rs          # Easing / stagger
├── lifecycle_lab.rs       # transition / presence / layout
├── interaction_lab.rs     # drag / scroll / animatable
└── orchestration_lab.rs   # scope / properties / capabilities
```

`#[entry]` 自动安装唯一的 root `AnimationHost`。所有示例通过稳定 target、typed `Property<T>`、同一 root clock 和 batch commit 工作；组件没有私有 frame loop。只有显式使用 `use_animation_snapshot` 的状态读数才订阅逐帧响应式更新。

例如，多目标动画仍从公开 builder 组合：

```rust
let _target = use_animation_target("card");
let card = Animation::new(AnimationSelector::Target(TargetName::owned("card")))
    .tween(&OPACITY, 0.0, 1.0, TimeSpan::from_millis(600))
    .keyframes(
        &SCALE_X,
        [
            PropertyKeyframe::new(0.0, 0.7),
            PropertyKeyframe::new(0.6, 1.15),
            PropertyKeyframe::new(1.0, 1.0),
        ],
        TimeSpan::from_millis(600),
    )?;

let controls = use_animation(
    Timeline::new()
        .label(LabelName::owned("intro"), TimelinePosition::START)
        .add(card, TimelinePosition::START)
        .iterations(IterationCount::finite(2)?)
        .alternate(true),
);
```

完整 API 和 v1 升级说明见 [Animation v2](../guide/animation.md)，精确 controls/callback 语义见 [Animation v2 可执行语义](../guide/animation-semantics.md)。

## 构建与运行

```sh
cargo check --manifest-path examples/animation/Cargo.toml
cargo clippy --manifest-path examples/animation/Cargo.toml -- -D warnings

cd examples/animation
ohrs build --arch aarch
```

也可以从仓库根目录同步进统一示例应用：

```sh
./app/run.sh animation sync
./app/run.sh animation build
```
