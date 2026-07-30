---
title: Avatar
description: "圆形头像；图片还没好时显示 fallback。"
---

# Avatar

固定尺寸的头像。图片还在加载、或者干脆没有时，会显示 fallback。

## 用法

```rust
Avatar {
    src: "https://example.com/avatar.png",
    ring: true,
    fallback: rsx! { AvatarFallback { content: "AR" } },
}
```

## Props

| Prop       | 类型              | 说明                   |
| ---------- | ----------------- | ---------------------- |
| `src`      | `Option<String>`  | 图片地址               |
| `fallback` | `Option<Element>` | 图片下层占位内容       |
| `ring`     | `Option<bool>`    | 使用 background 色描边 |
| `radius`   | `Option<f32>`     | 默认 full radius       |

`AvatarFallback` 接收 `content: String`，用 muted surface 显示 initials。fallback 会先渲染，图片成功后覆盖；业务负责 URL 权限、缓存和隐私策略。
