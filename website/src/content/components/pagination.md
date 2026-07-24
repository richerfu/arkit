---
title: Pagination
description: "页码和前后翻页。"
---

# Pagination

Pagination 渲染上一页、当前附近页码、间隔省略号和下一页。

```rust
Pagination {
    page: page(),
    total_pages: total_pages(),
    previous_label: "上一页".into(),
    next_label: "下一页".into(),
    on_page_change: move |next| page.set(next),
}
```

| 属性             | 类型                | 说明                                  |
| ---------------- | ------------------- | ------------------------------------- |
| `page`           | `i32`               | 当前页，从 1 开始；自动限制到有效范围 |
| `total_pages`    | `i32`               | 总页数，最小按 1 处理                 |
| `previous_label` | `String`            | 上一页按钮文案                        |
| `next_label`     | `String`            | 下一页按钮文案                        |
| `on_page_change` | `EventHandler<i32>` | 目标页回调                            |

服务器分页还需要在业务层管理 query、loading、total 或 cursor。请求返回时确认结果仍对应当前 query，避免快速翻页导致旧响应覆盖新页面。
