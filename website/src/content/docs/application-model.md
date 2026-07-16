---
title: 组件与 RSX
description: "组件边界、Props、children 与组合模式。"
---

# 组件与 RSX

组件是 Arkit 应用的基本边界。它接收 Props、读取 Hook 状态并返回 `Element`；RSX 描述的不是一次性 native node，而是下一次 VirtualDom diff 的目标结构。

## 定义组件

```rust
#[component]
fn Greeting(name: String, emphasis: bool) -> Element {
    rsx! {
        text {
            font_size: if emphasis { 22.0 } else { 16.0 },
            "你好，{name}"
        }
    }
}
```

`#[component]` 生成 Props 类型和调用胶水。Props 应是可比较的轻量值或稳定 handle；不要把大型可变对象按值复制进每个子组件。

## 组合 children

容器组件通过 `Element` children 组合内容：

```rust
#[component]
fn Section(title: String, children: Element) -> Element {
    rsx! {
        column {
            width: 320.0,
            text { font_size: 18.0, "{title}" }
            {children}
        }
    }
}

rsx! {
    Section { title: "账户",
        text { "Ada" }
    }
}
```

直接调用组件函数会绕过正常组件身份；始终在 RSX 中实例化组件。

## 条件与列表

```rust
rsx! {
    column {
        if loading() {
            loadingprogress {}
        } else {
            for item in items() {
                text { key: "{item.id}", "{item.title}" }
            }
        }
    }
}
```

会插入、删除或重排的列表必须给稳定 `key`。key 表达业务身份，不使用当前下标；这样 Hook 状态和 native node 才会跟随正确 item。

## 组件边界原则

- 一个组件只拥有自己能完整创建、更新和清理的状态。
- 共享业务状态通过 Context/Provider 传递，不用全局可变静态量。
- native handle、订阅和后台任务跟随创建它们的 scope 清理。
- 纯展示组件接收值和回调；页面组件负责路由、请求和长期状态。

## Root 组件

`#[entry]` 自动安装 ArkHost、SafeArea、OverlayRoot，以及启用 feature 时的 AnimationHost。业务 root 直接返回页面：

```rust
#[entry]
fn app() -> Element {
    rsx! { HomePage {} }
}
```

不要在业务 root 重复调用 framework provider，否则会产生多个 host 所有者和不一致的节点解析范围。
