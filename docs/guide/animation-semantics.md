# Animation v2 播放语义

> 状态：Phase 0 冻结候选稿  
> 日期：2026-07-13  
> 架构来源：[Animation 终态架构与实施计划](./animation-engine-final-design-plan.md)

本文是跨 sampled、ArkUI implicit、ArkUI keyframe 和 Animator backend 的统一行为合同。backend 不得自行解释这些操作；无法完整保持语义时必须拒绝 lowering 或回退 sampled backend。

## 时间模型

- core 使用单调纳秒整数作为权威时间。
- duration、delay、loop delay 必须非负。
- `IterationCount::Finite(n)` 的 `n` 表示总播放次数，`n >= 1`。
- `IterationCount::Infinite` 不存在逻辑终点，只有 cancel/revert 才能终止。
- playback rate 必须为有限正数；方向由 direction 独立表达。
- playback easing 作用于 iteration logical progress；property easing 作用于单个 tween progress。
- zero-duration tween 在时间边界被跨越时原子更新。

## 状态

| 状态 | 含义 |
| --- | --- |
| Idle | 已注册但未播放，视觉位于计划起点 |
| Scheduled | 已收到 play，但 target/backend 尚未进入首个 active frame |
| Running | logical time 正在推进 |
| Paused | 保留当前位置，不申请仅由该实例产生的 frame |
| Completed | finite animation 已自然或显式 complete |
| Cancelled | 停止并保留当前视觉值 |
| Reverted | 停止并恢复实例开始前 baseline snapshot |

## Controls

### play

- Idle：进入 Scheduled，并在 target 可用后从计划起点开始。
- Scheduled/Running：no-op。
- Paused：等同 resume，从当前位置继续。
- Completed/Cancelled/Reverted：重建本轮 baseline，回到当前 direction 对应起点并开始。

### pause / resume

- `pause` 只对 Running/Scheduled 生效，进入 Paused，触发一次 pause。
- `resume` 只对 Paused 生效；不得重新触发 begin。
- pause 期间 wall-clock 时间不得计入 logical time。

### restart

严格等价于 `reset` 后 `play`。它创建新一轮 begin/complete 生命周期。

### reverse

- 只反转当前播放方向。
- 不隐式 play 或 resume。
- Running 保持 Running，Paused 保持 Paused，Idle 保持 Idle。
- 位于边界时只改变下一次推进方向，不自动跳到对侧边界。

### alternate

- 切换后续 iteration boundary 是否反向。
- 不改变当前 direction，不隐式播放。

### seek

- clamp 到 finite active range；infinite animation 按当前 iteration duration 解析位置。
- 立即 sample/compose/render 目标位置。
- 普通 seek 不触发被跨越的 call、loop、begin、complete。
- 如果 seek 后实例原为 Running，继续从新位置运行；原为 Paused/Idle 时保持原状态。

### seek_with_events

- 与 seek 相同，但按移动方向处理 crossing table。
- call 是否触发取决于 `CallPolicy`。
- crossing 多个 iteration 时按逻辑顺序发出 loop。
- 到达 finite terminal boundary 时触发 complete。

### complete

- finite animation 渲染当前方向对应的 terminal value。
- begin 尚未发生时，先触发 begin。
- before_update/update/render 在 complete callback 之前执行。
- complete 最多触发一次，随后状态为 Completed。
- infinite animation 的 complete 返回 typed runtime error。

### cancel

- 停止 logical/native execution，保留最后已渲染值。
- 触发一次 cancel，不触发 complete。
- 已 Completed/Cancelled/Reverted 时 no-op。

### reset

- 停止执行并回到计划的逻辑起点。
- 清零 completed iterations 和 event crossing cursor。
- 渲染计划起点值，状态变为 Idle。
- 不恢复实例启动前 baseline。

### revert

- 停止执行并恢复实例首次开始时捕获的 baseline snapshot。
- 清除 instance-owned temporary layout/overlay/native state。
- 触发 terminal cleanup，不触发 complete。
- 状态变为 Reverted。

### stretch

- 重新映射所有 timeline 时间位置和 tween duration。
- 保持 normalized progress、direction、iteration 和运行状态。
- zero-duration 节点保持 zero-duration crossing。

### refresh

- 重新解析 target set、function values、unit context、layout snapshot 和 baseline reader。
- 保持 logical position、direction、iteration、运行状态。
- 已卸载 target 按 instance target policy 移除或终止。

## Iteration 与方向

- 初始 direction 由 reversed 决定。
- 非 alternate：每次 iteration 从相同逻辑起点重新开始。
- alternate：每次非终止 iteration crossing 后 direction 翻转。
- loop delay 位于两次 iteration 之间，期间保持上一 iteration terminal value。
- playback easing 每个 iteration 独立采样。
- 一帧跨越多个 iteration 时必须完整处理 overflow、loop delay 和 crossing events。

## Callback 顺序

自然 frame：

```text
begin（如果首次进入 active range）
before_update
advance / sample / compose
update
adapter render
render
loop（0..N 次）
complete 或 cancel（如果进入 terminal）
resolve async waiters
```

规则：

- callback 在 Engine 可变借用释放后执行。
- callback 内的 controls 操作进入 command queue，不得重入当前 tick。
- begin、complete、cancel 每轮最多一次。
- update 可以在 delay/loop delay 期间触发，但视觉值不变时 adapter 不得重复写属性。
- render 只在 adapter batch 已提交后触发。
- `tick` 返回唯一单调 `FrameId`；host 可以先 drain begin/before_update/update 等 pre-commit event，然后按 FrameBatch 顺序调用 adapter。只有 adapter apply 全部成功后才能以该 `FrameId` acknowledgement。
- acknowledgement 严格 FIFO 且同时最多一个 pending frame；pending frame 未 ack 时下一次 tick 返回 typed error，错误或过期 FrameId 不得释放任何 post-commit event。
- ack 按 dense `InstanceId` 顺序发布 Render，然后发布该帧 Call/Loop、Complete/Cancel/Revert、StateChanged，最后发布 `Settled { outcome }`。future facade 只能消费 Settled 解析 waiter，禁止直接观察 state 推测完成。
- Cancel 仍经过 frame ack 后发布 terminal/Settled，但不得伪造 Render；Revert 只有存在需恢复的 compiled output 时产生 Render；seek、stretch、alternate、reset 和显式 complete 发生采样时必须产生 before_update/update，并在 ack 后产生 Render。

## Engine command ownership

- 每个 UI root 只有一个 `AnimationEngine`；plan 使用 `Arc<CompiledAnimation>` 共享，instance mutable state 只由 Engine 持有。
- controls 只向 FIFO command queue 写入；command 在 tick 起始阶段串行 drain，callback 产生的新 command 不得重入当前处理栈。
- instance 使用 dense `InstanceId`；未知或已 remove 的 ID 产生 typed runtime error event，不得 panic。
- event queue 必须由 host 显式 drain；Engine 不持有 Dioxus Signal、adapter callback 或跨线程锁。
- FrameBatch 的借用只在 pending `FrameId` 生命周期内有效；host ack 前不得再次 tick，ack 后不得再次应用旧 batch。
- infinite extent 的 instance 接收 complete 时产生 typed runtime error，状态和 logical time保持不变。

## Timeline call

同一逻辑时刻的 timeline event 固定按 `set → call → barrier` 执行；同类 event 保持 source insertion order。这样 call 能观察到同刻 set 的结果，同时所有 backend 使用同一确定性 event table。

| Policy | 正向自然播放 | 反向自然播放 | 普通 seek | seek_with_events |
| --- | --- | --- | --- | --- |
| ForwardOnly | 触发 | 不触发 | 不触发 | 仅正向 crossing |
| BothDirections | 触发 | 触发 | 不触发 | 按 crossing 方向触发 |
| Once | 首次 crossing 触发 | 首次 crossing 触发 | 不触发 | 首次 crossing 触发 |

实现约束：

- Compiler 必须为每个 `TimeDomainId` 生成连续 event range；正常 tick 和 seek 禁止扫描其他 domain 的 event。
- 正向 crossing 按 time、event kind、source insertion order；反向 crossing 按 time 逆序，但同一时刻仍保持 event kind 和 source insertion order。
- 一帧跨越多个 iteration 时必须依次处理上一轮 remainder、完整中间 iteration、当前轮 prefix；不得漏掉 call 或 loop。
- `Once` 状态属于 animation instance，reset/restart/revert 清除；普通 seek 不修改，seek-with-events 首次 crossing 后锁定。
- SetAlternate 修改后必须立即重新映射当前 root-domain local time，但不得隐式 play 或触发 crossing event。

## Timeline position resolution

- absolute position 直接使用 root logical time。
- label 必须先定义后引用；forward reference 和重复 label 返回 typed resolve error。label 本身不改变 previous child。
- previous start/end 指最近一个非 label node；没有 previous child 时以零点为基准。
- timeline end 指解析到当前位置时已累计的最大 node end，不指尚未解析的最终时长。
- normalized percentage 必须在 `0..=1`，相对于当前位置已累计 timeline end 计算；结果按纳秒四舍五入。
- 正 offset 使用 checked arithmetic，溢出返回 typed error；负 offset 在 root logical zero 截断。
- timer 只推进 position graph，不产生运行时 callback event；set/call/barrier 进入同一个 event table。
- nested timeline 递归 flatten 到同一个 plan，并为每层分配 `TimeDomainId`；tween、track、set/call/barrier event 都显式关联所属 domain。
- child domain 必须保留 offset、delay、loop delay、playback rate/easing、reverse、alternate 和 finite/infinite iterations；`TimeDomainMapper` 将 parent time 映射为 phase、iteration、direction 与 local time。
- nested finite extent 按 rate 缩放 active duration，并计入 iteration 和 loop delay；nested infinite iterations 将 parent extent 提升为 `TimeExtent::Infinite`。对 infinite end 使用 TimelineEnd、PreviousEnd 或 percentage 必须返回 typed error。
- domain parent 只能引用更早创建的 dense slot；Compiler 必须拒绝缺失 root、forward parent 或环，不允许运行时嵌套 Player。

## Composition

- Replace：timeline position、显式 priority、insertion order 依次决定 winner。
- Add：在实例 baseline 上累加当前 active contribution。
- Accumulate：每个 iteration 的 terminal delta 成为下一 iteration 的累计 baseline。
- tween 到达 end 后保留 terminal contribution；后续 Replace winner 可以覆盖它，Add/Accumulate contribution 则继续参与当前 iteration 的 composition。
- Compiler 必须把 start/end boundary、Replace winner、Add/Accumulate contributor 预编译为 per-property segment graph；运行时禁止重新排序或按字符串查找 property。
- Compiler 必须再把同一 `(TargetId, PropertyId)` 的所有 TimeDomain track 和 Set event 收敛到一个 `CompiledOutput`；跨 domain Replace 在 output 层决胜，Add/Accumulate contribution 在 output baseline 上统一累加。
- Set 是从其 domain local time 到达指定位置起持续生效的 Replace 状态；反向播放或 seek 回到该位置之前时自动撤销到前一个 winner/baseline，不依赖一次性 mutable event side effect。
- 多实例写同一 adapter-global property 时形成 activation layer：Play/Resume/Restart 获得单调递增 activation sequence；最后激活的 Replace 成为 base，并截断更早 layer 的 Add/Accumulate，Replace 同层以及此后激活 layer 的 Add/Accumulate 按稳定 contributor 顺序叠加。不存在 Replace 时使用最早 activation layer 的 baseline。
- descriptor 未声明支持的 composition 必须在 compile 阶段报错。
- revert 必须恢复 composition 发生前的 baseline，而不是某个 tween 的 from value。

## Sampling 与 dirty batch

- instance 注册必须同时提交 typed baseline snapshot；baseline 数量和 value kind 必须与 compiled outputs 一致，否则返回 typed runtime error。一个 target/property 无论横跨多少 TimeDomain 都只读取和保存一份 baseline。
- 每个 domain 先通过 `TimeDomainMapper` 得到 local time、iteration 和 direction，再移动对应 track cursor；正向连续帧使用 cursor advance，反向或跳转使用 seek。
- tween 固定按 easing → interpolation → modifier 采样；Replace 先确定 base，Add 叠加当前 contribution，Accumulate 再叠加已完成 iteration 的 terminal delta。
- arithmetic composition 当前由 core 对 Scalar、Length、Angle、Color、Vec2、Vec3 提供 typed 运算；其他 value kind 必须由 property contract 拒绝或 adapter custom compositor 处理，禁止退化为字符串/f32。
- dirty compare 必须使用 property descriptor precision；未变化值不得进入 FrameBatch。
- Engine 在 instance 注册冷路径把 output 映射为 adapter-global `EngineOutputId`，并维护 target、invalidation class、property 的稳定 apply 顺序；FrameBatch 只按该顺序 append，正常帧禁止排序、去重和字符串 lookup。
- FrameBatch identity 必须使用 adapter-global `(AdapterId, AdapterTargetId, AdapterPropertyId)`，禁止把不同 plan 的 local TargetId/PropertyId 直接比较。
- 同一 global property 的并发输出由 global slot 完成 Replace/Add/Accumulate layer fold；`InstanceId` 是 activation sequence 相同时的确定性 tie-breaker，每个 global slot 每帧至多产生一次写入。
- 首次注册 global property 时固化完整 `PropertyDescriptor` contract；后续 plan 对同一 adapter/target/property 提交不同 value kind、unit、interpolation、composition、precision、invalidation 或 native contract 必须返回 typed runtime error，禁止仅依赖 debug assertion 或在 compose 时静默降级。

## Resolution snapshot

- Resolver 只消费 adapter 提供的只读 resolution snapshot；target discovery、property schema、window metrics 和 layout snapshot 不得进入 frame loop。
- 同一次 resolution 中 `(AdapterId, AdapterTargetId)` 与 `(AdapterId, AdapterPropertyId)` 必须 intern 为唯一 dense slot；重复 target 或冲突 property schema 返回 typed error。
- baseline 每个 target/property 最多读取一次，并同时服务 `Current`、无前驱的 `Previous` 和 `RelativeBaseline`。
- function value 在 resolve/start/refresh 阶段按稳定 target order 执行，`TargetContext` 必须包含 index、total、target name、layout snapshot 和 window metrics；默认禁止每帧执行。
- function、relative 和 unit resolution 的结果必须在进入 Compiler 前成为 owned `AnimationValue`，并立即按 property descriptor 验证。

## Target 生命周期

- play 时 target 未挂载：实例进入 Scheduled；target 可用后自动开始，不要求业务重复调用 play。
- target 播放中卸载：默认从实例 target set 移除；最后一个 target 消失时实例 cancel。
- required target policy 可以把任一 target 丢失升级为整个实例 cancel。
- duplicate target name 是 resolve error，禁止静默选择其中一个。

## Backend 一致性

- backend selection 不得改变上述状态、controls、callback、baseline 和 composition 语义。
- backend 缺少所需 control/callback 时不得被选中。
- `NativeOnly` 无法表达时构建失败；其他策略回退必须产生 `LoweringReport`。
- native completion/cancel 必须由真实 native callback 驱动，禁止 fixed timeout。
