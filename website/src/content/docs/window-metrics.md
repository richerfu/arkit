---
title: 窗口与尺寸
description: "读窗口尺寸、密度和方向，做响应式布局时怎么用。"
---

# 窗口与尺寸

窗口大小、像素密度、方向都会变。`use_window_metrics` 给你一份当前快照，布局和动画都可以据此响应。

## 读取 Metrics

```rust
let metrics = use_window_metrics();
```

| 字段              | 单位     | 含义                                          |
| ----------------- | -------- | --------------------------------------------- |
| `window_rect`     | 物理像素 | 主窗口矩形                                    |
| `content_rect`    | 物理像素 | 当前 XComponent surface                       |
| `scale`           | px/vp    | 物理像素与 ArkUI vp 比例                      |
| `safe_area`       | vp       | system、cutout、navigation indicator 合并边距 |
| `gesture_area`    | vp       | 系统手势区域                                  |
| `ime_area`        | vp       | 键盘 avoid area 与 content 的交集             |
| `keyboard_height` | vp       | 独立键盘高度 callback                         |

avoid-area 会先与 `content_rect` 求交，再换算为 vp。宿主已经避让系统栏时不会重复产生相同 padding。

## 响应式布局

```rust
let metrics = use_window_metrics();
let compact = metrics().content_rect.width / metrics().scale < 600.0;

rsx! {
    if compact {
        CompactLayout {}
    } else {
        TwoPaneLayout {}
    }
}
```

断点应基于有效内容尺寸，不根据设备型号。窗口可在旋转、分屏、折叠态和自由窗口中动态变化。

## 物理像素与 VP

native SDK 返回的像素坐标要先减去 content origin，再除以 scale 才能与 ArkUI layout 对齐。反向传入 pixel API 时执行相反换算。集中封装转换，避免某些页面漏掉 scale。

## 订阅边界

普通组件使用 `use_window_metrics`；非 Dioxus 服务可持有 `WindowMetricsHandle`/`WindowMetricsSubscription`。订阅必须随 owner 注销，callback 更新 UI 状态时通过当前 root 的 `RuntimeHandle::queue_ui`。

## 键盘

不要把 `keyboard_height` 直接加到整个 root。resize-mode window 可能已经缩小 content；输入页根据 `ime_area` 调整滚动区域或底部操作条，避免二次避让。
