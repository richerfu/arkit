# router

路径：`examples/router`

router 示例展示页面模块拆分和导航集成：

```text
src/lib.rs
src/routes.rs
src/components.rs
src/pages/home.rs
src/pages/settings.rs
src/pages/user.rs
src/pages/not_found.rs
```

## 使用方式

- `routes.rs` 定义路由表和导航入口。
- `pages/*` 每个文件负责一个页面 view。
- 页面按钮产生 `RouterMessage` 或应用自己的导航 Message。
- 未匹配路径进入 not found 页面。

## 适合参考的场景

- 应用页面拆分。
- 带参数路由。
- 返回和替换导航。
- 页面级组件组织。
