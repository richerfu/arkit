---
title: 后端与性能
description: "采样后端和原生 lowering 怎么选，失败时如何回退和排查。"
---

# 后端与性能

动画可以走 root 引擎逐帧采样，也可以在语义完整时降到 ArkUI native。怎么选取决于「能不能保住契约」，而不是「尽量原生」。

## Execution Policy

| Policy            | 行为                                  |
| ----------------- | ------------------------------------- |
| `Auto`            | 可完整保持语义时 native，否则 sampled |
| `SampledOnly`     | 强制 root Engine 每帧采样             |
| `NativePreferred` | 优先 native，fallback 写入 report     |
| `NativeOnly`      | 任一能力不足即 typed error            |

## Lowering 检查

lowerer 检查 seek、pause/resume、reverse、cancel、alternate、callbacks、property timing、composition、modifier、infinite iteration、layout invalidation、custom easing 和 property support。

```rust
let report = controls.lowering_report();
for rejection in &report.rejections {
    tracing::debug!(?rejection, "native lowering rejected");
}
```

`LoweringReport` 包含 backend、拒绝原因、target/property/tween 数与估算工作量。不要把 fallback 当错误吞掉；它是性能诊断的重要输入。

## 运行时失败

native handle 创建失败，或运行时 control 无法保持原契约时，`Auto`/`NativePreferred` 一次性切到 sampled clock；`NativeOnly` 返回错误。框架不会报告成功却悄悄忽略命令。

## 帧提交

同一帧按 sample → compose → adapter commit 批量提交 dirty property。普通帧不触发 VirtualDom render；Drawing、Chart 和 ArkUI adapter 共用 root tick。

## 优化顺序

1. 减少同时 active 的 target/property 数。
2. 避免动画布局属性导致大 subtree 反复 measure，能用 transform 时优先 transform。
3. 不订阅无须展示的 progress snapshot。
4. 检查 native lowering report，而不是盲目切 `NativeOnly`。
5. 用 `AnimationPerformanceCounters` 对比 sample、compose、commit 成本。

## 验证

`examples/animation` 包含 easing、timeline、interaction、lifecycle 和 orchestration labs。真机覆盖 pause/resume、reverse、seek、cancel/revert、循环 callback、layout/presence、drag/scroll 和 fallback report。
