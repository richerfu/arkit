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
| `arkit_chart` | ECharts-like option、原生 Custom canvas 绘制、图表命中与 tooltip |
| `arkit_router` | dioxus-router 的 ArkUI 集成 |
| `arkit_i18n` | locale context 与类型安全消息 |
| `arkit_shadcn` | Dioxus 组件与 theme context |

## 渲染规则

Dioxus logical tree 是真实来源。Text child、placeholder、template path 和 ElementId 都保留在 HostTree；ArkUI tree 只是投影。例如 `text { "count = {count}" }` 的逻辑 text child 会更新父 ArkUI Text 的 `TextContent`，不会生成错误的 nested Text。

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

Gesture recognizer、callback context 和目标 native wrapper 由对应 HostNode 持有。listener 被移除、native wrapper 被替换或 subtree 被 dispose 时，renderer 必须先移除并释放 recognizer，再释放 callback context，禁止把裸 context 指针留给 ArkUI。

## Overlay

`OverlayRoot` 是应用唯一浮层出口。浮层内容仍属于同一 Dioxus tree，不创建第二个 VirtualDom。受控菜单在保持打开时必须把最新 props 重新发布到同一个 overlay subtree；关闭再打开才能看到新状态属于 stale `Element` snapshot bug。重新发布时必须保留 overlay-local hook state，例如已展开的 submenu path。

## Window metrics 与安全区

`arkit_runtime` 是窗口几何的唯一来源。它把 OpenHarmony 的 `avoidAreaChange`、`windowRectChange`、surface resize 和 keyboard 事件归一成 `WindowMetrics`，并在事件变化时使 Dioxus tree 重新渲染。avoid-area rectangle 必须先与 XComponent `content_rect` 求交集，再从物理像素转换为 vp；这样非全屏宿主已经完成的系统栏避让不会被重复应用。

普通 `#[entry]` 默认使用 `SafeAreaPolicy::Safe`。框架 root 保持一个覆盖完整 XComponent 的 window stack，业务 subtree 位于 safe content viewport，`OverlayRoot` 与它并列：backdrop 可以覆盖整个窗口，而 Dialog、Sheet、Menu、Popover 等面板被约束在 safe viewport 内。沉浸式页面可使用 `#[entry(edge_to_edge)]`，此时业务 subtree 填满 XComponent，但 `use_window_metrics()`、`use_safe_area()` 以及框架浮层避让仍然有效。宿主还需配置对应的 window edge-to-edge 模式。

System、Cutout 与 NavigationIndicator 合并为 visual safe area；SystemGesture 和 IME 保持独立。键盘高度不能作为全局页面 padding，滚动容器和输入法协调逻辑应单独消费 `ime_area` / `keyboard_height`。

## Escape hatch

业务 UI 默认写 `rsx!`。只有 ArkUI 无法声明式表达的能力才使用 `arkit_hooks` 获取 native node，例如布局观测、NodeAdapter 虚拟化、动画、原生图表或嵌入 WebView。`arkit_chart` 把 Custom canvas escape hatch 封装在 `ECharts` 组件内，业务代码只传受控 props。
