---
title: 上下文与 Provider
description: "跨层依赖、provider 所有权与作用域。"
---

# 上下文与 Provider

Context 用于跨多层组件共享稳定依赖。Provider 把值放在当前 subtree，consumer 读取最近一层同类型值。

## 定义上下文

```rust
#[derive(Clone, Copy)]
struct SessionContext {
    user_id: Signal<Option<u64>>,
}

#[component]
fn SessionProvider(children: Element) -> Element {
    let user_id = use_signal(|| None);
    use_context_provider(|| SessionContext { user_id });
    rsx! { {children} }
}

#[component]
fn ProfileButton() -> Element {
    let session = consume_context::<SessionContext>();
    rsx! { button { "user = {session.user_id:?}" } }
}
```

Context 类型应表达领域职责。不要用一个巨大 `AppContext` 包含所有可变状态，否则任何模块都会隐式依赖整个应用。

## Provider 所有权

Provider 负责：

- 创建状态或外部资源。
- 向 subtree 发布稳定 handle。
- 在 scope 卸载时取消订阅、任务和 native 资源。
- 定义没有 provider 时是 panic、错误还是默认值。

Arkit 自带的 ArkHost、I18n、Router、Theme 和 AnimationHost 都遵循这一模型。

## 何时不用 Context

只有一两层传递时直接用 Props，依赖关系更清晰。列表 item 的当前数据也应通过 Props，而不是循环里覆盖同类型 Context。

## 响应式粒度

Context 本身通常保持稳定，内部使用多个 Signal 划分更新粒度。consumer 只读取需要的 Signal；把每次变化都重新构造为一个大结构会扩大重渲染范围。

## 嵌套覆盖

测试、预览或子路由可以安装同类型 Provider 覆盖上层。例如某个设置页临时预览另一主题。consumer 总是读取最近的 provider，卸载后自然回到外层值。
