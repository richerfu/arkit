---
title: 事件与 Action
---

# 事件与 Action

`ChartController` 提供 imperative action 与查询。mount 前 action 会排队，bind 后按顺序执行；组件卸载后检查 `is_mounted`。

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

| API                     | 说明                                             |
| ----------------------- | ------------------------------------------------ |
| `dispatch_action(s)`    | 单个或 batch action                              |
| `append_data`           | scatter/lines 增量数据                           |
| `clear`                 | 清空当前 instance                                |
| `get_option`            | 读取含 runtime selection/zoom 的 resolved option |
| `get_size/width/height` | logical canvas 尺寸                              |
| `convert_to/from_pixel` | 数据与 canvas pixel 转换                         |
| `contain_pixel`         | 测试点是否在 grid/axis/series                    |
| `is_mounted`            | 是否已绑定组件                                   |

## Action 类型

`ChartActionKind` 支持 Highlight、Downplay、Select、Unselect、ToggleSelect、ShowTip、HideTip、LegendSelect/Unselect/Toggle、DataZoom、TimelineChange、TimelinePlayChange、Restore。

```rust
controller.dispatch_action(ChartAction::new(
    ChartActionKind::ToggleSelect(
        ChartActionTarget::item(0, 3),
    ),
));
```

`silent()` 只抑制对应事件，不抑制状态变化。batch 完成全部 mutation 后产生 aggregate event。

## 事件

`on_select` 收到 `ChartEvent`，包含 component/series/data index、name、value 等。

`on_event` 收到 `ChartRuntimeEvent`，包括 event_type、pointer source、from_action、selected items、legend map 和 batch results。

pointer hit-test 使用实际绘制结果缓存。业务回调不要按理想几何重新算一次命中。

## 坐标转换

overlay annotation 可先 `convert_to_pixel` 得到 canvas logical point，再结合组件 layout frame 定位。窗口 resize/zoom 后重新转换，不长期缓存旧 pixel。

## 状态同步

Action 修改 runtime selection/zoom。需要业务持久化时从 event/get_option 同步到领域状态；不需要持久化则让 Chart instance 自己拥有，避免每次 pointer 都重建完整 option。
