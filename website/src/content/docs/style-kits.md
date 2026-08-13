---
title: 样式库与主题
description: "无样式原语、样式外覆，以及 shadcn / ColorUI 各自管什么。"
---

# 样式库与主题

`arkit_component` 只提供结构、状态和交互。颜色、圆角、阴影、几何都由外部重写：样式库传入 `appearance`，或通过 token 覆盖 `use_theme()`。

两套样式库能力对齐，只换皮。

## shadcn

同一套 headless 控件，shadcn 在样式库里重写皮。`Theme` / `ThemeProvider` 只做 Zinc / Neutral / 明暗切换，并挂上 `ShadcnKit`。复合层（Dialog / Sheet / Form / Select / Tabs）按 [shadcn/ui new-york-v4](https://ui.shadcn.com) 重写，不是指望 headless 自己长得像 shadcn。按钮默认 48vp 是官方 `h-9` 的触控映射（`h-12`），和拆分前的移动端 kit 一致。

```rust
use arkit::shadcn::components::*;
use arkit::shadcn::theme::{Theme, ThemePreset, ThemeProvider};

ThemeProvider {
    theme: Theme::light(ThemePreset::Zinc),
    Button { variant: ButtonVariant::Default, "保存" }
}
```

## ColorUI

同一套组件名和 props，外观按 [ColorUI](https://github.com/weilanwl/coloruicss) 重写。

```rust
use arkit::colorui::prelude::*;

fn app() -> Element {
    use_colorui(ColorUiTheme::light(PaletteColor::Green));
    rsx! {
        Button { variant: ButtonVariant::Default, "保存" }
        Button { color: Some(PaletteColor::Blue), "蓝色" }
    }
}
```

`ColorUiTheme` 只提供默认主色和页面底。`use_colorui` 会同时挂上 `ColorUiKit`，嵌套的 headless 原语（例如 `AlertDialogAction` 里的 Button）也会涂成 ColorUI。复合层（Dialog / Sheet / Form / Select）按 ColorUI 结构重写，不是换一套 token 就完事。不要把 `component::Button` 和样式库 Button 混用后再指望 Theme 换皮。

ColorUI 另有自己的复合组件：`Bar`、`Tag`、`List`、`Nav`、`Timeline`、`Steps`、`Chat` 等。

完整示例：`examples/shadcn_showcase` 与 `examples/colorui_showcase`（同一套页面，不同皮）。
