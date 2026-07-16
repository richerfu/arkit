---
title: 资源与消息
description: "FTL 资源、参数、选择器和编译期校验。"
---

# 资源与消息

Arkit 使用 Fluent `.ftl` 资源。每个 locale 必须提供同一组公开 message/attribute，并在解析引用后需要相同变量集合。

## 资源示例

`locales/zh-CN.ftl`：

```ftl
-brand = Arkit

app-title = 示例应用
welcome = 欢迎使用 { -brand }，{$name}。
inbox-count = { $count ->
    [one] 一条消息
   *[other] { $count } 条消息
}
account-button =
    .label = 打开账户
    .hint = 当前用户：{$name}
```

term 用 `-` 开头，只用于资源内部复用，不生成公开 Rust constructor。message attribute 会生成独立的类型安全函数。

## 宏生成内容

```rust
arkit::i18n! {
    pub mod tr {
        path: "locales",
        fallback: "zh-CN",
        locales: ["zh-CN", "en-US"],
    }
}
```

生成 `tr::Locale`、`FALLBACK_LOCALE`、`CATALOG`，以及：

- `tr::app_title()`
- `tr::welcome(name)`
- `tr::inbox_count(count)`
- `tr::account_button_label()`
- `tr::account_button_hint(name)`

参数名来自 Fluent 变量；string、整数、float 和 bool 会转换为 `I18nValue`。

## Fluent 能力

- select/plural expression
- message reference 与 term
- message attribute
- 嵌套 placeable
- Fluent 内建格式化函数

term 中未绑定的变量会传递给最终 message constructor。

## 编译期校验

宏拒绝非法 Language Identifier、fallback 缺失、parser Junk、重复定义、缺失引用、循环引用、locale key/attribute 不一致、变量集合漂移，以及生成 Rust identifier 后的冲突。

资源改动后直接编译即可验证 catalog：

```sh
cargo check -p arkit_example_i18n
```

## 手工消息

特殊场景可构造 `TypedMessage` 和 `I18nArg`，但正常 UI 优先使用宏生成函数。字符串 key 会绕过编译期拼写和参数检查。
