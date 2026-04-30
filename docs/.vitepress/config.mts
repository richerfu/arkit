import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Arkit',
  description: 'ArkUI framework for OpenHarmony',
  lang: 'zh-CN',
  cleanUrls: true,
  themeConfig: {
    logo: '/logo.svg',
    nav: [
      { text: '开始', link: '/guide/getting-started' },
      { text: '能力', link: '/guide/application-model' },
      { text: '示例', link: '/examples/' }
    ],
    sidebar: [
      {
        text: '开始使用',
        items: [
          { text: '项目概览', link: '/' },
          { text: '安装与第一个页面', link: '/guide/getting-started' },
          { text: '应用模型', link: '/guide/application-model' },
          { text: 'ArkTS 接入', link: '/guide/arkts-integration' },
          { text: '项目组织', link: '/guide/project-structure' },
          { text: '业务功能开发', link: '/guide/first-feature' }
        ]
      },
      {
        text: '框架能力',
        items: [
          { text: '异步任务与接口请求', link: '/guide/async-workflows' },
          { text: '页面路由', link: '/guide/router-workflows' },
          { text: '国际化', link: '/guide/i18n-workflows' },
          { text: '业务 UI 与组件库', link: '/guide/ui-workflows' },
          { text: 'WebView', link: '/guide/webview-workflows' },
          { text: '能力总览', link: '/guide/architecture' }
        ]
      },
      {
        text: '示例',
        items: [
          { text: '示例索引', link: '/examples/' },
          { text: 'counter', link: '/examples/counter' },
          { text: 'async_task', link: '/examples/async-task' },
          { text: 'router', link: '/examples/router' },
          { text: 'i18n', link: '/examples/i18n' },
          { text: 'shadcn_showcase', link: '/examples/shadcn-showcase' },
          { text: 'webview', link: '/examples/webview' }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/ohos-rs' }
    ],
    search: {
      provider: 'local'
    },
    outline: {
      level: [2, 3],
      label: '本页目录'
    },
    docFooter: {
      prev: '上一页',
      next: '下一页'
    },
    darkModeSwitchLabel: '外观',
    sidebarMenuLabel: '菜单',
    returnToTopLabel: '返回顶部',
    lastUpdatedText: '最后更新'
  },
  lastUpdated: true
})
