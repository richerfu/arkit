---
layout: home

hero:
  name: Arkit
  text: Dioxus 的 OpenHarmony ArkUI 原生渲染器
  tagline: 使用标准 Dioxus 组件、rsx、signals、hooks、异步资源和路由编写 OpenHarmony 原生应用。
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 示例
      link: /examples/

features:
  - title: Dioxus 原生模型
    details: VirtualDom 负责组件、hooks、diff 和任务调度，不维护第二套 Element 或 update runtime。
  - title: ArkUI HostTree
    details: renderer 保留 Dioxus text、placeholder 和 ElementId 语义，再投影到 ArkUI native tree。
  - title: 响应式状态
    details: 使用 use_signal、use_memo、use_effect 和 context 组织状态与副作用。
  - title: 异步任务
    details: 使用 use_resource、use_future、use_coroutine；任务完成会通过 Dioxus scheduler 唤醒 ArkUI。
  - title: 类型化路由
    details: 直接复用 dioxus-router，并提供 ArkUI 原生 Link 与返回键桥接。
  - title: 原生能力
    details: 提供布局测量、overlay、虚拟列表、动画、ECharts-like 原生图表、图标、i18n、shadcn 和 WebView。
---
