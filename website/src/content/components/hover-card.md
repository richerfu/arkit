---
title: Hover Card
---

# Hover Card

HoverCard 在 pointer hover 时展示预览信息，适合用户、链接或对象摘要。

```rust
HoverCard {
    trigger: rsx! { Text { "@arkit" } },
    width: 320.0,
    UserPreview {}
}
```

Props 为 `trigger`、`open`、`default_open`、`on_close`、`on_open_change`、`width` 与 `children`，状态模型与 Popover 相同。

触屏设备没有稳定 hover。关键内容和唯一操作不能只放在 HoverCard 中，应提供点击进入详情或其他可发现路径。
