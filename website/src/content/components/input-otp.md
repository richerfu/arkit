---
title: Input OTP
---

# Input OTP

InputOtp 把一个完整字符串投影为多个输入槽，适合验证码和短 PIN。外部状态始终保存完整值，不需要逐槽管理 Signal。

```rust
let mut code = use_signal(String::new);

InputOtp {
    value: code(),
    digits: 6,
    mode: InputOtpMode::Number,
    group_size: 3,
    separator: InputOtpSeparator::Dash,
    on_change: move |next| code.set(next),
}
```

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
