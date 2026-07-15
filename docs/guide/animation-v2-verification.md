# Animation v2 验证记录

> 日期：2026-07-13  
> 结论：终态能力代码、workspace cutover、文档和三套 OpenHarmony 功能验收通过。没有新增独立 benchmark feature/page；量化 p95 与长稳时长保留为发版参考设备验收项。

## 自动化验证

| 检查 | 结果 |
| --- | --- |
| `cargo test -p arkit_animation_core` | PASS，64 passed |
| 1,000 次 instance insert/remove dense storage reuse | PASS，live instance 回到 0，instance ID 和 output slot 复用 |
| `cargo check --workspace --all-targets --all-features` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo metadata --format-version 1 --no-deps` | PASS |
| derive / i18n proc-macro trybuild contracts | PASS，非法 entry、配置、locale 与 key 均产生稳定编译错误 |
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

本轮 workspace 架构收口后再次构建三套 arm64 library，并用最新 `libanimation.so` 重建 `entry-default-unsigned.hap`，均通过。app 自有的 target SDK 与 window-stage exception warning 已消除；剩余三条 ArkTS warning 来自 `@ohos-rs/ability` 包内未使用的 DefaultWebview 实现。

用户随后提供 `127.0.0.1:5555` 模拟器。最终 HAP 通过 `hdc install -r` 覆盖安装并由 `aa start` 启动；最终进程 PID 4985 在完整交互回归后仍存活，按该 PID 查询 ERROR/FATAL hilog 为 0 条。

## 真机/模拟器验收

目标：`127.0.0.1:5555 TCP Connected localhost`，分辨率 1320 × 2856。

| Example | 验收结果 |
| --- | --- |
| `animation` | 五个 lab 页面完整显示；Tab 切换通过 0ms 原生 scroll offset 明确回到各页顶部；Timeline 正向、反向、seek 和终态控制有可见变化；Easing 六条轨迹可见运行且初始状态为真实 `Idle`，Stagger 为确定的 4×3；Presence 的 leaving 节点与按钮不重叠；真实 drag 在 `(184,110)` 边界报告 `Settled`，scroll 100% 目标保持在舞台内；Scope 控制器初始解析完成并显示真实 `Idle`，反向/正向分别到达左右终点且不移动按钮热区。最终包额外执行 20 次 tab mount/unmount，进程存活且 ERROR/FATAL hilog 为 0。 |
| `shadcn_showcase` | 首页、搜索框、组件列表和主题按钮完整显示，无空白/崩溃 |
| `chart` | line/bar/scatter Canvas 正常绘制；live tick 自动推进；transition 和 effect phase 由 Engine-driven Animatable 提供 |

## 结构审计

- UI root 自动安装一个 `AnimationHost`/`AnimationEngine`/`FrameDriver`。
- `use_ark_node` 同 scope 多次调用共享 resolver slot；这修复了多个 animation hook 同组件时后注册 hook 永远拿不到 native node 的问题。
- target 注册会同步重试仍待解析的弱引用 controls，并在首次插入成功后立即发布初始 snapshot；跨组件 target 不再依赖父子 effect 的偶然执行顺序，状态、backend 和 lowering plan 也不会长期停留在假 `pending`。
- steady-state Engine tick 使用 dense ID、预编译 plan、复用 scratch/batch/host event buffer，不进行 target/property 字符串 lookup 或排序；native frame callback 直接把 command 写入 Engine queue，不再构造中间 `Vec`。
- adapter frame 在完整预校验后按 adapter 连续批量提交；只有 acknowledge 成功后发布 render/call/terminal/settled。
- ArkUI adapter 在 lowering 边界把 arithmetic composition 产生的 opacity overshoot 钳制到平台 `[0, 1]` 域，并在 native write 失败时保留底层错误文本；示例 additive opacity 也预留了合法 headroom。
- instance、adapter、target 和 Engine output hole 会复用；target/adapter detach 会取消受影响实例并释放 binding。
- Presence leaving 节点等待真实 terminal settlement；Layout shared projection 在 settle/drop 清理；Gesture/Scroll 无私有 timer 或 frame loop。
- dynamic native fallback 与 hot timeline replacement 会更新 controls 查询到的 `LoweringReport`，不会留下过期 backend。
- layout observer 先合并订阅需求，每次 dispatch 最多查询一次 size、仅在需要 frame 时查询 position，并用 inline `SmallVec` 暂存常见通知，回调执行前释放 hub borrow。
- renderer 的结构同步支持真实 reorder；native remove/insert 失败会中止本轮同步，不会把 logical host 误绑定到目标 index 上已有的 sibling。renderer teardown 会先注销 listener token 和 gesture，再释放 native subtree，`Drop` 兜底覆盖提前返回路径。
- NodeBuilder、VirtualNodeAdapter 与 Embedded WebView 的失败路径会显式释放 native owner；adapter reset 或 WebView attach 失败不会把 handle 从 Rust state 中丢失后泄漏。
- overlay scope unmount 会按 checked token 主动 dismiss 并使遗留 handle 失效，因此 dismiss closure 反向持有 `OverlayApi` 时也不会形成永久引用环。
- Easing lane 明确使用 start cross-axis alignment，并为 spring/Bézier overshoot 预留轨道余量；目标不会再从默认中心出发或越过卡片边界，避免状态已完成但视觉上像未执行。
- renderer 暴露了类型正确的 `scroll_offset` 混合参数编码（vp 浮点 offset + 整数 duration/curve/options）；Demo 的 Tab 变化使用 0ms offset 归零，避免不同页面复用同一个原生 Scroll 偏移。
- Demo 将“改变 direction”和“开始播放”区分为明确的 `Reverse + play`，所有正向 replay 会先规范化 direction；Engine 的 `Complete` 按当前 direction 选择 0 或 duration 终点。
- `openharmony-ability` 固定到已升级 OHOS binding 的 `edc4e49d0d431035c6c001fc5e583abf62a998e3`，workspace 不再需要 type-only Cargo patch；HAP 依赖图只保留一代 ArkUI/XComponent/Display/resource-manager binding/sys stack。

## Backend 边界

capability lowerer 和真实 native owner 已落地。root host 通过 mounted node 的 `OH_ArkUI_GetContextByNode` 获取 `ArkUIContext`，`ArkUiAnimator` 作为外部 root clock 驱动同一个 compiled Engine，保持 per-property timing、composition、modifier、callback 和统一提交语义。创建失败或运行时 control 不能完整表达时，非 `NativeOnly` 实例原子切回 internal sampled clock，`LoweringReport` 同步改为 `Sampled` 并记录 control/reason；`NativeOnly` 返回 typed error，不静默降级。非运行态 reverse、reverse 后 replay 和 hot timeline replacement 被保守处理，避免 native 状态与 Engine snapshot 分叉。

## 未伪造的发版数据

`AnimationPerformanceCounters` 已提供 frame、dirty write、adapter failure、target miss、fallback、compute/apply 纳秒和 Engine live/active/output/command 计数。100/500 target p95、30 分钟 infinite retained heap 和 10 分钟掉帧属于参考设备量化发版门槛；本轮按要求没有为它们增加独立 benchmark 页面或 feature，也没有用 host 数字冒充设备数据。
