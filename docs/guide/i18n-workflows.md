# 国际化

`arkit_i18n` 用 Dioxus context 保存 active locale，并从 Fluent 文件生成类型安全消息 API。

```rust
arkit_i18n::i18n! {
    pub mod tr {
        path: "locales",
        fallback: "zh-CN",
        locales: ["zh-CN", "en-US"],
    }
}

#[entry]
fn app() -> Element {
    let _ = use_i18n_provider(&tr::CATALOG, tr::FALLBACK_LOCALE.id());
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

读取翻译会订阅 locale signal；切换 locale 后相关组件自动重渲染。完整代码见 `examples/i18n`。
