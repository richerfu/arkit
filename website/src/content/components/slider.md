---
title: Slider
description: "拖动选值，也支持范围和多个拇指。"
---

# Slider

拖动选值；也可以做成范围或多个拇指。`Slider` 在拖动中通过 `on_change` 返回实时值，手指抬起后通过 `on_change_end` 返回一次最终值；需要异步提交的场景应在后者发起请求，避免拖动期间重复提交。
