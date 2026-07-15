---
title: 布局与浮层
---

# 布局与浮层

组件使用 ArkUI vp 布局。父容器必须提供可确定尺寸；百分比只相对于已确定的 parent content box。

## 尺寸与对齐

- `percent_width: 1.0` 表示填满父宽。
- 固定 width/height 默认使用 vp。
- Text 换行前必须有可用宽度。
- Row/Column 的默认对齐不替代组件显式布局，页面自定义 children 需要时设置 `align_items`。

## OverlayRoot

`#[entry]` 已安装唯一 OverlayRoot。Dialog、Popover、Select、Menu 和 Sonner 都复用它，不需要业务再次创建。

```text
页面 subtree ─┐
              ├─ 同一个 VirtualDom / Context
OverlayRoot ──┘
```

浮层发布后仍能读取 Theme、Signal 和其他 Context。scope 卸载会撤销所属 token。

## SafeArea 与键盘

Modal panel 使用 safe viewport；backdrop 可以覆盖完整 surface。BottomSheet、Drawer 和 Sonner 会避让安全边距。包含输入框的底部面板还要考虑 IME area，长内容放 ScrollArea。

## 锚点定位

Popover、Select、DropdownMenu 等先通过 layout hook 测量 trigger，再结合 `FloatingSide`、`FloatingAlign` 和 viewport 选择位置。旋转、分屏或窗口 resize 后必须重新计算，业务不要缓存绝对坐标。

## 层级原则

不要在同一 scope 同时发布相互竞争的 modal。子菜单由父菜单 session 管理；确认弹窗打开前先关闭无关 menu，避免多个 backdrop 捕获同一事件。
