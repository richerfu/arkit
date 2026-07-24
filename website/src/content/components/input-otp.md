---
title: Input OTP
description: "验证码格子输入，支持分隔和逐位填写。"
---

# Input OTP

验证码格子输入：每位一个 slot，中间可以加分隔。

## Props

| 属性                   | 类型                           | 默认值     | 说明                 |
| ---------------------- | ------------------------------ | ---------- | -------------------- |
| `value`                | `String`                       | 空字符串   | 完整输入值           |
| `digits`               | `usize`                        | `6`        | 输入槽数量           |
| `mode`                 | `InputOtpMode`                 | 默认模式   | 文本或数字输入语义   |
| `group_size`           | `usize`                        | `3`        | 每组槽数量           |
| `separator`            | `InputOtpSeparator`            | 默认分隔符 | 分组分隔样式         |
| `masked`               | `bool`                         | `false`    | 隐藏实际字符         |
| `show_caret`           | `bool`                         | `true`     | 显示当前输入位置     |
| `disabled` / `invalid` | `bool`                         | `false`    | 禁用态与错误态       |
| `style`                | `InputOtpStyle`                | 默认样式   | 尺寸、边框等视觉配置 |
| `on_change`            | `Option<EventHandler<String>>` | `None`     | 完整值变化回调       |

验证码属于敏感数据，不应写入普通日志。倒计时、重新发送、自动提交和服务端错误都应由业务层控制。
