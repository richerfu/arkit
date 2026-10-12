---
title: 页面转场
description: "用淡入淡出和弹簧曲线切页，让旧页面完成离场。"
---

# 页面转场

推荐在公共 layout 中使用 `AnimatedOutlet`。页面默认只改变透明度，不平移、缩放或旋转；旧页面先快速淡出，新页面稍后淡入，两页内容几乎不重叠。

## AnimatedOutlet

```rust
#[component]
fn AppShell() -> Element {
    rsx! { AnimatedOutlet::<Route> {} }
}
```

默认 preset 为 `Fade`，两条临界阻尼弹簧（透明度不回弹）：

|      | 曲线                                  | 节奏                                       |
| ---- | ------------------------------------- | ------------------------------------------ |
| 离场 | `from_response(0.2, 1.0)`             | 立即开始，约 225ms 消失                    |
| 入场 | `from_response(0.36, 1.0)`，延迟 40ms | 约 140ms 过半、265ms 到 90%，约 390ms 结束 |

两页同时可见的透明度峰值约 0.2，不会出现内容叠在一起的半透明画面。透明度最后 4% 肉眼不可见，弹簧在此处结束（`with_rest_thresholds`），不保留看不见的收敛尾巴。fill 为 true。页面内部不再套转场组件，避免重复淡入。

首次挂载直接显示，不播放入场动画。之后 push、back 和路由参数变化都按 route 字符串区分页面，在同一个全尺寸 stack 中淡出、淡入。离场页面不接收输入；快速 A→B→C 时，每个旧页面各自完成离场并清理，不依赖固定 timeout。

需要调整节奏时可覆盖 `duration_ms`（入场时长，离场不会超过它）、`delay_ms`（入场延迟）、`fill`，也保留显式 `preset` 覆盖；页面建议保持 `Fade`。设置 `duration_ms: 0` 可关闭动画。

```rust
AnimatedOutlet::<Route> {
    duration_ms: 300,
}
```

嵌套路由深度与 Dioxus `Outlet` 一致，但有一个限制：如果离场的 **layout 本身包含嵌套 Outlet**，它在淡出期间会显示路由器当前的子页面，因为上游 Outlet 始终读取实时路由。优先把 `AnimatedOutlet` 放在稳定的公共 layout 中，包住叶子页面。

## RouteTransition

```rust
RouteTransition::<Route> {
    column { "Home" }
}
```

它是仅入场的辅助组件，默认同样为 `Fade`、入场弹簧及其自然时长，没有延迟。调用方通过 key 控制重播；父组件卸载它时，无法保留旧页面或播放离场。需要完整切页转场时使用 `AnimatedOutlet`。

## 验证

`examples/router` 演示淡入淡出、动态参数、Link 和系统 back。真机检查首次显示、快速连续导航、返回到根、离场页面输入屏蔽与卸载清理。`examples/demos` 的 `regressions` 路由保留普通 Outlet，避免原生转场 wrapper 干扰 renderer-root 诊断。
