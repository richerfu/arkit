---
title: 事件与 Action
description: "用 Controller 做选择、缩放，或发命令式操作。"
---

# 事件与 Action

`ChartController` 提供命令式操作和查询。命令不会跨挂载边界排队：未绑定返回 `ChartError::NotBound`，native canvas 尚未完成首次有效绘制返回 `NotReady`。一个 controller 同时只属于一张 chart；重复绑定以 `AlreadyBound` 拒绝，不会静默改向另一张图。

## 安装 Controller

```rust
let controller = use_hook(ChartController::new);

rsx! {
    ECharts {
        option,
        controller: controller.clone(),
        on_select: move |event| handle_select(event),
        on_event: move |event| handle_runtime_event(event),
    }
}
```

## Controller API

| API                     | 说明                                                        |
| ----------------------- | ----------------------------------------------------------- |
| `dispatch_action(s)`    | 返回 `Result<ChartCommandStatus, ChartError>`                |
| `append_data`           | scatter/lines 增量数据；无效 index/类型返回明确错误          |
| `clear`                 | 清空当前 instance，返回是否应用                              |
| `get_option`            | 读取只读 `ChartSnapshot`（含 dataset/runtime selection/zoom）|
| `get_source_option`     | 读取当前 raw input DTO                                       |
| `get_size/width/height` | 有效绘制后的 `LogicalSizeVp`                                 |
| `convert_to/from_pixel` | 数据与 canvas-local `LocalVpPoint` 转换                      |
| `contain_pixel`         | 测试本地 vp 点是否在 grid/axis/series                        |
| `hit_test`              | 查询最近一次实际绘制产生的命中缓存                           |
| `is_bound/is_ready`     | 区分逻辑绑定和 native 首次有效绘制                           |

## Action 类型

`ChartActionKind` 支持 Highlight、Downplay、Select、Unselect、ToggleSelect、ShowTip、HideTip、LegendSelect/Unselect/Toggle、DataZoom、TimelineChange、TimelinePlayChange、Restore。

```rust
let status = controller.dispatch_action(ChartAction::new(
    ChartActionKind::ToggleSelect(
        ChartActionTarget::item(0, 3),
    ),
))?;
```

`silent()` 只抑制对应事件，不抑制状态变化。batch 完成全部 mutation 后产生 aggregate event。为保证 batch 不发生部分提交，`TimelineChange` 和 `Restore` 这类会改变后续 target 含义的结构操作必须单独 dispatch；把它们放进多 action batch 会返回 `UnsupportedOperation`。

## 事件

`on_select` 收到 `ChartEvent`，包含 component/series/data index、name、value 等。

`on_event` 收到 `ChartRuntimeEvent`，包括 event_type、pointer source、from_action、selected items、legend map 和 batch results。

pointer hit-test 和 `ChartController::hit_test(LocalVpPoint)` 都使用实际绘制结果缓存。旧的 `hit_test(&ChartOption, ...)` raw facade 已移除，因为它无法知道 dataset/media、legend 隐藏状态和当前动画绘制帧。离线 option 不是已挂载实例命中的替代品。

## 坐标转换

overlay annotation 可先 `convert_to_pixel` 得到 canvas-local `LocalVpPoint`，再通过显式 window-px/local-vp 换算与组件 layout frame 组合。窗口 resize/zoom 后重新转换，不长期缓存旧坐标。

## 状态同步

Action 修改 runtime selection/zoom。`get_option()?.option()` 是只读 runtime snapshot，不应重新作为输入；需要原始受控值时使用 `get_source_option()`。需要业务持久化时从 event/snapshot 同步到领域状态；不需要持久化则让 Chart instance 自己拥有，避免每次 pointer 都重建完整 option。
