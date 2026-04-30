---
layout: home

hero:
  name: Arkit
  text: Rust ArkUI 应用框架
  tagline: 用 Rust 编写 OpenHarmony 原生应用。提供声明式 UI、消息驱动状态、异步任务、路由、国际化、组件库和 WebView。
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 示例
      link: /examples/

features:
  - title: 声明式 UI
    details: 用 Rust 函数描述页面，State 变化后重新生成 Element。
  - title: 消息驱动
    details: 事件进入 Message，update 统一处理状态变化。
  - title: 异步任务
    details: 用 Task::perform 执行异步操作，结果返回 Message。
  - title: 页面路由
    details: Router 管理 push、replace、reset、back、参数、嵌套路由和守卫。
  - title: 国际化
    details: 从 .ftl 文件生成类型安全的多语言 API。
  - title: 组件体系
    details: 提供基础 ArkUI 组件，并可使用 arkit_shadcn 组件库。
---
