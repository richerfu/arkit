# 示例索引

`examples/*` 是当前框架能力的活文档。新增功能时优先补一个最小示例，再补复杂组合示例。

| 示例 | 说明 |
| --- | --- |
| `counter` | 最小 State / Message / update / view。 |
| `async_task` | `Task::perform` 异步任务。 |
| `router` | 路由注册、页面切换、404 和导航消息。 |
| `i18n` | typed `.ftl` 国际化。 |
| `shadcn_showcase` | shadcn 组件库展示。 |
| `webview` | feature gated WebView。 |
| `complex_cases` | 复杂组合场景。 |

## 示例开发原则

- 示例要能独立构建。
- 示例代码应展示推荐 API。
- 与设备原生行为相关的能力必须有示例。
- 文档中的代码片段应与示例保持一致。
