---
title: 交互与无障碍
description: "ArkUI 语义、状态、动作和自定义交互组件的无障碍接入。"
---

# 交互与无障碍

Arkit 会把 RSX 无障碍属性直接写入 ArkUI 原生节点。普通 `button`、`textinput`、`checkbox`、`toggle`、`radio`、`slider` 以及由 `row` / `column` 封装的自定义控件都不需要额外接入 `ohos-a11y-binding`。

## 原生 button

`button` 默认具备 ArkUI Button role、可聚焦、语义分组和 click action；`onclick` 会同时响应普通点击和无障碍服务发出的 click action。图标按钮仍需显式提供可访问名称：

```rust
button {
    accessibility_text: "关闭对话框",
    accessibility_description: "返回上一页",
    enabled: !loading,
    onclick: move |_| close.call(()),
    Icon { name: "x" }
}
```

`enabled: false` 会同步到无障碍 disabled state。也可以显式使用 `accessibility_disabled`，但视觉状态和语义状态必须保持一致。

## 自定义 button

如果视觉按钮由 `row`、`column` 或 `stack` 实现，至少要补齐 role、名称、动作、焦点和状态：

```rust
row {
    accessibility_role: "button",
    accessibility_text: "提交订单",
    accessibility_description: "提交后进入支付页",
    accessibility_group: true,
    accessibility_actions: "click",
    accessibility_disabled: submitting,
    focusable: !submitting,
    focus_on_touch: true,
    enabled: !submitting,
    onclick: move |_| submit.call(()),

    text {
        accessibility_mode: "disabled",
        "提交订单"
    }
}
```

只要声明了 `accessibility_actions: "click"`，ArkUI 的 `performAction("click")` 就会触发控件原有的 `onclick`，不需要再写一套读屏点击逻辑。不要在 `onaccessibilityaction` 中再次调用同一个 click handler，否则一次读屏激活会执行两次业务动作。

如需观测 accessibility action，或处理 click 以外的动作，可以监听 `onaccessibilityaction`：

```rust
row {
    accessibility_role: "button",
    accessibility_text: "复制邀请码",
    accessibility_actions: "long_click",
    onaccessibilityaction: move |event| {
        // 当前节点只声明了 LongClick，因此无需比较原始整数 action。
        // AccessibilityManager provider action 与 ArkUI 属性 bit mask
        // 使用不同编号空间，不能把这里的值硬编码成属性掩码。
        open_context_menu();
    }
}
```

## 可用属性

| 分类   | RSX 属性                                    | 用途                                                                                                  |
| ------ | ------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| 名称   | `accessibility_text`                        | 读屏首先朗读的控件名称                                                                                |
| 提示   | `accessibility_description`                 | 操作说明、展开/折叠等补充信息                                                                         |
| 结构   | `accessibility_group`                       | 把当前节点和后代作为一个语义目标                                                                      |
| 可见性 | `accessibility_mode`                        | `auto`、`enabled`、`disabled`、`disabled_for_descendants`                                             |
| 角色   | `accessibility_role`                        | `button`、`checkbox`、`radio`、`switch`、`slider`、`progress`、`text_input`、`list` 等 ArkUI 节点角色 |
| 动作   | `accessibility_actions`                     | `click`、`long_click`、`cut`、`copy`、`paste`，使用竖线组合                                           |
| 状态   | `accessibility_disabled`                    | 禁用状态                                                                                              |
| 状态   | `accessibility_selected`                    | Tab、菜单项、轮播页等选中状态                                                                         |
| 状态   | `accessibility_checked`                     | Checkbox、Radio、Switch、Toggle 的勾选状态                                                            |
| 数值   | `accessibility_value_min/max/current`       | 整数范围和值                                                                                          |
| 数值   | `accessibility_value_range_min/max/current` | ArkUI API 18+ 范围值                                                                                  |
| 数值   | `accessibility_value_text`                  | 百分比、带单位值等可朗读文本                                                                          |

所有基础元素都支持上述属性，`Row` / `Col` 封装也提供同名 builder 方法。

## shadcn 组件

shadcn 的交互组件已经在根节点声明相应语义：

- `Button`、菜单、分页、导航、手风琴和弹层 trigger 暴露 button/action/disabled/selected 状态。
- `Checkbox`、`RadioGroup`、`Switch`、`Toggle` 暴露 checked/selected 状态。
- `Slider`、`Progress` 暴露范围、当前值和可朗读 value text。
- `Input`、`Textarea`、`InputOtp` 暴露文本输入角色、名称和禁用状态。
- `Select`、`Combobox`、`Tabs`、日期/时间选择器、轮播和菜单会隐藏非活动内容并暴露展开、选中状态。
- Dialog、Sheet、Drawer、BottomSheet、Toast 和 Guide 为面板及图标操作提供名称；装饰性遮罩不会成为读屏目标。
- Card、Badge、Code、Form、Table、Timeline、Index、ScrollArea 等内容与结构组件暴露 heading、group、list、list item、scroll、required、invalid 等语义，同时保留可交互后代。

包含可见文字的组件通常可以从 label 得到名称。纯图标或业务含义不同于可见文字时，必须传 `accessibility_label`：

```rust
Button {
    size: ButtonSize::Icon,
    accessibility_label: "添加联系人",
    onclick: move |_| add.call(()),
    Icon { name: "plus" }
}

Switch {
    accessibility_label: "接收订单通知",
    checked: Some(notifications()),
    on_change: move |value| notifications.set(value),
}
```

## 自绘与 XComponent

`Canvas`、原生 `ECharts`、`Barcode`、`LottiePlayer`、`CameraPreview` 和 `Terminal` 都提供可访问名称或文本替代入口：

- `Canvas` 和 `LottiePlayer` 默认按装饰内容处理；业务内容有意义时传 `accessibility_label` 和 `accessibility_description`。
- `ECharts`、`CameraPreview` 和 `Terminal` 默认提供可覆盖的名称；图表摘要、相机操作提示和终端关键输出通过 `accessibility_description` 提供。
- `Barcode` 默认只朗读“Barcode”，不会自动朗读可能包含敏感信息的 payload；业务方可显式提供用途描述。
- `CameraView` 内置的闪光灯、镜头切换、快门、分辨率选择和缩放控件均具备名称、状态、焦点及原生 action。

这些组件把整个绘制表面作为一个 ArkUI 语义节点时，不需要额外 provider。

## 什么时候需要 ohos-a11y-binding

常规 ArkUI 原生节点不需要。Arkit 通过 `ohos-arkui-binding` 的节点属性和事件接口即可生成系统可识别的语义树。

只有自绘表面内部还要暴露多个可独立聚焦、命中和操作的虚拟子节点时才需要 `ohos-a11y-binding`，例如终端的逐行浏览、图表的数据点、地图标记或游戏内按钮。这类场景需要注册自定义 accessibility provider、维护虚拟节点树、命中测试和 action 回调；它不应该成为普通组件库或“整块表面 + 文本替代”场景的强依赖。

## 设备验收

`examples/demos` 中的“无障碍验收”页面同时覆盖原生 button、自定义 row button、CustomNode、Barcode、原生图表和 shadcn 控件。仓库脚本直接使用 `ohos-qemu` 带 a11y 能力的镜像，不修改或重打镜像：

```bash
scripts/run-qemu-a11y-e2e.sh
```

QEMU/真机验收至少检查：

1. ArkUI/UITest 树中存在稳定的 `A11Y` 名称、正确 role、checked/selected/disabled/value 和 click action。
2. 通过无障碍 action 激活原生、自定义和 shadcn button，页面计数各增加一次。
3. Checkbox、Switch 的状态切换后，语义树中的 checked state 同步变化。
4. 键盘/读屏焦点不会进入遮罩、装饰图标或隐藏的 Tab/Carousel 内容。

无障碍树和真实 action 是最终标准；普通坐标点击与截图只能验证视觉结果，不能替代这组检查。
