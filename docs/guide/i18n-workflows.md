# 国际化

`arkit_i18n` 在编译期解析 Fluent 资源并生成类型安全消息构造器，在运行时用 Dioxus context 保存 active locale。使用 facade 时需要启用 `arkit` 的 `i18n` feature。

```rust
use arkit::prelude::*;

arkit::i18n! {
    pub mod tr {
        path: "locales",
        fallback: "zh-CN",
        locales: ["zh-CN", "en-US"],
    }
}

#[entry]
fn app() -> Element {
    let _i18n_provider = use_i18n_provider(&tr::CATALOG, tr::FALLBACK_LOCALE.id());
    let i18n = use_i18n();
    let title = t!(tr::app_title());

    rsx! {
        button {
            onclick: move |_| i18n.set_locale_id("en-US"),
            "{title}"
        }
    }
}
```

读取 `t!`/`I18nContext::tr` 会订阅 locale signal；切换 locale 后，仅依赖翻译的 Dioxus scope 重新渲染。locale id 在 context 内以 `Rc<str>` 共享，读取不会复制整段字符串。

## Fluent 能力

资源不是简单的 `key = value` 替换器。运行时使用 Fluent bundle，支持 select/plural、message reference、term、attribute、内建格式化函数和嵌套 placeable。例如：

```ftl
-brand = Arkit

welcome = Welcome to { -brand }, {$name}.
inbox-count = { $count ->
    [one] One message
   *[other] { $count } messages
}
account-button =
    .label = Open account
    .hint = Signed in as {$name}
```

宏会生成 `tr::welcome(name)`、`tr::inbox_count(count)`、`tr::account_button_label()` 和 `tr::account_button_hint(name)`。term 是可复用的内部定义，不单独生成公开构造器；它引用的未绑定变量会传递到最终消息函数。

编译期会验证：

- locale id 符合 Unicode Language Identifier；fallback 必须存在于 locale 列表。
- 每个 `.ftl` 都能被 Fluent parser 完整解析，Junk、重复定义和非法引用直接编译失败。
- 所有 locale 的公开 message/attribute key 集合完全相同。
- 每个 key 在解析 message/term reference 后需要的变量集合完全相同。
- 缺失引用和循环引用直接编译失败。
- locale variant、消息函数和参数转换成 Rust identifier 后不得冲突。

因此语言文件漂移不会延迟到点击某个页面后才暴露。

## Runtime 与错误

每个 UI 线程按 `(Catalog, locale)` 缓存已解析的 Fluent bundle；steady-state 翻译不重复解析 `.ftl`。消息参数使用小对象内联存储，常见的 0–2 参数路径不分配参数 `Vec`。bundle 是 UI-thread local，避免为非 `Send` 的 Fluent formatter 引入全局 mutex。

`translate`/`t!` 在错误时返回消息 key，适合 UI 的容错展示；需要诊断时使用 `try_translate` 或 `I18nContext::try_tr` 获取 `I18nError`。请求的 locale 不存在时先使用 catalog fallback；目标 locale 缺少消息时也会尝试 fallback。由 `i18n!` 生成的 catalog 已在编译期保证 key 完整，这个运行时分支主要服务手工构造 catalog 和防御性容错。

完整可运行代码见 `examples/i18n`。该 example 通过 `framework = { package = "arkit", ... }` 的 Cargo rename 使用 `#[entry]` 和 `framework::i18n!`，同时作为公开 proc macro 不硬编码 crate 名的编译契约。
