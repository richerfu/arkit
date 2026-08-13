---
title: 主题系统
description: "主题预设和明暗模式，以及运行时如何换肤。"
---

# 主题系统

`Theme` 只管 **shadcn 这一套风格** 的预设和明暗。Zinc / Neutral / Stone 以及 Light / Dark 都是同一批组件换 token，不是换组件库。组件皮在 `arkit_shadcn` 里按官方 class 重写，对照见 `crates/arkit_shadcn/src/spec.rs`。

ColorUI 不走这套 Theme。要用 ColorUI 就用 `arkit::colorui::components::*`，见 [ColorUI](../colorui/)。

## 挂载 shadcn 主题

```rust
ThemeProvider {
    theme: Theme::dark(ThemePreset::Zinc),
    AppContent {}
}
```

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

## 对照证据

出处：[shadcn/ui new-york-v4](https://github.com/shadcn-ui/ui/tree/main/apps/v4/registry/new-york-v4/ui)。`1rem = 16vp`。按钮默认用 `h-12`（48）做触控映射，对应官方 `h-9`。

| 组件                | 官方 class                                 | 规格                            | 本实现 (`spec`)                                  |
| ------------------- | ------------------------------------------ | ------------------------------- | ------------------------------------------------ |
| Zinc 底/字          | `--background` / `--foreground`            | `#FFFFFF` / `#09090B`           | `ZINC_BG` / `ZINC_FG`                            |
| Zinc primary        | `--primary`                                | `#09090B` / fg `#FAFAFA`        | `ZINC_PRIMARY`                                   |
| Zinc muted / border | `--muted-foreground` / `--border`          | `#71717A` / `#E4E4E7`           | 同左                                             |
| Button              | `h-9 px-4` / `h-8` / `h-10` / `size-9`     | 36 / 32 / 40 / 36               | 移动端 48 / 36 / 56 / 40                         |
| Button 变体         | `bg-primary` / `border` / `bg-destructive` | primary / outline / destructive | kit `shadcn_button`                              |
| Dialog              | `sm:max-w-lg p-6 rounded-lg bg-black/50`   | 512 / 24 / 8 / 50% 黑           | `DIALOG_MAX_W` / `PAD` / `RADIUS_LG` / `OVERLAY` |
| Dialog 标题         | `text-lg font-semibold`                    | 18 / 600                        | `TEXT_LG` / `FONT_SEMIBOLD`                      |
| Card                | `rounded-xl border p-6`                    | 12 / 1px / 24                   | `RADIUS_XL` / 24                                 |
| Input               | `h-9 rounded-md text-sm`                   | 36 / 6 / 14                     | 触控 48 / 6 / 16                                 |
| Switch              | `h-5 w-9`                                  | 20×36                           | 预拆分 18.4×32                                   |
| Checkbox            | `size-4 rounded-[4px]`                     | 16                              | `CHECK`                                          |
| Progress            | `h-2`                                      | 8                               | `PROGRESS_H`                                     |
| Avatar              | `size-8`                                   | 32                              | `AVATAR`                                         |
| Popover             | `w-72 rounded-md border`                   | 288 / 6                         | `POPOVER_W`                                      |
| Sheet               | `sm:max-w-sm`                              | 384                             | `SHEET_W`                                        |
| Tabs                | muted 轨道 + active `bg-background`        | 36 高                           | `TabsList`                                       |
| AlertDialog         | `flex-col-reverse` 移动端                  | 动作在上                        | 先 action 后 cancel                              |

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
