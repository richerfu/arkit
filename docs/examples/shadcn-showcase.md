# shadcn_showcase

路径：`examples/shadcn_showcase`

该示例是 `arkit_shadcn` 的组件展示应用。

## 结构

```text
src/showcase/constants.rs
src/showcase/layout.rs
src/showcase/home.rs
src/showcase/component.rs
src/showcase/examples/*.rs
```

## 使用方式

- `layout.rs` 负责展示页布局。
- `constants.rs` 维护组件分组和导航。
- `examples/*.rs` 每个文件展示一个组件的状态和交互。
- 组件示例应尽量覆盖默认、禁用、变体、交互回调等状态。

## 新组件要求

新增 shadcn 组件时必须同步补 showcase 页面，否则很难在设备上验证 ArkUI 真实表现。
