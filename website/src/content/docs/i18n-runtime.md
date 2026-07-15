---
title: 运行时切换
---

# 运行时切换

`use_i18n_provider` 为 subtree 安装 `I18nContext`，并持有当前 locale Signal。`t!`/`I18nContext::tr` 读取该 Signal，因此切换语言只重渲染依赖翻译的 scope。

## 安装与读取

```rust
#[entry]
fn app() -> Element {
    let _provider = use_i18n_provider(
        &tr::CATALOG,
        tr::FALLBACK_LOCALE.id(),
    );
    rsx! { Page {} }
}

#[component]
fn Page() -> Element {
    let i18n = use_i18n();
    let title = t!(tr::app_title());

    rsx! {
        column {
            text { "{title}" }
            button {
                onclick: move |_| i18n.set_locale_id("en-US"),
                "English"
            }
        }
    }
}
```

Locale id 以 `Rc<str>` 共享。切换前可将用户偏好持久化，启动时把已校验的 id 传给 provider。

## Fallback

请求 locale 不存在时先选择 catalog fallback；目标 locale 缺少 message 时再尝试 fallback。宏生成 catalog 已保证 key 完整，第二层主要保护手工 catalog。

| API                                     | 失败行为                         |
| --------------------------------------- | -------------------------------- |
| `translate` / `t!` / `I18nContext::tr`  | 返回 message key，保证 UI 可显示 |
| `try_translate` / `I18nContext::try_tr` | 返回详细 `I18nError`             |

`I18nError` 区分 locale/message/value 缺失、非法资源和格式化失败。

## 缓存

每个 UI 线程按 `(Catalog address, locale index)` 缓存解析后的 `FluentBundle`。steady state 不重复解析 FTL，也不使用 process-wide mutex。

## Locale 选择

推荐顺序是用户显式设置 → 系统首选语言映射 → catalog fallback。地区变体无法精确匹配时由应用建立受控映射，例如 `zh-HK` 到 `zh-TW`，不要静默截断所有 BCP-47 tag。

## 测试

至少覆盖 fallback、参数、plural/select、attribute 和运行时往返切换。`examples/i18n` 提供中英文完整路径。
