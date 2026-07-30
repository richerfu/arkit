---
title: Alert
description: "页内提示条：标题、说明和补充列表。"
---

# Alert

页内提示条，用来放警告、错误或补充说明，而不是打断当前流程的模态框。

## 用法

```rust
Alert {
    icon: "circle-alert",
    variant: AlertVariant::Destructive,
    AlertTitle {
        content: "保存失败",
        variant: AlertVariant::Destructive,
    }
    AlertDescription {
        content: "检查网络后重试",
        variant: AlertVariant::Destructive,
    }
    AlertList {
        items: vec!["网络不可用".into(), "草稿仍保留".into()],
        variant: AlertVariant::Destructive,
    }
}
```

## API

`Alert`：`icon`、`variant`、`children`。`AlertTitle`/`AlertDescription`：`content`、`variant`。`AlertList`：`items`、`variant`。

Variant 只有 Default 与 Destructive。子 primitive 不会自动继承 root variant，组合时传入相同 variant，保证标题和说明色一致。

Alert 是页内内容，不自动关闭。短时后台结果使用 Toast。
