---
title: Pictorial Bar 象形柱图
description: "Symbol 重复和裁切。"
---

# Pictorial Bar 象形柱图

PictorialBar 用 symbol 的重复、裁切或缩放表达数值，适合少量强调展示。

```rust
let series = Series::pictorial_bar("完成量", [12.0, 18.0, 15.0]);
```

构造签名：`Series::pictorial_bar(name, impl IntoIterator<Item = f64>)`。类别来自 Axis，symbol、repeat、clip、大小、间隔和位置通过 series option/JSON 设置。

象形图不适合高密度精确比较。symbol 尺寸和 repeat 会显著影响布局与绘制成本；需要快速读数时优先普通 Bar。
