# Arkit Animation 终态架构与实施计划

> 状态：终态能力实施完成；量化性能门槛保留为发版验收项  
> 日期：2026-07-13  
> 适用范围：`arkit_animation`、动画核心、ArkUI/Dioxus 集成、Layout、Presence、Gesture、Scroll、Drawing adapter  
> 基准：以 Anime.js v4.5 的能力模型为参照，但只实现 ArkUI/OpenHarmony 有意义的终态能力，不复制 DOM/CSS 专属接口。

当前进度（2026-07-13）：

- Phase 0–3：完成。语义冻结、纯 Rust core、Source/Resolved/Compiled、Resolver/Compiler、nested TimeDomain、单一 Engine、完整 controls/callback/crossing、composition、两阶段 frame commit、refresh 原子替换和 deterministic trace 均已落地。
- Phase 4–6：完成。root host/frame driver、ArkUI/Drawing adapter contract、property schema、dirty batch、target lifecycle、capability lowering/report、真实 native instance owner、最终 builders、Animatable、Scope、Stagger、MountTransition 和 facade cutover 均已落地。
- Phase 7–8：完成。集中 LayoutSnapshot/FLIP、Presence Sync/Wait/PopLayout、shared projection owner、Draggable velocity/inertia/spring/snap/auto-scroll command、Scroll threshold/direction/coalescing/sync 全部接入同一 Engine，不存在外围 frame loop 或 exit timeout。
- Phase 9：完成。Chart transition、timeline autoplay 和 effect phase 已迁移到 Engine-driven Animatable；workspace 调用方已切换，v1 文件和 public symbol 已删除，没有 compatibility wrapper。
- Phase 10：功能验收完成。57 个 core test、workspace check、严格 Clippy、fmt、metadata、文档构建、三个 aarch64 OHOS build、HAP 安装和真机视觉/交互验收均通过，详见 [Animation v2 验证记录](./animation-v2-verification.md)。按明确约束没有新增独立 benchmark feature/page；第 20 节参考设备 p95 与长稳时长仍作为发版运营门槛，不伪造数据，也不阻塞能力对齐结论。

## 1. 本文是终态合同，不是 MVP 路线图

本文定义最终必须保留的架构、类型、依赖方向、运行时语义和验收门槛。实施阶段只用于切分可评审交付，不允许通过简化数据模型换取短期进度。

以下做法被明确禁止：

- 禁止先实现固定字段版 `AnimationStateV2`，再计划以后改成 property/value 系统。
- 禁止先复制 `TimelinePlayer`、`TimelineGroupPlayer`、`DraggablePlayer` 等多套时钟，再计划以后合并。
- 禁止把 target、property、label 的 `String` 身份带入每帧热路径。
- 禁止先以 `HashMap<String, f32>` 表达属性，再计划以后补类型。
- 禁止让 ArkUI native path 和 sampled path 各自定义播放语义。
- 禁止把 Layout、Presence、Gesture、Scroll 做成不经过统一 Engine 的外围计时器。
- 禁止用长期 compatibility wrapper 同时保留 v1/v2 两套公开 API。
- 禁止以 host `cargo check` 代替 OpenHarmony 目标构建和设备验收。

实施允许在切换前保留当前 v1 供生产调用方继续编译，但 v2 的内部模型从第一次提交开始就必须是本文定义的终态模型；完成 workspace 迁移后必须在同一交付中删除 v1。

## 2. 结论与实施前硬阻塞

以下内容记录实施开始时的 v1 基线。对应阻塞均已通过本文定义的「可解析动画图 + 编译计划 + 单一引擎 + 多后端适配器」边界消除，用于解释本次 breaking cutover 的原因。

实施前必须消除的硬阻塞：

1. `arkit_animation` 同时依赖 Dioxus、ArkUI 和纯时间算法，导致纯算法测试也需要链接 OHOS native 库。
2. `Timeline` 与 `TimelineGroup` 拥有重复状态机，行为会持续漂移。
3. `AnimationState` 固定列出属性，任何新属性都要求修改公共结构、插值器和 writer。
4. `TimelineGroup::sample_targets` 每帧重新分组、排序、构造 `Vec` 和 target `String`。
5. 每个 timeline 单独申请 frame callback；框架没有 root-owned Engine。
6. 当前 frame callback 忽略平台提供的 frame timestamp，改用各 Player 自己的 `Instant`。
7. `progress` Signal 每帧无条件写入，动画计算与 Dioxus 重渲染耦合。
8. ArkUI 已提供 `Animation`、`KeyframeAnimation`、`Animator`，当前没有 capability-based lowering。
9. 当前 fixed retention 不能正确表达 infinite native animation 的所有权和销毁。
10. reset、revert、cancel、seek、reverse、loop、callback crossing 没有冻结的跨后端语义。

这些问题不是增量补字段可以解决的，必须按本文的最终边界重构。

## 3. 终态能力范围

### 3.1 必须完成的能力域

| 能力域 | 终态要求 |
| --- | --- |
| Timer | delay、duration、iterations、loop delay、alternate、reversed、playback rate、frame rate、playback easing |
| Tween | 每属性独立 from/to/delay/duration/easing/composition/modifier |
| Value | scalar、length、angle、color、vec2、vec3、transform、shadow、discrete、adapter custom value |
| Keyframes | sparse、duration-based、percentage-based、property-local、timeline-global |
| Timeline | animation、timer、call、set、label、remove、nested timeline、absolute/relative position |
| Controls | play、pause、resume、restart、reverse、alternate、seek、complete、cancel、reset、revert、stretch、refresh |
| Callbacks | begin、before update、update、render、loop、pause、complete、cancel、async finished |
| Composition | replace、add、accumulate；确定性优先级与 baseline snapshot |
| Easing | 完整 builtin families、cubic Bézier、linear points、steps、irregular、spring、custom pure easing |
| Stagger | time/value、range、unit、origin、direction、1D/2D/3D grid、axis、ease、modifier、jitter、seed |
| Animatable | 高频 imperative getter/setter、retarget、per-property defaults，不重建 Timeline |
| Scope | 生命周期、defaults、responsive/window conditions、named methods、refresh、revert、keep-time |
| Target adapter | ArkUI node、Drawing/Canvas object、用户自定义 adapter；统一 property schema 和 batch writer |
| Native lowering | implicit animation、native keyframe、Animator、sampled engine 的 capability-based 选择 |
| Layout | FLIP、reorder、enter、exit、swap parent、shared element、visibility/display 等价语义 |
| Gesture | drag、axis、bounds、snap、velocity、inertia、spring release、map-to-animation、auto scroll |
| Scroll | enter/leave threshold、direction、progress sync、smooth/eased sync、in-view、refresh/revert |
| Diagnostics | typed build/runtime error、lowering report、missing target/property、fallback 原因、性能计数器 |

### 3.2 明确不实现的浏览器专属能力

- CSS selector、DOM query、DOM attribute、CSS variable 和 inline style cleanup。
- 浏览器 WAAPI 对象兼容层。
- SVG path morph/draw/motion-path 的 DOM 版本。
- HTML text split/scramble 的 DOM 节点版本。
- Three.js adapter。

对应能力如果对 ArkUI 有价值，必须通过 Drawing/Canvas adapter、Text layout adapter 或 ArkUI component adapter 实现，不能把 DOM 概念带入 core。

## 4. 最终 crate 边界

### 4.1 `arkit_animation_core`

新增纯 Rust domain crate：

```text
crates/arkit_animation_core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── id.rs
    ├── time.rs
    ├── value.rs
    ├── property.rs
    ├── easing.rs
    ├── modifier.rs
    ├── composition.rs
    ├── source.rs
    ├── resolved.rs
    ├── tween.rs
    ├── timeline.rs
    ├── stagger.rs
    ├── compiler.rs
    ├── plan.rs
    ├── player.rs
    ├── engine.rs
    ├── command.rs
    ├── event.rs
    ├── frame.rs
    └── error.rs
```

该 crate 必须拥有：

- 所有时间、值、插值、easing、composition 和状态机语义。
- `AnimationCompiler` 与不可变 `CompiledAnimation`。
- 单一 `AnimationEngine`，所有动画实例、timer、call、layout motion、gesture release 都由它推进。
- 平台无关的 `EngineCommand`、`EngineEvent`、`FrameBatch`。
- `ManualClock` 和 deterministic trace 测试能力。

该 crate 禁止依赖：

- `arkit_prelude`
- `arkit_hooks`
- `arkit_arkui`
- `ohos-arkui-binding`
- `napi-ohos`
- Dioxus crate
- hilog

第三方依赖必须在根 `[workspace.dependencies]` 集中声明。密集 ID 必须使用 `oxc_index::IndexVec`；编译期 lookup 使用 `rustc_hash::FxHashMap`/`FxHashSet`；小型固定更新集合可以使用根依赖统一管理的 `smallvec`。

### 4.2 `arkit_animation`

`arkit_animation` 保留为最终公开的 ArkUI/Dioxus animation domain adapter：

```text
crates/arkit_animation/src/
├── lib.rs
├── api.rs
├── properties.rs
├── selector.rs
├── resolver.rs
├── target.rs
├── target_store.rs
├── property_schema.rs
├── property_reader.rs
├── property_writer.rs
├── adapter.rs
├── adapter_registry.rs
├── frame_driver.rs
├── host.rs
├── controls.rs
├── callbacks.rs
├── hooks.rs
├── scope.rs
├── animatable.rs
├── native_capability.rs
├── native_lowerer.rs
├── native_instance.rs
├── transition.rs
├── presence.rs
├── layout.rs
├── draggable.rs
├── scroll.rs
├── drawing_adapter.rs
└── diagnostic.rs
```

该 crate 必须拥有：

- 用户面对的 type-safe builder、target selector 和 property 常量。
- Dioxus hook、Scope、Controls、回调注册和 async completion。
- ArkUI target/property schema、snapshot reader、dirty batch writer。
- root-owned `AnimationHost` 与唯一 ArkUI `FrameDriver`。
- native capability 检查和 lowering，不重新实现 core 时间语义。
- Layout、Presence、Draggable、Scroll 和 Drawing adapter。

### 4.3 其他 crate 的职责

- `arkit` facade 必须在 `arkit_entry_root` 自动安装 `use_animation_host_provider()`，并只窄化 re-export 最终公共 API。
- `arkit_hooks` 继续拥有 node、layout、overlay、window metrics 等通用 Dioxus/ArkUI bridge；它禁止依赖 animation，以避免环。
- `arkit_shadcn` 只消费 `arkit_animation`，禁止内建组件私有动画时钟。
- `arkit_chart` 如果需要动画，必须通过 Drawing adapter 或 Engine-driven animatable 接入，禁止保留第二套 chart clock。

### 4.4 最终依赖方向

```text
arkit_animation_core
        ↑
arkit_animation ← arkit_hooks / arkit_prelude / ohos-arkui-binding
        ↑
arkit_shadcn / arkit_chart adapters / examples
        ↑
arkit facade
```

`arkit_animation_core` 到上层的反向依赖被永久禁止。

## 5. 最终数据模型

### 5.1 Dense ID

core 使用以下 typed dense ID：

```rust
TargetId
TargetSetId
PropertyId
TweenId
TrackId
TimelineNodeId
TimeDomainId
LabelId
CallId
InstanceId
AdapterId
AdapterTargetId
AdapterPropertyId
```

它们必须由 `oxc_index` 定义并用于所有 plan/runtime 容器。任何 target/property/track 的字符串身份都必须在 resolve/compile 阶段结束前转换为 dense ID。

稳定用户名字使用专用类型：

```rust
TargetName(Arc<str>)
PropertyName(Arc<str>)
LabelName(Arc<str>)
LayoutId(Arc<str>)
ScopeMethodName(Arc<str>)
```

禁止在 API 内接受没有语义包装的裸 `String` 作为身份。

### 5.2 Property schema

公开属性使用类型参数保持调用端类型安全：

```rust
Property<f32>
Property<Length>
Property<Angle>
Property<Color>
Property<Vec2>
Property<Vec3>
Property<Transform>
Property<Shadow>
Property<DiscreteValue>
```

每个 adapter 的 `PropertyDescriptor` 必须声明：

- `PropertyName`
- `ValueKind`
- 默认值策略
- 是否可读、可写
- interpolation strategy
- 支持的 composition modes
- unit domain 和转换规则
- dirty comparison precision
- invalidation class：transform、paint、layout、measure、discrete
- native implicit/keyframe/animator capability
- reset/revert strategy

ArkUI 内建 schema 必须覆盖所有平台确认可动画的公共属性族，而不是只覆盖当前 demo：

- opacity、visibility
- translate/scale/rotate/skew/perspective/transform origin
- position、offset
- width/height/min/max/aspect ratio
- background/font/border/foreground colors
- per-corner border radius、border width
- blur、shadow、brightness、contrast 等可用 visual effect
- clip 和离散显示属性
- adapter-specific custom properties

不支持连续插值的属性必须声明为 discrete，禁止隐式转换为 `f32`。

### 5.3 Value model

core 的 erased value 必须完整表达：

```rust
enum AnimationValue {
    Scalar(f32),
    Length(Length),
    Angle(Angle),
    Color(LinearRgba),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Transform(TransformValue),
    Shadow(ShadowValue),
    Discrete(DiscreteValue),
    Custom(CustomValue),
}
```

要求：

- `Length` 必须区分 vp、px、percent；跨单位插值必须经过 adapter resolution context。
- `Angle` 内部使用统一单位，API 提供 degrees/radians 构造。
- `Color` 必须在线性或伪线性色彩空间插值，再转换为 ArkUI ARGB。
- `TransformValue` 必须表达 3D transform 和 origin，不以字符串表示。
- `CustomValue` 必须由 adapter 注册的 value codec/interpolator 拥有，core 只持有稳定 type token 和 owned payload。
- value mismatch 必须在 resolve/compile 阶段返回 typed error，禁止帧循环 fallback。

### 5.4 Source、Resolved 与 Compiled 三层

终态必须明确分离三层：

1. `AnimationSource`：用户 builder 产生，包含 typed property、selector、label 和 function value。
2. `ResolvedAnimation`：adapter 已解析 target/property/from/unit/function value，身份已转换为稳定 slot。
3. `CompiledAnimation`：排序、composition graph、cursor table、event crossing table 全部完成，可直接进入 Engine。

`CompiledAnimation` 必须是 `Arc` 持有的不可变 plan。运行时禁止 clone 完整 plan。

function-based value 接受：

```rust
TargetContext {
    index,
    total,
    target_name,
    layout_snapshot,
    window_metrics,
}
```

它在 resolve/start/refresh 阶段计算，默认禁止每帧执行。需要每帧变化的函数必须显式声明为 `DynamicModifier`，并强制 sampled backend。

## 6. Tween、Keyframe 与 Composition 终态

### 6.1 Tween

每个 property tween 独立拥有：

```rust
TweenSpec {
    from: FromValue,
    to: ValueSource,
    delay: TimeSpan,
    duration: DurationSpec,
    priority: i32,
    easing: Easing,
    composition: Composition,
    modifier: Modifier,
}
```

`FromValue` 必须支持 explicit、current snapshot、previous tween result、relative baseline。`ValueSource` 必须支持 fixed、relative、target function、stagger distribution。

### 6.2 Keyframes

必须同时支持：

- property-local tween value 数组
- duration-based keyframe
- percentage-based keyframe
- sparse state keyframe
- timeline absolute keyframe
- 每段独立 easing/delay/duration

sparse keyframe 中未出现的属性必须保持自己的 tween chain，禁止通过完整状态默认值覆盖。

### 6.3 Composition

终态 composition：

- `Replace`：相同 target/property 上按 timeline position、priority、insertion order 选择确定性 winner。
- `Add`：在 baseline 上累加当前贡献；仅允许 descriptor 声明可加的 value kind。
- `Accumulate`：每次 iteration 将上一轮 delta 累加到下一轮 baseline。

每个 animation instance 开始时必须捕获所需 baseline snapshot。`revert` 恢复 animation 开始前 snapshot；`reset` 恢复计划初始位置；两者不得混同。

## 7. Timeline 终态

v2 只有一个多目标 `Timeline`；`TimelineGroup` 被永久删除。

Timeline node 必须支持：

- animation
- timer
- set
- call
- nested timeline
- label
- barrier/synchronization point

Position 表达式必须支持：

- absolute time
- label
- label ± offset
- previous child start ± offset
- previous child end ± offset
- timeline end ± offset
- normalized percentage

nested timeline 必须在 compile 阶段 flatten 成同一个 plan，同时保留事件和 callback ownership；禁止运行时嵌套 Player。Flatten 后每个 node 只持 `TimeDomainId`，domain parent 必须指向更早的 dense slot，从结构上保证无环。child 的 delay、loop delay、playback rate/easing、reverse、alternate 和 finite/infinite iterations 必须保存在 immutable domain table，由纯函数 mapper 计算 local logical time。

Timeline defaults 必须以 immutable inherited settings 在 compile 阶段展开，禁止每帧向父级查找。

## 8. 时间与控制语义冻结

### 8.1 时间

- core 内部统一使用单调纳秒或微秒整数，不以累积 `f32` 毫秒作为权威时间。
- frame timestamp 必须来自 ArkUI frame callback，经 `FrameClock` 统一规范化。
- `ManualClock` 必须产生和平台 clock 相同的 tick 输入。
- playback rate 和 playback easing 作用于 logical time，不直接修改 property easing。
- zero-duration tween 在 crossing 时原子切换并按规则触发事件。

### 8.2 Iteration

API 必须使用无歧义模型：

```rust
IterationCount::Finite(NonZeroU32)
IterationCount::Infinite
```

finite 数值表示总播放次数，不使用“首次播放之外的 loop 数”语义。`completed_iterations` 在完整越过一次 active duration 后递增。

alternate 在 iteration boundary 反转局部方向；reversed 反转实例初始方向。二者组合必须由 trace 测试覆盖。

### 8.3 Controls

- `play`：Idle/Completed/Cancelled/Reverted 从定义起点开始；Paused 从当前位置继续；Running no-op。
- `pause`：冻结 logical time，不触发 complete。
- `resume`：只允许 Paused；其他状态 no-op。
- `restart`：reset 后立即 play。
- `reverse`：只切换方向；不得隐式改变 Paused/Idle 状态。
- `alternate`：切换后续 iteration mode，不隐式 play。
- `seek`：设置当前位置并渲染该位置；默认不触发穿越的 call/loop/complete。
- `seek_with_events`：显式要求按方向触发 crossing events。
- `complete`：渲染当前方向终点并执行一次 complete 序列。
- `cancel`：停止并保持当前视觉值，触发 cancel，不触发 complete。
- `reset`：回到计划初始值和 Idle，不恢复计划开始前样式。
- `revert`：停止并恢复开始前 baseline snapshot，清理 instance-owned effect。
- `stretch`：重新映射 plan 时间，不改变 normalized progress。
- `refresh`：重新解析 target/function value/unit/layout baseline，保持 logical state。

### 8.4 Callback 顺序

自然 tick 的顺序固定为：

```text
begin（仅首次进入 active range）
before_update
core advance/sample/compose
update
adapter batch render
render
loop（如果 crossing）
complete/cancel（如果 terminal）
async waiter resolve
```

用户 callback 必须在 Engine 可变借用释放后执行。callback 产生的控制命令进入 command queue，在当前事件批次之后处理，禁止重入 Engine。

Timeline `call` 必须声明 `CallPolicy::{ForwardOnly, BothDirections, Once}`。普通 `seek` 不触发 call；`seek_with_events` 根据 policy 触发。

## 9. Easing、Spring、Modifier 与 Stagger 终态

### 9.1 Easing

必须覆盖：

- linear
- quad/cubic/quart/quint 的 in/out/in-out
- sine/expo/circ/back/bounce/elastic 的 in/out/in-out
- cubic Bézier
- steps 及 jump mode
- piecewise linear points
- irregular/rough seeded easing
- physical spring
- custom pure easing

`SpringSpec` 必须包含 mass、stiffness、damping、initial velocity、rest speed、rest displacement，并能计算 natural duration。用户显式 duration 时必须有确定的 time scaling 规则。

native curve converter 只能在完全保持语义时转换；否则返回 sampled requirement。

### 9.2 Modifier

内建 modifier：

- clamp
- round/precision
- snap
- wrap
- map range
- lerp/damp
- chain

custom modifier 必须是 pure function；其存在会成为 native lowering capability 条件。

### 9.3 Stagger

终态 `Stagger<T>` 必须支持：

- time 和任意可插值 value range
- start offset
- first/center/last/index/random/normalized coordinate origin
- normal/reverse direction
- 1D、显式/自动 2D、显式 3D grid
- x/y/z axis
- easing
- modifier
- jitter range
- deterministic seed
- explicit total

stagger 在 resolve 阶段生成 target-local fixed values，不在每帧重新计算。

## 10. Target adapter 终态

### 10.1 Adapter contract

`arkit_animation` 定义 object-safe、UI-thread-owned `TargetAdapter`。它必须提供：

- selector resolution
- target lifecycle/version
- property schema
- baseline snapshot
- unit/layout resolution
- dirty batch apply
- reset/revert
- native capability query
- native lowering/instance construction
- diagnostic naming

adapter registry 使用 `AdapterId` 和 adapter-owned dense target handle。Engine plan 只保存 `(AdapterId, AdapterTargetId)`，不保存 `ArkUINode`、raw pointer 或 target 名字。

### 10.2 ArkUI adapter

ArkUI adapter 必须：

- 通过 `ArkNodeRef` 注册/注销稳定 target。
- 检测 duplicate target name，而不是静默保留第一个。
- 批量读取仅计划需要的 baseline 属性。
- 只写 dirty properties。
- 按 invalidation class 排序：transform/paint 优先，layout/measure 独立批次。
- 对写入错误做 instance-level rate limit，禁止每帧重复刷日志。
- target unmount 后在下一 command/tick 中移除 binding，并按 instance policy 继续其余 target 或 cancel。

### 10.3 Drawing/Canvas adapter

Drawing adapter 必须使用同一 Engine 和 property schema，支持：

- object transform
- opacity/color/stroke/fill
- path progress
- numeric uniforms/custom scalar/vector
- chart data transition 所需的 typed value

`arkit_chart` 的现有私有动画时钟必须在 Drawing adapter 验收后迁移并删除。

### 10.4 用户自定义 adapter

用户可注册自定义 target/property，但必须：

- 在 scope/host 初始化阶段注册。
- 提供稳定 adapter/property name。
- 提供 value kind 与 interpolation/composition contract。
- 不允许在 frame apply 中做文件 IO、网络 IO 或 target discovery。

## 11. 单一 Engine 终态

### 11.1 所有权

一个 Arkit UI root 只允许一个 `AnimationHost` 和一个 `AnimationEngine`。`arkit_entry_root` 自动安装 provider；嵌套 animation scope 只管理生命周期/defaults，不创建第二个 clock。

Engine 在 UI 线程拥有：

```rust
AnimationEngine {
    instances: IndexVec<InstanceId, AnimationInstance>,
    active: Vec<InstanceId>,
    command_queue: VecDeque<EngineCommand>,
    event_queue: Vec<EngineEvent>,
    output_slots: IndexVec<EngineOutputId, EngineOutputSlot>,
    ordered_outputs: Vec<EngineOutputId>,
    frame_batch: FrameBatch,
    scratch: EngineScratch,
}
```

共享 plan 使用 `Arc<CompiledAnimation>`。Engine mutable state 保持单 owner；禁止 `Arc<Mutex<...>>`。

### 11.2 每帧阶段

```text
1. drain commands
2. resolve pending target/layout invalidations
3. advance logical clocks
4. move per-track cursors / binary-search seeks
5. sample active tweens
6. compose per target/property contributions
7. compare previous output and build dirty FrameBatch
8. adapters sequentially apply sorted batches
9. publish EngineEvents
10. update observed status/progress snapshots
11. schedule next frame only if work remains
```

正常正向播放必须使用 cursor O(1) 前进；seek/reverse jump 使用预编译索引二分查找。adapter-global output slot 在注册/注销冷路径维护 `(AdapterId, AdapterTargetId, invalidation class, AdapterPropertyId)` 顺序；正常帧只按该稠密顺序扫描和 append，禁止在 final batch 排序或去重。

### 11.3 Signal 策略

Engine 内部禁止使用 Dioxus Signal。Controls 提供：

- 非响应式即时 getter
- callback/subscription
- 显式 reactive snapshot hook

只有存在 reactive observer 时才更新对应 Signal；禁止所有 animation 每帧无条件触发 Dioxus 调度。

### 11.4 空闲策略

- 没有 Running instance、pending layout measure、gesture inertia 或 scroll sync 时，FrameDriver 不申请下一帧。
- pause 的 instance 不保持 root frame callback。
- 新 command、native callback、gesture/scroll event 负责重新唤醒 Engine。

## 12. Native lowering 终态

### 12.1 Backend 类型

```text
ArkUiImplicitBackend   -> Animation / animate_to
ArkUiKeyframeBackend   -> KeyframeAnimation
ArkUiAnimatorBackend   -> Animator / native progress clock
SampledBackend         -> core Engine + ArkUI batch writer
```

### 12.2 Capability lattice

每个 backend 必须声明：

- property kinds
- easing kinds
- per-property timing
- composition modes
- pause/resume/reverse/cancel/seek
- callback phases
- infinite iteration
- dynamic modifier
- layout invalidation

`NativeLowerer` 根据完整 plan 和所需 controls 选择 backend。`ExecutionPolicy`：

- `Auto`
- `NativePreferred`
- `SampledOnly`
- `NativeOnly`

`NativeOnly` 无法完整表达时必须构建失败。其他模式 fallback 必须产生可查询 `LoweringReport`，禁止静默语义降级。

### 12.3 Controls 与 native path

完整 `AnimationControls` 的默认语义不可因 backend 改变：

- backend 不支持某项 control 时，Lowerer 不得为该 controllable instance 选择它。
- ArkUI implicit/keyframe backend 主要用于声明为 one-shot、controls/callback 要求受限的 animation island。
- 需要任意 seek、per-property timing、composition 或 dynamic modifier 的 timeline 进入 sampled backend。
- Animator 可以作为 native progress clock，但 property sampling 和 callback 顺序仍以 core logical plan 为权威。

native object 必须由 `NativeAnimationInstance` 持有到 finish/cancel/drop。必须删除 fixed 60-second retention。

## 13. Animatable 与 Scope 终态

### 13.1 Animatable

`Animatable<Target>` 提供缓存的 imperative getter/setter：

- 每属性 unit/duration/easing/modifier defaults
- 新 setter 从当前 sampled/native progress retarget
- 同一属性更新合并，不创建新的 Timeline graph
- getter 返回 logical output，而不是等待 ArkUI attribute round-trip
- `revert` 恢复创建前 baseline

Animatable 只是 Engine instance 的高频 API，不拥有私有时钟。

### 13.2 AnimationScope

每个 scope 必须拥有：

- target/instance/callback/layout/scroll/draggable 生命周期集合
- defaults
- named methods
- responsive conditions
- cleanup/revert policy
- `refresh`
- `keep_time`

responsive condition 使用 typed `WindowCondition` 和 `WindowMetrics`，禁止复制 CSS media query 字符串语法。

Dioxus scope drop 时必须先取消事件源，再 cancel/revert instance，最后注销 target；callback 禁止访问已卸载 node。

## 14. Layout 与 Presence 终态

### 14.1 Layout snapshot

`LayoutEngine` 必须拥有 typed snapshot：

```rust
LayoutSnapshot {
    root,
    nodes: IndexVec<LayoutNodeId, LayoutNode>,
    window_metrics,
    scroll_offsets,
    generation,
}
```

每个 `LayoutNode` 包含稳定 `LayoutId`、parent、frame、transform、visibility、clip、z-order、mount state。snapshot 读取在布局阶段集中完成，禁止每个 tween 自己查询 layout。

### 14.2 FLIP

统一流程：

```text
record old snapshot
Dioxus mutation/render
record new snapshot
compute delta and topology changes
emit transform/size/property animation source
compile/register with the same Engine
cleanup temporary inverse state
```

reorder、size change、enter、exit、swap parent、shared element 都必须基于这一套 snapshot/delta model。

### 14.3 Presence

`AnimatePresence` 必须：

- 在 logical child 被移除后保留 native/Dioxus-owned leaving representation。
- 等 exit animation complete/cancel policy 后真正卸载。
- 支持多 child、wait/sync/pop-layout modes。
- 支持 enter/exit stagger。
- scope drop 时确定性清理 leaving nodes。

不得用 fixed timeout 模拟 animation completion。

### 14.4 Shared element / swap parent

稳定 `LayoutId` 在 old/new snapshot 中匹配。跨父节点动画通过 overlay projection 或 snapshot clone 执行；最终 owner 仍是 Dioxus 新树。临时 overlay 必须由 Layout instance 持有并在 terminal event 清理。

## 15. Gesture 与 Scroll 终态

### 15.1 Draggable

`Draggable` 必须支持：

- x/y/both axes
- trigger 与 target 分离
- container/bounds/padding
- drag threshold
- value modifier/map-to
- snap points/grid/function
- velocity estimator
- min/max velocity
- inertia
- mass/stiffness/damping release
- container friction/release friction
- scroll threshold/speed 和 auto scroll
- grab/drag/update/release/snap/settle/resize callbacks
- enable/disable/refresh/reset/revert/stop

gesture 事件只生成 Engine commands。release inertia/spring 必须编译成同一 property plan，不得创建独立 timer。

### 15.2 Scroll observer

必须支持：

- root/container/target
- horizontal/vertical axis
- numeric、position shorthand、relative、min/max thresholds
- enter/leave/forward/backward callbacks
- repeat/once
- method sync、progress sync、smooth/eased sync
- velocity 和 direction
- linked animation
- in-view animation/scroll helpers
- refresh/revert

scroll event 更新 latest input 并唤醒 Engine；同一 frame 多次 scroll event 必须 coalesce。

## 16. Diagnostics、错误与可观测性

### 16.1 Typed errors

必须提供：

```text
AnimationBuildError
AnimationResolveError
AnimationCompileError
AnimationRuntimeError
AdapterError
NativeLoweringError
LayoutAnimationError
```

覆盖至少：unknown/duplicate target、unknown property/label、value kind mismatch、unit resolution failure、invalid duration/easing、unsupported composition、missing baseline getter、native-only unsupported、disposed scope/instance。

library crate 禁止使用 `anyhow` 作为公开错误。

### 16.2 Lowering report

每个 instance 可查询：

- selected backend
- native/sample reasons
- unsupported feature list
- target/property count
- compiled tween count
- layout-affecting property count
- estimated per-frame work

### 16.3 Performance counters

debug/diagnostic build 提供：

- active instances/tweens/targets
- frame compute/apply duration
- dirty writes
- skipped unchanged writes
- frame callbacks requested
- target misses
- fallback count
- allocations in benchmark harness

禁止在 release 每帧格式化日志字符串。

## 17. 公共 API 终态与迁移

### 17.1 保留的概念

- `Animation`
- `Timeline`
- `AnimationControls`
- `AnimationTarget`
- `AnimationScope`
- `Animatable`
- `Easing`
- `Stagger`
- `MountTransition`
- `TransitionPreset`
- `AnimatePresence`
- `LayoutAnimation`
- `Draggable`
- `ScrollObserver`

### 17.2 必须删除的 v1 概念

- `AnimationState`
- `AnimationDelta`
- `TimelineGroup`
- `TimelineGroupControls`
- `TimelineGroupError`
- `TimelineTrack`
- `use_timeline_group`
- v1 `Player`
- v1 `GroupPlayer`
- `RETAINED_ANIMATIONS`
- 当前 `group.rs`
- 两套 frame scheduling/tick/callback 状态机
- 每帧 `BTreeMap<String, ...>` target sampling

### 17.3 迁移策略

仓库当前版本为 `0.1.0`，采用一次性 workspace cutover：

1. v2 在新 core 和 `arkit_animation` 私有模块中完成，不对外发布半成品 API。
2. core、adapter、engine、controls、Layout/Gesture/Scroll 达到本文语义和测试门槛后冻结公开 API。
3. 同一迁移批次修改 `arkit` facade、`arkit_shadcn`、`arkit_chart` 和所有 examples。
4. 调用方全部编译并通过设备验证后立即删除 v1。
5. 禁止提交长期 `type TimelineGroup = Timeline` 或 deprecated wrapper。

`MountTransition` 和 `TransitionPreset` 可以保持调用形态，但实现必须完全迁移到 v2 Timeline/Scope/Presence。

## 18. 实施拓扑

阶段表示依赖顺序，不表示架构简化。每一阶段都必须使用本文的最终类型和边界。

### Phase 0：语义冻结与基线（2–3 天）

交付：

- 本文评审通过并标记 design frozen。
- `animation-semantics.md`：controls、iterations、callbacks、composition、seek crossing 的可执行语义表。
- 当前 v1 golden traces。
- v1 结构、测试和已知热路径静态基线。

完成门槛：任何后续实现都能判断是语义一致、显式 breaking，还是 bug。设备 benchmark 延后到能力闭环后的 Phase 10，不得打断 Compiler/Engine/Adapter 主线。

### Phase 1：终态 core skeleton 与值系统（5–6 天）

交付：

- 新 `arkit_animation_core` crate 和最终文件边界。
- 所有 dense ID、time/value/property/easing/modifier/composition 最终类型。
- Source/Resolved/Compiled 三层接口。
- typed errors。
- host test harness 和 `ManualClock`。

禁止交付固定属性 state 作为临时 core。

### Phase 2：Resolver、Compiler 与完整 Timeline IR（7–8 天）

交付：

- target/property/label interning。
- function value、unit、baseline resolution contract。
- per-property tween chains。
- nested timeline flatten。
- call/timer/set/event crossing tables。
- replace/add/accumulate graph。
- forward cursor 和 seek index。

完成门槛：相同 source/target snapshot 必须产生 byte-for-byte deterministic plan trace。

### Phase 3：单一 Engine 与完整播放器语义（7–8 天）

交付：

- Engine、command queue、event queue、frame batch、scratch reuse。
- 本文所有 controls、iteration、callback semantics。
- async completion。
- observed/unobserved progress strategy。
- zero-allocation steady-state core 结构约束与分配审计；性能数据统一在 Phase 10 全量验收采集，不新增独立 benchmark feature。

完成门槛：Timeline、多目标、timer、call、Animatable retarget 都由同一个 Engine 推进。

### Phase 4：终态 adapter registry 与 ArkUI adapter（7–8 天）

交付：

- TargetAdapter/PropertySchema/AdapterRegistry 最终 contract。
- ArkUI selector resolver、target store、snapshot reader、dirty writer。
- root AnimationHost/FrameDriver。
- facade 自动安装 provider。
- target lifecycle、duplicate/missing diagnostics。

完成门槛：100 target sampled animation 每帧不进行 target/property 字符串 lookup 和容器分配。

### Phase 5：Native lowering 全能力矩阵（6–7 天）

交付：

- 四 backend、capability lattice、ExecutionPolicy。
- LoweringReport。
- native handle 生命周期和真实 finish/cancel callback。
- native/sample trace parity tests。
- 删除 fixed retention。

完成门槛：任何 fallback 都可解释，任何 native path 都不改变公开 controls/callback 语义。

### Phase 6：终态 API、Animatable、Scope、Stagger（6–7 天）

交付：

- 最终 public builders 和 re-export。
- AnimationScope lifecycle/defaults/responsive/named methods。
- Animatable getter/setter/retarget。
- 完整 Stagger。
- MountTransition 基于 v2 重写。

完成门槛：不暴露 core dense ID、raw ArkUI handle 或实现模块。

### Phase 7：Layout、Presence、Shared Element（8–10 天）

交付：

- LayoutSnapshot/FLIP。
- reorder/size/enter/exit/swap parent/shared element。
- AnimatePresence modes 和 leaving lifecycle。
- layout benchmark 和 device visual acceptance。

完成门槛：退出节点无 timeout，所有临时 overlay/snapshot 在 terminal event 后释放。

### Phase 8：Draggable 与 Scroll（8–10 天）

交付：

- 本文完整 Draggable/Scroll contract。
- velocity/inertia/spring/snap/auto-scroll。
- scroll threshold/progress/smooth sync。
- 同一 Engine 的 gesture/scroll command integration。

完成门槛：不存在 gesture/scroll 私有 frame loop。

### Phase 9：Drawing adapter、Chart 迁移与 v1 删除（6–8 天）

交付：

- Drawing adapter。
- `arkit_chart` 动画接入统一 Engine。
- examples/shadcn/facade 全部迁移。
- 删除第 17.2 节所有 v1 类型、文件和状态机。

完成门槛：workspace 搜索不到 v1 public symbol 和第二套 animation clock。

### Phase 10：全量验证、文档与发布门槛（5–6 天）

交付：

- host/core/adapter tests。
- OpenHarmony builds。
- device performance、长稳、交互、视觉报告。
- API guide、semantics、backend diagnostics、Layout/Gesture/Scroll 示例。
- upgrade notes。

完成门槛见第 21 节。

单人预计 60–73 工程日。两名熟悉 Rust/ArkUI 的工程师可以在 Phase 3 后并行推进 native adapter 与 Layout/Gesture，但 Compiler、Engine、cutover 仍是串行关键路径。

## 19. 测试合同

### 19.1 Core host tests

必须覆盖：

- 所有 value kind 的插值、unit resolution 和 mismatch。
- 所有 easing endpoint、finite、NaN、invalid parameter。
- physical spring natural/explicit duration。
- delay、loop delay、zero duration、infinite iteration。
- reverse、alternate、playback rate/ease 的所有组合。
- 一帧跨越多个 iteration/event。
- seek/seek-with-events 前后向 crossing。
- sparse/property-local/percentage keyframes。
- nested timeline flatten。
- replace/add/accumulate。
- timer/call/set/callback ordering。
- target function value 和 refresh。
- 完整 Stagger 参数矩阵和 seeded determinism。
- cancel/reset/revert baseline semantics。
- callback 产生新 command 的非重入行为。
- deterministic plan/frame traces。

时间测试禁止使用 sleep。

### 19.2 Adapter tests

fake adapter 必须覆盖：

- target mount/unmount/rebind/version。
- duplicate/missing target。
- baseline snapshot/read failure。
- dirty write 与 unchanged skip。
- property ordering/invalidation class。
- adapter error rate limit。
- custom property/value/interpolator。
- Engine idle wake/sleep。

### 19.3 Native parity tests

对同一 plan 在 fixed sample times 比较 native callback trace 与 sampled trace：

- simple transform
- multi-property color/size
- keyframes
- iterations/alternate/reversed
- cancel/finish
- native fallback cases

浮点比较使用 property descriptor precision，不使用全局随意 epsilon。

### 19.4 Layout/Gesture/Scroll tests

- reorder、resize、enter/exit、swap parent、shared element snapshot delta。
- leaving child 生命周期和 scope drop。
- velocity estimator、bounds、snap、spring settle。
- scroll threshold crossing、direction、coalescing、smooth sync。
- gesture/scroll 中 target unmount。

### 19.5 Property/fuzz tests

为 Engine 状态机增加 property-based tests：

- 任意 command/tick 序列不得 panic。
- logical time 必须在合法区间或定义的 infinite domain。
- complete/cancel callback 最多一次。
- revert 后所有 touched property 等于 baseline。
- deterministic seed 产生相同 trace。

## 20. 性能与内存门槛

固定参考设备后，必须记录 60Hz 和设备支持时的 120Hz 数据。

硬门槛：

- 编译完成的固定 plan 在 steady-state core tick 零堆分配。
- 正常帧禁止排序、字符串构造、target discovery 和 property schema lookup。
- idle Engine 零 frame callback。
- 100 transform target：Engine compute + ArkUI apply p95 小于 4ms。
- 500 transform target stress：p95 小于 8ms。
- 100 target 相比当前 TimelineGroup baseline，CPU 时间至少下降 40%。
- 未变化属性写入为零。
- transform、paint、layout 三类分别统计；layout 属性不得掩盖 Engine 自身成本。
- infinite animation 连续运行 30 分钟，Rust retained heap 增长小于 1 MiB。
- 反复 mount/unmount 1,000 次后 instance、target、callback、native handle 数量回到基线。
- scope drop 后下一次 command/tick 释放所有 owned instance。
- 标准 60Hz 用例连续运行 10 分钟，不得出现 Engine 引起的持续掉帧。

性能不达标必须先修复数据布局和批处理，禁止通过降低默认帧率掩盖。

## 21. 验证命令与完成定义

### 21.1 每个核心 PR

```sh
cargo test -p arkit_animation_core
cargo check -p arkit_animation
cargo clippy -p arkit_animation_core -p arkit_animation --all-targets -- -D warnings
cargo fmt --all -- --check
cargo metadata --format-version 1 --no-deps
```

### 21.2 Workspace cutover

```sh
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
rg -n "AnimationState|AnimationDelta|TimelineGroup|use_timeline_group|GroupPlayer|RETAINED_ANIMATIONS" crates examples docs
```

最后一条必须只命中迁移说明或为零；生产代码命中视为未完成。

### 21.3 OpenHarmony

```sh
cd examples/animation
ohrs build --arch aarch

cd ../shadcn_showcase
ohrs build --arch aarch

cd ../chart
ohrs build --arch aarch
```

成功后必须逐个通过 `app/run.sh` 打包安装；一次只验证一个 example。host `cargo check` 不构成目标平台完成证据。

### 21.4 最终完成定义

以下条件必须同时满足：

- 本文定义的 core/adapter/engine/layout/gesture/scroll 边界全部落地。
- 所有时间和 controls 语义只有一份 core 实现。
- 所有动画源使用同一个 root Engine；不存在第二套 frame loop。
- Source/Resolved/Compiled 三层完整存在。
- property/value/adapter 是终态扩展模型，不需要为新增属性重写 core。
- native lowering capability 可查询且无静默降级。
- Layout/Presence/Draggable/Scroll 使用同一 Engine。
- Drawing adapter 完成，Chart 私有 animation clock 删除。
- 第 17.2 节所有 v1 类型和代码删除。
- host、clippy、fmt、workspace checks 全通过。
- 三个 OpenHarmony example 构建成功并完成设备验收。
- 第 20 节全部性能和内存门槛满足。
- API、语义、adapter、diagnostics、Layout/Gesture/Scroll 文档完整。

任何一项缺失都只能标记为阶段完成，不能标记 Animation v2 完成。

## 22. 风险与强制处理方式

### ArkUI native backend 能力不对齐

处理：capability lattice + `NativeOnly` build error + 可查询 fallback。禁止模拟不存在的 native control。

### Layout measurement 与动画写入互相反馈

处理：集中 snapshot phase、invalidation class、FLIP 优先 transform。禁止 tween 自行查询 layout。

### Dioxus callback 重入和高频 Signal 重渲染

处理：Engine event queue、borrow 释放后 callback、按观察者启用 reactive snapshot。禁止 Engine 内部使用 Signal。

### Exit child 已被 Dioxus 卸载

处理：Presence owner 保留 leaving representation，以真实 animation terminal event 驱动卸载；禁止 timeout。

### 自定义 adapter 破坏实时性

处理：adapter contract 禁止热路径 discovery/IO；diagnostic build 统计 apply time；超预算 adapter 独立报告。

### 长迁移期双实现漂移

处理：v1 只维持当前调用方，不新增能力；所有新能力只进入 v2；cutover 后同批删除 v1，不保留 alias。

## 23. 评审检查表

每个实现 PR 必须回答：

- 是否使用本文的最终类型，而不是过渡结构？
- 文件和 crate owner 是否与第 4 节一致？
- 是否新增了第二套时钟、Player、target registry 或 property model？
- 是否把字符串、HashMap、排序或分配带入帧循环？
- 是否改变了第 8 节语义？若改变，本文是否先更新并重新评审？
- native path 是否有 capability 证据和 sampled parity test？
- 是否包含明确的旧代码删除项？
- host test、OpenHarmony build、设备验证分别完成了哪些？
- 性能计数器是否证明没有热路径退化？

不满足终态边界的“先跑起来”实现不得合入。
