---
title: 交互与可访问性
description: "触控、焦点、反馈和移动端适配。"
---

# 交互与可访问性

组件默认面向触屏 ArkUI，但业务仍需提供清楚的文本、状态和反馈。颜色、图标或位置不能成为唯一语义来源。

## 触控目标

按钮、菜单 action、Toast 关闭等主要触控目标保持至少约 40–48vp。小图标可以视觉上是 16vp，但外层 hit area 必须更大。

## 文本与状态

- IconButton 同时提供上下文可理解的 label/说明。
- FieldError 使用文字显示错误，不只改变红色边框。
- Slider 附近显示当前值和单位。
- Loading、Empty、Error 分开建模。
- destructive action 在菜单样式之外还需要明确文案和确认流程。

## 焦点与键盘

输入组件使用 ArkUI 原生焦点。Modal 打开后应把操作保持在面板语义范围，关闭后恢复到合理 trigger。Menubar、Command 等键盘行为需要在目标 OpenHarmony 设备上验证。

## Hover 降级

触屏没有稳定 hover。HoverCard 和 Tooltip 的关键信息必须能通过点击、长按或页面正文获得；提交、删除等关键操作不能只放在 hover 内容中。

## 动效

尊重减少动态效果设置：减少 Carousel、Dialog、Progress 等 transition，避免无限装饰动画。Spinner 仅在任务确实运行时 spinning。

## 测试清单

使用大字体、窄窗口、横屏、键盘弹出、深浅主题和系统返回键检查组件。真机事件与 ArkUI layout 是最终标准，桌面截图不能替代。
