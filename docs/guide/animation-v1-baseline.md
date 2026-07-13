# Animation v1 基线

> 日期：2026-07-13  
> 用途：Animation v2 行为、规模、性能和删除验收的对照基线。

## 源码基线

| 指标 | 当前值 |
| --- | ---: |
| 核心 Rust 源码 | 2,850 行 |
| `lib.rs` | 754 行 |
| `timeline.rs` | 1,067 行 |
| `group.rs` | 865 行 |
| `stagger.rs` | 164 行 |
| 单元测试 | 15 个 |

## 当前公共模型

- 两套播放器：`Player` 与 `GroupPlayer`。
- 固定完整状态：`AnimationState`。
- 固定相对增量：`AnimationDelta`。
- 单 target `Timeline` 与多 target `TimelineGroup`。
- target identity 使用 `String`。
- `TimelineGroup::sample_targets` 在每帧构造 `BTreeMap<&str, Vec<&TimelineTrack>>`、排序并产生 owned `String`。
- `apply_state` 每帧始终写 opacity、translate、scale、rotate，并按 Option 写 color/radius/blur/size。
- 每个 timeline/player 各自注册 ArkUI one-shot frame callback。
- frame callback 忽略平台 timestamp，使用 `Instant::now()`。
- native `Motion` 使用 `Animation::animate_to`，但 progress/is_running 是同步粗粒度状态，stop 不取消 native execution。

## 验证基线

```text
cargo clippy -p arkit_animation --all-targets -- -D warnings: PASS
cargo fmt --all -- --check: PASS
cargo test -p arkit_animation: HOST LINK BLOCKED
```

host test 的阻塞是 macOS 缺少 `ohresmgr` 等 HarmonyOS native 库。v2 的 pure core 必须消除这一阻塞并在 host 直接执行测试。

## 性能基线策略

- 当前仓库没有 animation engine 帧耗时、allocation、dirty write 计数器，因此不能伪造 1/10/100/500 target 数值。
- 实施优先完成能力模型、Compiler、Engine、Adapter 和高级能力对齐；benchmark 不作为前置 feature 或独立页面进入 animation example。
- 设备性能数据统一在 Phase 10 使用终态 instrumentation 采集，并与本节记录的 v1 热路径结构做对照。

## v2 删除验收

cutover 后以下生产代码搜索必须为零：

```sh
rg -n "AnimationState|AnimationDelta|TimelineGroup|use_timeline_group|GroupPlayer|RETAINED_ANIMATIONS" crates examples
```

文档中的历史说明可以保留，但必须明确标记 v1。
