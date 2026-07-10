# 示例

| 示例 | 重点 |
| --- | --- |
| `counter` | `use_signal`、`rsx!`、native click event |
| `async_task` | `use_resource`、Tokio、scheduler wake |
| `chart` | ECharts-like typed/JSON option、多图表、signal 实时更新、点击命中 |
| `complex_cases` | ArkUI NodeAdapter 虚拟列表/网格 |
| `router` | Routable enum、Router、ArkUI Link、route transition |
| `i18n` | locale context、类型安全翻译 |
| `shadcn_showcase` | Dioxus 组件、theme signal、overlay、native gesture |
| `webview` | layout/native-node hook 与嵌入 WebView |

所有示例都是 workspace member。OpenHarmony 验证必须进入对应示例目录执行 `ohrs build --arch aarch`；host `cargo check` 只能作为辅助诊断，不能替代目标平台构建。
