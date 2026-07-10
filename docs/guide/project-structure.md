# 项目组织

推荐按 Dioxus component/domain 组织，而不是按 State/Message/update 分层：

```text
src/
  lib.rs          # #[entry] 与顶层 providers/router
  routes.rs       # Routable enum
  pages/
    home.rs       # #[component] Home
    settings.rs   # #[component] Settings
  components/
    user_card.rs  # 可复用 Dioxus component
  services/
    user.rs       # async domain operations
```

规则：

- 含 hooks 的函数必须是 hook（`use_*`）或 Dioxus component。
- 页面状态尽量留在拥有它的 component；真正跨树的数据才放 context。
- async I/O 放 service，component 用 `use_resource` 消费。
- ArkUI raw node 操作集中在 hook/adapter 边界，不散落到业务组件。
- 顶层 `#[entry]` 只安装 providers、router 和应用 layout。
