---
title: Timeline
description: "按时间顺序展示事件或里程碑，带完成态指示。"
---

# Timeline

一条竖向或横向的事件轴，用 `orientation` 切换方向，用 `align` 切换内容相对轨道的位置。完成项圆点描 primary；通往下一个完成节点的连线用实色。

ArkUI 没有 CSS 绝对定位，所以 `TimelineItem` 自己排轨道。

## 用法

```rust
use arkit::shadcn::components::{Timeline, TimelineItem};

Timeline {
    default_value: Some(3),
    TimelineItem {
        step: 1,
        date: Some("Mar 15, 2024".into()),
        title: Some("Project Kickoff".into()),
        description: Some("Initial team meeting and project scope.".into()),
    }
    TimelineItem {
        step: 2,
        date: Some("Mar 22, 2024".into()),
        title: Some("Design Phase".into()),
        description: Some("Wireframes and stakeholder review.".into()),
    }
    TimelineItem {
        step: 3,
        last: true,
        date: Some("Apr 5, 2024".into()),
        title: Some("Development Sprint".into()),
        description: Some("API and frontend work.".into()),
    }
}
```

`step <= value` 的节点视为完成。`value: Some(0)` 全部未完成；`value` 大于最大 `step` 则全部完成。最后一项要设 `last: true`，否则会多出一段连接线。

## 自定义内容

日期 / 标题 / 描述只是快捷 props。需要更复杂的内容时，用 Header / Date / Title / Content：

```rust
use arkit::shadcn::components::{
    Timeline, TimelineContent, TimelineDate, TimelineHeader, TimelineItem, TimelineTitle,
};

TimelineItem {
    step: 1,
    icon: Some("rocket".into()),
    TimelineHeader {
        TimelineDate { content: "Mar 15, 2024" }
        TimelineTitle { content: "Project Kickoff" }
    }
    TimelineContent { content: "Initial team meeting.".to_string() }
}
```

`icon` 是 Lucide 图标名，画在 16vp 圆点里。`indicator` 可以换成头像或其他节点；此时用 `indicator_size` 放大圆点。

## 方向

```rust
use arkit::shadcn::components::TimelineOrientation;

Timeline {
    orientation: TimelineOrientation::Vertical, // 默认
    // ...
}

Timeline {
    orientation: TimelineOrientation::Horizontal,
    item_min_width: Some(148.0),
    // ...
}
```

| 方向         | 默认轨道        | 条目宽度                                           |
| ------------ | --------------- | -------------------------------------------------- |
| `Vertical`   | 竖线 + 内容在右 | 始终 `100%`                                        |
| `Horizontal` | 横线 + 内容在下 | 默认均分容器；设 `item_min_width` 后定宽并横向滚动 |

横向不设 `item_min_width` 时条目 `flex-1` 均分宽度，适合 3～4 个短标签。条目一多或标题较长时传入 `item_min_width`，轨道在容器内横向滚动，连线连续穿过条目之间，不会被 padding 掐断。

## 对齐

```rust
use arkit::shadcn::components::TimelineAlign;

Timeline {
    align: TimelineAlign::Right, // 默认：内容在轴的右侧 / 下方
    // ...
}

Timeline {
    align: TimelineAlign::Left, // 内容在轴的左侧 / 上方，纵向文字右对齐
    // ...
}

Timeline {
    align: TimelineAlign::Alternate, // 轴居中，按 step 奇偶左右（或上下）交错
    // ...
}
```

| `align`     | 纵向                             | 横向                       |
| ----------- | -------------------------------- | -------------------------- |
| `Right`     | 轴在左，内容在右                 | 轴在上，内容在下           |
| `Left`      | 轴在右，内容在左，文案尾对齐     | 轴在下，内容在上           |
| `Alternate` | 轴居中；奇数 step 在右、偶数在左 | 轴居中；奇数在下、偶数在上 |

交错布局两侧等宽（`1fr | 轴 | 1fr`），轨道始终在容器中线，不会因为某一侧的间距把轴挤偏。纵向连线按条目内容高度绘制，从圆点中心接到下一项；横向连线铺满条目宽度，圆点压在线上。左侧内容使用 `text_align: end`。

## 受控

```rust
Timeline {
    value: Some(step()),
    orientation: TimelineOrientation::Horizontal,
    interactive: true,
    on_value_change: move |next| step.set(next),
    // ...
}
```

`value: Some(_)` 为受控；省略则读 `default_value`（默认 `1`）。`interactive` 时点击条目会选中该 `step`：非受控直接改内部状态，受控只发 `on_value_change`。

## Compound API

| 组件                | 作用                                      |
| ------------------- | ----------------------------------------- |
| `Timeline`          | 容器，提供 active step 与方向             |
| `TimelineItem`      | 一条事件，自带 indicator / separator 轨道 |
| `TimelineHeader`    | 日期和标题的堆叠                          |
| `TimelineDate`      | 小字号 muted 时间                         |
| `TimelineTitle`     | `text-sm` medium 标题                     |
| `TimelineContent`   | muted 说明                                |
| `TimelineIndicator` | 圆点；完成态 primary 边框                 |
| `TimelineSeparator` | 连接线；通往下一个完成节点时为 primary    |

## Props

### Timeline

| Prop              | 默认       | 说明                                             |
| ----------------- | ---------- | ------------------------------------------------ |
| `value`           | 非受控     | 当前完成到第几步；`Some` 即为受控                |
| `default_value`   | `1`        | 非受控初始步                                     |
| `orientation`     | `Vertical` | `Vertical` 或 `Horizontal`                       |
| `align`           | `Right`    | `Right`、`Left` 或 `Alternate`                   |
| `item_min_width`  | —          | 横向条目最小宽度（vp）；设置后横向滚动。纵向忽略 |
| `interactive`     | `false`    | 点击条目选中该 step                              |
| `on_value_change` | —          | 步数变化                                         |
| `children`        | 必填       | `TimelineItem` 列表                              |

### TimelineItem

| Prop             | 默认    | 说明                          |
| ---------------- | ------- | ----------------------------- |
| `step`           | 必填    | 与 `value` 比较以决定完成态   |
| `last`           | false   | 最后一项，隐藏 separator      |
| `completed`      | 按 step | 覆盖 step 比较                |
| `date`           | —       | 快捷日期                      |
| `title`          | —       | 快捷标题                      |
| `description`    | —       | 快捷说明                      |
| `icon`           | —       | Lucide 图标名                 |
| `indicator`      | —       | 自定义圆点内部节点            |
| `indicator_size` | 16      | 圆点边长，单位 vp             |
| `children`       | —       | Header / Content 等自定义内容 |

连接线颜色跟的是「下一个 step 是否完成」（`step < value`），不是当前点是否完成。点本身在 `step <= value` 时描 primary。
