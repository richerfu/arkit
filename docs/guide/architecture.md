# 架构

Arkit 的唯一 UI 运行模型是 Dioxus。

```text
#[entry] root component
        │
        ▼
Dioxus VirtualDom + scheduler
        │ WriteMutations
        ▼
arkit_arkui HostTree
        │ deterministic projection
        ▼
ArkUI native nodes
```

## 所有权

| Crate | 责任 |
| --- | --- |
| `arkit` | facade、prelude、入口 root wrapper |
| `arkit_runtime` | VirtualDom、scheduler/OpenHarmony event loop、原生事件队列与 Dioxus event dispatch |
| `arkit_arkui` | HostTree、projection、attributes、native node event/gesture、image、virtual adapter |
| `arkit_elements` | `rsx!` 使用的 ArkUI element/attribute/event registry |
| `arkit_hooks` | native node、layout、overlay、virtual list hooks |
| `arkit_animation_core` | 无平台依赖的 resolve/compile/sample/state engine |
| `arkit_animation` | root frame driver、ArkUI/Drawing adapter、native lowering、layout/presence/drag/scroll |
| `arkit_chart` | ECharts-like option、原生 Custom canvas 绘制、图表命中与 tooltip |
| `arkit_router` | dioxus-router 的 ArkUI 集成 |
| `arkit_i18n` | locale context 与类型安全消息 |
| `arkit_icon` | 内嵌 SVG catalog 与有界的 UI-thread render cache |
| `arkit_shadcn` | Dioxus 组件与 theme context |

`arkit` facade 默认只包含 renderer/runtime/core hooks。`animation`、`chart`、`i18n`、`icon`、`router`、`shadcn` 是显式 feature；`full` 才启用全部领域 crate。这样基础应用不会因为 facade glob 自动链接完整组件和图表栈。

## 渲染规则

Dioxus logical tree 是真实来源。Text child、placeholder、template path 和 ElementId 都保留在 HostTree；ArkUI tree 只是投影。例如 `text { "count = {count}" }` 的逻辑 text child 会更新父 ArkUI Text 的 `TextContent`，不会生成错误的 nested Text。结构更新按最终 HostTree 同步：已有 native child 的 reorder 会先 detach 再插入目标位置，不会通过重复 insert 留下 stale tail；任何 native insert 失败都禁止把 logical host 误绑定到该 index 上原有的 sibling wrapper。

## 调度规则

runtime 对 `VirtualDom::wait_for_work` 注册 OpenHarmony waker。signal、event、effect 或异步任务产生工作后，scheduler 唤醒 UI loop；UI loop 只渲染 ready work，然后重新挂起等待。不存在固定次数轮询。

原生输入必须经过明确的阶段边界：

```text
ArkUI node event / gesture
        │ owned payload
        ▼
RuntimeEventSink queue
        │ wake
        ▼
OpenHarmony UI loop
        │ Runtime::handle_event
        ▼
Dioxus scheduler → render_immediate → ArkUI projection
```

ArkUI callback 只复制 payload、入队并唤醒 UI loop。禁止在原生 callback 中直接借用 `VirtualDom`、调用 `Runtime::handle_event` 或启动 render；ArkUI 可能在 native tree patch 期间同步回调，这样做会造成 Dioxus render 重入。每个 UI tick 必须先执行已排队的 UI effect 和 native event，再渲染 scheduler ready work。

## Event 与 gesture

普通 click/change/scroll 等事件由 mounted native wrapper 的 node event 转发。`onlongpress` / `on_long_press` 使用 ArkUI 原生 LongPress Gesture，不允许降级为 click；默认语义为单指、500ms、非重复，并只在 gesture `Accept` 阶段派发一次。

event 名称的 alias、语义分类和 bubbling 规则只定义在 `arkit_elements::event`；runtime 和 ArkUI bridge 共同消费这一份 schema。Toggle change 读取 native bool，hover move 保留 pointer payload，Grid 不会映射到不相关的 WaterFlow event。高频 pointer move 在进入 Dioxus 前按 pointer/node 合并，但 down/up/cancel、click、change 等离散事件保持 FIFO。

Gesture recognizer、callback context 和目标 native wrapper 由对应 HostNode 持有。listener 被移除、native wrapper 被替换或 subtree 被 dispose 时，renderer 必须先移除并释放 recognizer，再释放 callback context，禁止把裸 context 指针留给 ArkUI。

renderer 的 host handle 使用可复用的 free-list arena；ElementId 到 host、native node 到 host、listener token 都有反向索引。subtree dispose 会释放 native listener/gesture、image、virtual adapter 和 arena slot，listener 的 active token 会拒绝已经注销但晚到的 native callback。

## Overlay

`OverlayRoot` 是应用唯一浮层出口。浮层内容仍属于同一 Dioxus tree，不创建第二个 VirtualDom。受控菜单在保持打开时必须把最新 props 重新发布到同一个 overlay subtree；关闭再打开才能看到新状态属于 stale `Element` snapshot bug。重新发布时必须保留 overlay-local hook state，例如已展开的 submenu path。

每次 `use_overlay` 拥有独立 checked token；scope unmount 会先移除该 token 对应的 entry，再释放 hook state。即使 overlay element 的 dismiss closure 反向持有 `OverlayApi`，scope cleanup 也会主动打破这个引用环；scope 结束后遗留的 handle 不能重新发布浮层。

## Window metrics 与安全区

`arkit_runtime` 是窗口几何的唯一来源。它把 OpenHarmony 的 `avoidAreaChange`、`windowRectChange`、surface resize 和 keyboard 事件归一成 `WindowMetrics`，并在事件变化时使 Dioxus tree 重新渲染。avoid-area rectangle 必须先与 XComponent `content_rect` 求交集，再从物理像素转换为 vp；这样非全屏宿主已经完成的系统栏避让不会被重复应用。

普通 `#[entry]` 默认使用 `SafeAreaPolicy::Safe`。框架 root 保持一个覆盖完整 XComponent 的 window stack，业务 subtree 位于 safe content viewport，`OverlayRoot` 与它并列：backdrop 可以覆盖整个窗口，而 Dialog、Sheet、Menu、Popover 等面板被约束在 safe viewport 内。沉浸式页面可使用 `#[entry(edge_to_edge)]`，此时业务 subtree 填满 XComponent，但 `use_window_metrics()`、`use_safe_area()` 以及框架浮层避让仍然有效。宿主还需配置对应的 window edge-to-edge 模式。

System、Cutout 与 NavigationIndicator 合并为 visual safe area；SystemGesture 和 IME 保持独立。键盘高度不能作为全局页面 padding，滚动容器和输入法协调逻辑应单独消费 `ime_area` / `keyboard_height`。

## Escape hatch

业务 UI 默认写 `rsx!`。只有 ArkUI 无法声明式表达的能力才使用 `arkit_hooks` 获取 native node，例如布局观测、NodeAdapter 虚拟化、动画、原生图表或嵌入 WebView。`arkit_chart` 把 Custom canvas escape hatch 封装在 `ECharts` 组件内，业务代码只传受控 props。

ArkUI binding 的 node/adapter wrapper 没有隐式 `Drop`，所以 escape hatch 必须显式转移和释放 native ownership。`NodeBuilder` 在 `build` 前持有 RAII cleanup guard；VirtualList adapter 的 attach/detach/reload 失败路径仍会释放 adapter 与已挂载 item；Embedded WebView 在 native node attach 失败时立即 dispose ArkTS controller，并且只在内容切换成功后提交 URL/HTML snapshot。错误返回不能遗留不可达 native handle。

## 热路径与缓存

动画帧使用 dense/generational ID、预编译 plan、复用 frame batch 和 dirty-write 提交；只有订阅 snapshot 的 scope 才响应式重渲染。Chart transition 共享 `Rc` snapshot，命中数据按绘制结果缓存，文字排版使用按 style 分桶的有界 cache，命中不分配 key。image、icon 与 typography cache 都有容量上限；UI-only native 对象使用 thread-local ownership，避免全局 mutex 和跨线程析构。

## Native 依赖边界

`openharmony-ability` 固定在提交 `edc4e49d0d431035c6c001fc5e583abf62a998e3`，该提交的 ArkUI 0.2、XComponent 0.3、Display 0.1 和 resource-manager 0.3 依赖范围与 workspace 一致。Arkit 不再维护本地 Cargo patch adapter；锁文件必须保持单一 binding/sys 版本，确保 HAP 内只有一套底层 native wrapper 和一致的 Rust 类型身份。

该提交尚未进入 `openharmony-ability` 的正式发布版本，因此完整 Git SHA 是当前 workspace/HAP 的集成边界。对外发布 Arkit 前必须升级到包含这些依赖范围和所需 runtime API 的正式版本；当前禁止把固定 Git 提交的本地构建通过等同于 crates.io 消费链路已通过。
