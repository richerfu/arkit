# 能力总览

| 能力 | 入口 API | 使用场景 |
| --- | --- | --- |
| 应用入口 | `#[entry]`、`application` | 创建应用。 |
| ArkTS 挂载 | `init`、`render`、`destroy`、`ContentSlot` | 在 ArkTS 页面中显示 Rust UI。 |
| 页面状态 | `State`、`Message`、`update`、`view` | 管理业务状态和 UI。 |
| 异步任务 | `Task` | 接口请求、加载数据、提交表单。 |
| 路由 | `Router`、`Routes`、`RouterOutlet` | 多页面、返回栈、详情页参数。 |
| 国际化 | `arkit::i18n::i18n!` | 多语言文案。 |
| 基础组件 | `text`、`button`、`list`、`grid` | 编写原生 ArkUI 页面。 |
| 组件库 | `arkit_shadcn` | 按统一设计风格构建业务 UI。 |
| WebView | `web_view`、`WebViewController` | 承载 H5 或网页。 |

推荐接入顺序：

1. Rust 层用 `#[entry]` 定义应用。
2. ArkTS 层用 `ContentSlot` 挂载 Rust UI。
3. 需要请求接口时使用 `Task`。
4. 页面变多时使用 Router。
5. 需要多语言时使用 i18n。
6. UI 重复后抽业务组件或使用 `arkit_shadcn`。
7. 需要承载网页时启用 WebView。
