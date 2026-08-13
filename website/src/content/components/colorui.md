---
title: ColorUI
description: "高饱和色板、Bar / Tag / Timeline / Steps / Chat 等 ColorUI 组件。"
---

# ColorUI

基于 `arkit_component` headless 原语重写的一套组件，不是往 Button 上挂 theme。

色值对齐 [weilanwl/coloruicss](https://github.com/weilanwl/coloruicss) `main.css`：red / orange / yellow / olive / green / cyan / blue / purple / mauve / pink / brown / grey / gray / black / white，以及 gradual-blue 等渐变起点色。

## 使用

```rust
use arkit::colorui::prelude::*;

fn app() -> Element {
    use_colorui(ColorUiTheme::light(PaletteColor::Green));
    rsx! {
        Button { "默认绿" }
        Button { color: Some(PaletteColor::Blue), round: Some(true), "按钮" }
    }
}
```

`use_colorui` / `ColorUiProvider` 提供默认 `primary`、页面底色，并挂上 `ColorUiKit`，这样 Dialog 里的 headless `Button` 也会走 `cu-btn`。

未指定 `color` 的按钮走 `theme.primary`（默认 `#39B54A`）。暗色用 `ColorUiTheme::dark(...)`。

## 对照证据

单位：ColorUI `upx` 按 750 设计稿换算 `vp = upx / 2`。色值直接抄 `main.css`。锁定测试在 `crates/arkit_colorui/src/spec.rs`。

| 组件                                   | ColorUI 出处                              | CSS / 规格                                                     | 本实现                               |
| -------------------------------------- | ----------------------------------------- | -------------------------------------------------------------- | ------------------------------------ |
| 色板                                   | `.bg-green` 等                            | `#39B54A` / `#E54D42` / `#0081FF` / 页底 `#F1F1F1` / 字 `#333` | `spec::BG_*`、`ColorUiTheme::tokens` |
| Button                                 | `.cu-btn` / `.sm` / `.lg`                 | 高 64/48/80upx，字 28/20/32upx，左右 30/20/40upx，禁用 0.6     | 32/24/40，字 14/10/16                |
| Button 默认灰                          | `.cu-btn:not([class*="bg-"])`             | `#f0f0f0`                                                      | Secondary / Gray                     |
| Button 线框                            | `.cu-btn[class*="line"]`                  | 透明底 + currentColor 边                                       | `Outline`                            |
| Badge / Tag                            | `.cu-tag`                                 | 高 48upx，字 24upx，无圆角                                     | 24 / 12 / radius 0                   |
| Card                                   | `.cu-card>.cu-item`                       | 白底，圆角 10upx，margin 30upx                                 | radius 5，padding 15                 |
| Switch                                 | `switch .wx-switch-input`                 | 48×26，关 `#8799a3`，开 `#39b54a`                              | 同左                                 |
| Checkbox / Radio                       | `checkbox/radio 24px` + `.green[checked]` | 24px，选中 `#39b54a`                                           | `CHECK_RADIO` 24                     |
| Progress                               | `.cu-progress`                            | 高 28upx，轨道 `#ebeef5`                                       | 14 / `#EBEEF5`                       |
| Avatar                                 | `.cu-avatar`                              | 64upx，fallback `#ccc`                                         | 32 / `#CCCCCC`                       |
| Input / Textarea                       | `.cu-form-group input/textarea`           | 字 30upx，色 `#555`，无描边组                                  | 字 15/14，底边 `#eee`                |
| Alert                                  | `.bg-*.light`                             | 绿浅底 `#d7f0db` + 绿字                                        | `light_fill` / 无边框                |
| Text                                   | `.text-df/sm/lg/xl/xxl`                   | 28/24/32/36/44upx，`#333`/`#888`                               | 14/12/16/18/22                       |
| Dialog                                 | `.cu-modal` + `.cu-dialog`                | 遮罩 `rgba(0,0,0,.6)`，宽 680upx，底 `#f8f8f8`，圆 10upx       | 0x99、340、`#F8F8F8`、5              |
| Dialog 头/脚                           | `.cu-bar.bg-white`                        | 高 100upx，标题居中，红关                                      | 50，红 `x`                           |
| Sheet / Drawer / BottomSheet           | `.drawer-modal` / `.bottom-modal`         | 全高/底栏，无圆角或贴边                                        | 同结构                               |
| Form / Field / Date / Time / Select    | `.cu-form-group`                          | 白底，高 100upx，左右 30upx                                    | 50 / 15                              |
| Tabs / Breadcrumb / Nav                | `.nav .cu-item.cur`                       | 高 90upx，底边 4upx                                            | 下划线 2 / 主色                      |
| Accordion / Collapsible / Table / Menu | `.cu-list.menu>.cu-item`                  | 白底，最小 100upx，底部分割                                    | 50 行 + `#eee`                       |
| BottomNavigation                       | `.cu-bar.tabbar`                          | 高 100upx，字 22upx，图标 40upx                                | 50 / 11 / 20                         |
| Toggle / ToggleGroup                   | `.cu-btn.sm` + `.cu-capsule`              | 线/填，高 48upx                                                | 24–32，胶囊拼接                      |
| Calendar                               | 选中态映射 `.bg-green`                    | 默认绿，不是 shadcn sky-600                                    | `selection_color = #39B54A`          |
| Slider                                 | 映射 `.cu-progress` + 开关圆点            | 轨道 `#ebeef5`，高 28upx，圆点 26                              | 同左                                 |
| Carousel                               | `swiper.square-dot` + `.cu-card`          | 点 10upx / 激活 30upx，圆 10upx                                | 5 / 15 / 5                           |
| Tooltip                                | `.cu-chat .cu-info`                       | `rgba(0,0,0,.2)`，白字，圆 6upx                                | `0x33000000` / 3                     |
| Popover / HoverCard                    | `.cu-dialog` 碎片                         | 白卡，圆 10upx                                                 | 5                                    |
| Toast / Sonner                         | 无官方 toast → 白卡 + 语义色              | 圆 10upx，动作 `.bg-green`                                     | `ToastStyle` 注入                    |
| Pagination                             | `.cu-btn.sm`                              | 线/填                                                          | 24 高                                |
| Spinner / Load                         | `.cu-load` / `.load-modal`                | 主色 / 橙转圈                                                  | 主色 LoadingProgress                 |
| Dropdown / Context / Menubar / Command | 动作面板 = 底栏 + `.cu-list.menu`         | ColorUI 没有 shadcn 浮层菜单                                   | 底 modal 列表                        |

```rust
Button { color: Some(PaletteColor::Blue), shadow: Some(true), "蓝色" }
Button { variant: ButtonVariant::Outline, color: Some(PaletteColor::Red), "线框" }
Button { round: Some(true), size: ButtonSize::Lg, "圆角" }
Tag { content: "新品".into(), color: PaletteColor::Olive, round: true }
```

`color` / `round` / `block` 是 headless 上的色板提示；ColorUI kit 把 `Outline` 画成 `line-*`。

## 复合组件

| 组件                             | 对应 ColorUI                                       |
| -------------------------------- | -------------------------------------------------- |
| `Bar`                            | `cu-bar` / `cu-custom`（导航、搜索、tabbar、底栏） |
| `Tag` / `Capsule`                | `cu-tag` / `cu-capsule`                            |
| `List` / `ListItem` / `GridList` | `cu-list.menu` / `grid`                            |
| `Nav` / `NavItem`                | `.nav` 下划线导航                                  |
| `Timeline` / `TimelineItem`      | `cu-timeline`                                      |
| `Steps` / `StepItem`             | `cu-steps`                                         |
| `Chat` / `ChatItem` / `ChatInfo` | `cu-chat`                                          |
| `Load` / `LoadModal`             | `cu-load`                                          |
| `FormGroup`                      | `cu-form-group`                                    |
| `AvatarGroup`                    | `cu-avatar-group`                                  |
| `Indexes`                        | 字母索引条                                         |

示例：`examples/colorui_showcase`。
