---
title: 事件处理
description: "点击、输入、滚动如何进业务代码，闭包捕获要注意什么。"
---

# 事件处理

原生 ArkUI 事件会先进入 `EventSink`，再转成 Dioxus 事件按顺序派发。点击、变更这类离散事件不会悄悄丢掉；高频的 pointer move 可以在进 VirtualDom 前按节点合并。

## 事件与 Payload

| 事件                                | Payload            | 用途                       |
| ----------------------------------- | ------------------ | -------------------------- |
| `onclick` / `on_press`              | `ClickData`        | 点击                       |
| `onlongpress`                       | `ClickData`        | 原生长按                   |
| `onchange` / `oninput` / `ontoggle` | `ChangeData`       | 输入与选择                 |
| `onsubmit`                          | `SubmitData`       | 文本提交                   |
| `onscroll`                          | `ScrollData`       | 滚动 offset                |
| `onswiperchange`                    | `SwiperChangeData` | Swiper 页码                |
| `onrefresh`                         | `RefreshData`      | 下拉刷新                   |
| `onarea` / `onlayout`               | `AreaData`         | 布局区域                   |
| `onhover`                           | `HoverData`        | hover 状态                 |
| `onfocus` / `onblur`                | `FocusData`        | 焦点                       |
| `onfocusin` / `onfocusout`          | `FocusData`        | 子树内的焦点变化，会冒泡   |
| `onkey` / `onkeypreime`             | `KeyData`          | 物理按键、修饰键、按键阶段 |
| `onmouse`                           | `MouseData`        | 鼠标按键、移动、目标边界   |
| `onaxis`                            | `AxisData`         | 滚轮与触摸板轴增量         |
| drag/touch/move                     | `PointerData`      | 坐标、阶段、设备           |

事件使用紧凑名称，例如 `onchange`、`onreachend`。历史 snake_case 名称不再注册。

## 键盘与弹层

窗口断点仅选择布局，不关闭键盘、鼠标或触摸能力。原生按键仅入队一次，再由 Dioxus 沿逻辑树冒泡。

`key_capture: "enter space escape"` 声明由控件接管的按键，renderer 在原生回调阶段阻止对应的默认行为。文本编辑器应只接管需要的键；Terminal 使用 `onkeypreime` 与 `key_capture: "all"`，避免同一物理按键再进入 IME。

`focus_scope: true` 在挂载时迁入焦点，在移除时返回仍然有效的触发器；`focus_trap: true` 限制 Tab/Shift+Tab。列表可以用 `focus_navigation: "vertical"` 或 `"horizontal"`。`axis_capture: true` 阻止滚轮继续滚动祖先容器。

## 更新状态

```rust
let mut value = use_signal(String::new);

rsx! {
    textinput {
        value: value(),
        placeholder: "请输入",
        onchange: move |event| value.set(event.string_value.clone()),
    }
}
```

事件闭包通常捕获可复制的 Signal handle。回调应更新状态或派发领域命令，不直接保留 event 内部借用。

## 冒泡

浮层 panel 可阻止点击继续到 backdrop：

```rust
column {
    onclick: move |event| event.stop_propagation(),
    text { "弹层内容" }
}
```

只在组件确实拥有事件边界时停止传播；大面积容器无条件拦截会破坏上层手势和无障碍行为。

## 长按与默认行为

`onlongpress` 使用 ArkUI LongPress Gesture，默认单指保持约 500ms 后在 Accept 阶段派发一次。它不是延迟 click，普通点击不会触发长按 handler。

## Native callback 边界

不在 Dioxus event dispatch 内触发的原生回调可能发生在 tree patch 中。此时用当前 root 的 `RuntimeHandle` 把状态更新排到下一 UI tick，避免重入 render：

```rust
let runtime = use_runtime_handle();
let mut title = use_signal(String::new);
let callback = move |next: String| {
    runtime.queue_ui(move || title.set(next));
};
```
