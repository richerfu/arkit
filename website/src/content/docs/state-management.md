---
title: 状态管理
description: "Signal、派生状态、共享状态与更新原则。"
---

# 状态管理

Arkit 直接使用 Dioxus Signal。状态读取建立响应式依赖，写入唤醒相关 scope；renderer 随下一次 VirtualDom diff 更新 native tree。

## 局部状态

```rust
let mut count = use_signal(|| 0_u32);

rsx! {
    button {
        onclick: move |_| count += 1,
        "count = {count}"
    }
}
```

Signal handle 指向同一状态单元，可以复制进 `Fn` 事件闭包。不要为了 UI 局部状态额外包 Mutex。

## 派生状态

```rust
let query = use_signal(String::new);
let normalized = use_memo(move || query().trim().to_lowercase());
let empty = use_memo(move || normalized().is_empty());
```

能从其他状态计算的值不再存一份 Signal。这样不会出现“query 已变而 normalized 尚未同步”的中间态。

## 状态放在哪里

| 状态                 | 推荐所有者                                |
| -------------------- | ----------------------------------------- |
| 输入框、展开状态     | 最近的交互组件                            |
| 页面筛选与加载结果   | 页面组件或页面 Provider                   |
| 登录态、主题、locale | 应用级 Provider                           |
| 页面身份             | Router history                            |
| 高频动画进度         | Animation Engine；需要显示时订阅 snapshot |

把状态提升到所有消费者的最近公共祖先即可，不默认建立全局 store。

## 受控与非受控

可复用组件优先提供“值 + change callback”：

```rust
#[component]
fn Counter(value: i32, on_change: EventHandler<i32>) -> Element {
    rsx! {
        button { onclick: move |_| on_change.call(value + 1), "{value}" }
    }
}
```

非受控状态适合组件完全拥有的短期交互。表单提交、路由和服务端数据通常应受控，便于校验和恢复。

## 更新原则

- 一次用户操作尽量形成一次清晰的状态变更。
- 不从 native node 反读业务真相；native tree 是状态投影。
- 大集合更新保留稳定 item id，避免全部节点失去 key。
- 跨线程任务返回 owned 数据，再在 UI 调度器中写 Signal。
