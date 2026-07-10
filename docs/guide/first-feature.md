# 开发一个业务功能

以“加载用户并展示”为例：

1. 在 service 中定义 async 操作。
2. 页面 component 用 signal 保存输入。
3. 用 `use_resource` 依赖输入执行加载。
4. 用 `rsx!` 表达 loading/error/content。
5. 将重复 UI 提取为带 Props 的 `#[component]`。

```rust
#[component]
fn UserPage(id: u64) -> Element {
    let user = use_resource(move || async move { load_user(id).await });

    match (user.value())() {
        None => rsx! { text { "Loading..." } },
        Some(Ok(user)) => rsx! { text { "{user.name}" } },
        Some(Err(error)) => rsx! { text { "failed: {error}" } },
    }
}
```

不要再引入 Message enum 或手写 diff/reconcile；Dioxus 已拥有这些职责。
