---
title: Form
---

# Form

Form 提供表单布局与语义 primitive，不包含业务 schema。字段值、校验时机、提交任务和服务端错误由页面状态拥有。

```rust
Form {
    submit_label: "保存",
    submit_disabled: submitting(),
    on_submit: move |_| save(),
    FieldGroup {
        Field {
            invalid: error().is_some(),
            FieldLabel { content: "用户名", required: true }
            FieldContent {
                Input { value: name(), on_change: move |v| name.set(v) }
                FieldDescription { content: "最多 20 个字符" }
                FieldError { message: error() }
            }
        }
    }
}
```

## 组合组件

| 组件                      | 主要属性                                                  | 职责               |
| ------------------------- | --------------------------------------------------------- | ------------------ |
| `Form`                    | `submit_label`、`on_submit`、`submit_disabled`、`surface` | 表单容器与提交入口 |
| `Field`                   | `orientation`、`invalid`、`disabled`                      | 单字段布局         |
| `FieldContent`            | `children`                                                | 输入与辅助信息区域 |
| `FieldLabel`              | `content`、`required`、`invalid`                          | 字段标签           |
| `FieldTitle`              | `content`                                                 | 字段组标题         |
| `FieldDescription`        | `content`、`inset`                                        | 帮助文字           |
| `FieldError`              | `message`、`errors`                                       | 单条或多条错误     |
| `FieldGroup` / `FieldSet` | `children`                                                | 字段分组           |
| `FieldLegend`             | `content`、`variant`                                      | FieldSet 标题      |
| `FieldSeparator`          | `label`                                                   | 字段组分隔         |
| `FormItem`                | 标签、说明、错误、状态与子元素                            | 简化组合           |

同步校验可由 `use_memo` 派生，服务端校验放在异步任务中。提交失败保留输入；错误不仅改变边框，还要渲染可读文字并定位到第一个无效字段。
