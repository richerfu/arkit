---
title: Sonner
description: "安全区感知的 Toast 队列。"
---

# Sonner

Sonner 是安全区域感知的全局 Toast 堆栈。调用方持有 `Vec<SonnerToast>`，每个 live toast 必须有唯一稳定 id。

完整 **Notification** 对齐官方 [Sonner](https://ui.shadcn.com/docs/components/base/sonner) 叠法：

- 所有卡片 **同源绝对定位**（bottom/top 边）
- 折叠：`translateY(±index × gap)` + `scale(1 − index × 0.05)`，后卡高度锁为前卡高度、内容隐藏，只露边
- 展开：同一坐标系，`offset = Σheights_before + index × gap`，全宽全高
- 上滑展开 / 下滑收起；收起态下滑或横滑 dismiss 前台

极简 **Minimal** 与 notification 分离：居中内容宽 chip，不参与重叠栈。

```rust
let toast = SonnerToast::success(42, "保存成功")
    .description("资料已同步")
    .duration_ms(3000);

Sonner {
    toasts: vec![toast],
    position: SonnerPosition::BottomCenter,
    visible_toasts: 3,
    rich_colors: true,
}
```

极简：

```rust
let toast = SonnerToast::minimal(1, "已复制");
// 或
let toast = SonnerToast::success(2, "Saved")
    .appearance(ToastAppearance::Minimal)
    .dismissible(false);
```

| 属性             | 默认值         | 说明                                                           |
| ---------------- | -------------- | -------------------------------------------------------------- |
| `toasts`         | 空             | 结构化 `SonnerToast` 列表                                      |
| `messages`       | 空             | 兼容用纯文本列表，新代码优先 toasts                            |
| `position`       | `BottomCenter` | 上/下 × 左/中/右六种位置                                       |
| `visible_toasts` | `3`            | 重叠栈中同时露出的最大数量，至少 1                             |
| `rich_colors`    | `false`        | 语义强化配色                                                   |
| `style`          | 默认           | 最大宽度、safe-area offset、inset、`stack_offset` 与 card 样式 |

`SonnerToast` 提供 default/success/info/warning/error/loading/minimal constructors，以及 description、appearance、duration、action、dismiss builder。`duration_ms(0)` 保持到显式关闭；Loading 默认保持。timer、action、关闭按钮和 swipe 都必须最终从调用方列表清理对应 id。

手势约定（底部锚点）：

- **上滑**：切换到更旧的 notification（前台后移）
- **下滑**：若前台不是最新则回到更新的卡片；已是最新则 dismiss
- **横向滑动 / 关闭按钮 / action**：dismiss 当前前台卡片
