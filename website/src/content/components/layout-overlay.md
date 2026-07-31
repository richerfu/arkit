---
title: 布局与浮层
description: "测量尺寸、处理安全区与键盘，并用 Portal 投影浮层。"
---

# 布局与浮层

量尺寸、躲安全区、听键盘，以及把浮层投影到 root——这几件事经常一起出现。

## 尺寸与对齐

- `width: "100%"` 表示填满父宽。
- 固定 width/height 默认使用 vp。
- Text 换行前必须有可用宽度。
- Row/Column 的默认对齐不替代组件显式布局，页面自定义 children 需要时设置 `align_items`。

## Portal

Dialog、Popover、Select、Menu 和 Sonner 都使用 renderer 原生支持的声明式 `Portal`，业务不需要创建 host、注册 overlay service 或发布命令式 token。

```text
声明位置（状态 / Context）── HostTree ── root native projection
```

浮层仍能读取声明位置的 Theme、Signal 和其他 Context。scope 卸载会产生普通 Dioxus mutation 并清理 projection。

## SafeArea 与键盘

Modal panel 使用 safe viewport；backdrop 可以覆盖完整 surface。BottomSheet、Drawer 和 Sonner 会避让安全边距。包含输入框的底部面板还要考虑 IME area，长内容放 ScrollArea。

## 锚点定位

Popover、Select、DropdownMenu 等先通过 layout hook 测量 trigger，再结合 `FloatingSide`、`FloatingAlign` 和 viewport 选择位置。旋转、分屏或窗口 resize 后必须重新计算，业务不要缓存绝对坐标。

## 层级原则

不要在同一 scope 同时声明相互竞争的 modal。确认弹窗打开前先关闭无关 menu，避免多个 backdrop 捕获同一事件。
