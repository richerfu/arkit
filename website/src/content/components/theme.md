---
title: 主题系统
---

# 主题系统

`ThemeProvider` 把一个 `Signal<Theme>` 放入 Dioxus Context。组件调用 `use_theme()` 响应式读取；没有 provider 时回退 `Theme::default()`，便于独立 snippet 与测试。

## 安装主题

```rust
#[entry]
fn app() -> Element {
    rsx! {
        ThemeProvider {
            theme: Theme::dark(ThemePreset::Zinc),
            AppContent {}
        }
    }
}
```

预设包括 Zinc、Neutral、Stone、Mauve、Olive、Mist、Taupe，每个支持 Light/Dark。

## 运行时切换

组件自身拥有主题状态时：

```rust
let mut theme = use_theme_provider(
    Theme::light(ThemePreset::Zinc),
);

rsx! {
    button {
        onclick: move |_| theme.set(Theme::dark(ThemePreset::Zinc)),
        "切换深色"
    }
    AppContent {}
}
```

如果主题由父 Props 控制，优先使用 `ThemeProvider`；它会在 prop 变化时同步内部 Signal。

## Color Tokens

`ColorTokens` 包含 background/foreground、card、popover、primary、secondary、muted、accent、destructive、border、input、ring、surface、chart 1–5 和完整 sidebar token 组。

```rust
let base = Theme::dark(ThemePreset::Neutral);
let custom = base.with_colors(
    base.colors.with_surface(0xFF111827)
);
```

`with_alpha(color, alpha)` 用于基于 token 构造透明色。

## Radius 与尺度

`RadiusTokens` 提供 sm、md、lg、xl、xxl、full；`RadiusTokens::from_base` 可生成一致比例。`spacing`、`typography`、`radius` 模块提供公共常量。

## 自定义 Theme

`Theme::custom(colors)` 创建没有 preset 标记的 Light theme，再用 `with_mode`、`with_radius` 调整。自定义时必须为所有 ColorTokens 提供值，避免某些浮层或 sidebar 回退到不匹配的预设色。

## 主题原则

页面不直接缓存 token；每次 render 调用 `use_theme`。业务品牌色从 Theme 构造，不在组件 props 中层层传相同颜色。需要临时预览时嵌套 Provider，consumer 自动读取最近主题。
