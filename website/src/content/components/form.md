---
title: Form
description: "一组 Field 原语，方便统一表单校验与布局。"
---

# Form

表单由 Field 原语、输入组件和普通 Button 组合而成，框架不额外提供表单容器。

## 组合组件

| 组件                      | 主要属性                             | 职责               |
| ------------------------- | ------------------------------------ | ------------------ |
| `Field`                   | `orientation`、`invalid`、`disabled` | 单字段布局         |
| `FieldContent`            | `children`                           | 输入与辅助信息区域 |
| `FieldLabel`              | `content`、`required`、`invalid`     | 字段标签           |
| `FieldTitle`              | `content`                            | 字段组标题         |
| `FieldDescription`        | `content`、`inset`                   | 帮助文字           |
| `FieldError`              | `message`、`errors`                  | 单条或多条错误     |
| `FieldGroup` / `FieldSet` | `children`                           | 字段分组           |
| `FieldLegend`             | `content`、`variant`                 | FieldSet 标题      |
| `FieldSeparator`          | `label`                              | 字段组分隔         |

同步校验可由 `use_memo` 派生，服务端校验放在异步任务中。提交失败保留输入；错误不仅改变边框，还要渲染可读文字并定位到第一个无效字段。
