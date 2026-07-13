# Animation v2 验证记录

> 日期：2026-07-13  
> 结论：终态能力代码、workspace cutover、文档和三套 OpenHarmony 功能验收通过。没有新增独立 benchmark feature/page；量化 p95 与长稳时长保留为发版参考设备验收项。

## 自动化验证

| 检查 | 结果 |
| --- | --- |
| `cargo test -p arkit_animation_core` | PASS，57 passed |
| 1,000 次 instance insert/remove dense storage reuse | PASS，live instance 回到 0，instance ID 和 output slot 复用 |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo metadata --format-version 1 --no-deps` | PASS |
| `pnpm run docs:build` | PASS |
| v1 public symbol 搜索 | 生产代码 0 命中；仅设计/迁移文档命中 |
| Chart 私有 clock 搜索 | `arkit_chart` 中无 `SystemTime`、`Instant`、`tokio::time::sleep` 或 `use_future` 驱动动画 |

`arkit_animation` 的 host test binary 在 macOS 不能链接 OHOS `native_*` 系统库，这是目标库的 host linker 限制；测试模块由 workspace `--all-targets` Clippy 编译，纯算法和状态机在 `arkit_animation_core` 执行，平台链接由以下 OHOS build 覆盖。

## OpenHarmony 构建

以下命令全部通过并生成 arm64 NAPI library：

```sh
cd examples/animation && ohrs build --arch aarch
cd examples/shadcn_showcase && ohrs build --arch aarch
cd examples/chart && ohrs build --arch aarch
```

三个 example 随后分别通过 `app/run.sh` 同步到 HAP 壳并完成 `hvigorw assembleHap`。脚本内 piped `hdc install` 在本机偶发 `Connect server failed`，同一产物使用直接 `hdc install -r` 均成功，`aa start` 均成功；这属于 HDC server 瞬态，不是应用构建或启动失败。

## 真机/模拟器验收

目标：`127.0.0.1:5555 TCP Connected localhost`，分辨率 1320 × 2856。

| Example | 验收结果 |
| --- | --- |
| `animation` | 页面完整显示；SlideUp 初始动画完成；Fade preset 点击后 label 切换；Replay 可触发；节点 opacity/transform 由统一 Engine 写入 |
| `shadcn_showcase` | 首页、搜索框、组件列表和主题按钮完整显示，无空白/崩溃 |
| `chart` | line/bar/scatter Canvas 正常绘制；live tick 自动推进；transition 和 effect phase 由 Engine-driven Animatable 提供 |

## 结构审计

- UI root 自动安装一个 `AnimationHost`/`AnimationEngine`/`FrameDriver`。
- `use_ark_node` 同 scope 多次调用共享 resolver slot；这修复了多个 animation hook 同组件时后注册 hook 永远拿不到 native node 的问题。
- steady-state Engine tick 使用 dense ID、预编译 plan、复用 scratch/batch，不进行 target/property 字符串 lookup 或排序。
- adapter frame 在完整预校验后按 adapter 连续批量提交；只有 acknowledge 成功后发布 render/call/terminal/settled。
- instance、adapter、target 和 Engine output hole 会复用；target/adapter detach 会取消受影响实例并释放 binding。
- Presence leaving 节点等待真实 terminal settlement；Layout shared projection 在 settle/drop 清理；Gesture/Scroll 无私有 timer 或 frame loop。

## Backend 边界

capability lowerer 和真实 `ArkUiImplicitInstance`、`ArkUiKeyframeInstance`、`ArkUiAnimatorInstance` 已落地。当前通用 `#[entry]` 使用的 `@ohos-rs/ability` `DefaultXComponent.render(helper, NodeContent)` 不传 `ArkUIContext`，因此 hook host 选择 native backend 后会记录 `BackendUnavailable` 并回退 sampled；`NativeOnly` 返回 typed error，不静默降级。持有 `ArkUIContext` 的平台集成层可以直接创建这些 native owner。

## 未伪造的发版数据

`AnimationPerformanceCounters` 已提供 frame、dirty write、adapter failure、target miss、fallback、compute/apply 纳秒和 Engine live/active/output/command 计数。100/500 target p95、30 分钟 infinite retained heap 和 10 分钟掉帧属于参考设备量化发版门槛；本轮按要求没有为它们增加独立 benchmark 页面或 feature，也没有用 host 数字冒充设备数据。
