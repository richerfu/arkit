# 业务 UI 与组件库

基础 UI 直接使用 ArkUI element registry：

```rust
rsx! {
    column {
        padding: 16.0,
        text { font_size: 24.0, "Profile" }
        button { onclick: move |_| save(), "Save" }
    }
}
```

属性和事件由 `arkit_elements` 定义，renderer 将它们编码为 declarative desired state。组件应使用标准 `#[component]` props 和 Dioxus composition。

## 事件

事件 handler 修改 signal，renderer 负责把 ArkUI node event 或 gesture 转成 Dioxus event：

```rust
rsx! {
    row {
        onclick: move |_| select_item(),
        onlongpress: move |_| open_context_menu(),
        "Long press"
    }
}
```

`onlongpress` 和 `on_long_press` 都表示真实的 ArkUI LongPress Gesture：单指保持约 500ms 后触发一次。普通点击不会触发 long-press handler。组件禁止用 `onclick` 模拟长按；缺少的交互必须在 `arkit_elements` 和 `arkit_arkui` 的事件桥中实现。

原生 callback 不同步执行组件 handler。payload 会先进入 runtime event queue，再由 OpenHarmony UI loop 调用 Dioxus handler 并渲染更新，因此 handler 可以安全地修改 signal 或切换大块 subtree。

`arkit_shadcn` 提供 theme context 与业务组件：

```rust
rsx! {
    ThemeProvider {
        theme: Theme::default(),
        Button { "Submit" }
    }
}
```

需要组件内部可变主题时使用 `use_theme_provider`；普通子组件通过 `use_theme` 读取。实际设备覆盖见 `examples/shadcn_showcase`。

## Overlay 与受控状态

浮层统一通过应用级 `OverlayRoot` 渲染，内容仍处于当前 Dioxus tree。菜单、Popover、HoverCard 等组件不得创建第二个 VirtualDom，也不得把首次打开时生成的 `Element` 当作永久内容快照。

受控 menu entry 在浮层保持打开时必须立即反映最新 props，例如 checkbox/radio 点击后马上更新选中标记。共享菜单实现使用 overlay session 重新发布同一个 subtree，同时保留 submenu 的本地 hook 状态。

## Row/Column 对齐

ArkUI `Row` / `Column` 的 native 默认值不等于所有 shadcn 布局的设计默认值。组件内容需要左对齐时应显式写出 `align_items: "start"`；有固定宽度或会换行的 Text 还应显式使用 `text_align: 0`。不要依赖外层浮层的 start 对齐跨越多层容器自动生效。
