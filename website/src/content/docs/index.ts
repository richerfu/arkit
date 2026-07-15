import type { ComponentType } from "react";

import { markdownComponent, type ContentCatalog } from "../types";

const modules = import.meta.glob<{ default: ComponentType }>("./*.md");
const section = (id: string, title: string, summary: string) => ({
  id,
  title,
  summary,
  Component: markdownComponent(modules, id),
});

export const docsCatalog: ContentCatalog = {
  area: "docs",
  title: "文档",
  groups: [
    {
      title: "开始使用",
      sections: [
        section("getting-started", "安装与第一个应用", "创建 crate、声明入口并完成构建。"),
        section("arkts-integration", "ArkTS 工程接入", "NativeAbility、XComponent 与生命周期。"),
      ],
    },
    {
      title: "基础开发模式",
      sections: [
        section("application-model", "组件与 RSX", "组件边界、Props、children 与组合模式。"),
        section("elements-layout", "元素与布局", "ArkUI 元素、尺寸、Flex 与滚动布局。"),
        section("styling", "属性与样式", "类型约定、颜色、单位与条件样式。"),
        section("events", "事件处理", "事件 payload、闭包捕获与默认行为。"),
        section("state-management", "状态管理", "Signal、派生状态、共享状态与更新原则。"),
        section("hooks-lifecycle", "Hooks 与生命周期", "hook 规则、effect、memo 与清理。"),
        section("context", "上下文与 Provider", "跨层依赖、provider 所有权与作用域。"),
        section("async-runtime", "异步任务", "resource、future、Tokio 与取消语义。"),
      ],
    },
    {
      title: "基础能力",
      sections: [
        section("native-hooks", "原生节点 Hooks", "节点句柄、布局观测与 UI-loop handoff。"),
        section("virtualization", "虚拟列表", "List、Grid、WaterFlow 的 NodeAdapter。"),
        section("window-metrics", "窗口与尺寸", "窗口尺寸、密度、方向和响应式布局。"),
        section("safe-area-overlay", "安全区与浮层", "SafeArea、OverlayRoot 与层级管理。"),
        section("webview", "嵌入 WebView", "原生挂载、导航、消息与资源清理。"),
      ],
    },
    {
      title: "国际化 i18n",
      sections: [
        section("i18n", "国际化概览", "Fluent catalog、locale 与类型安全消息。"),
        section("i18n-catalog", "资源与消息", "FTL 资源、参数、选择器和编译期校验。"),
        section("i18n-runtime", "运行时切换", "Provider、语言回退与响应式刷新。"),
      ],
    },
    {
      title: "路由",
      sections: [
        section("router", "路由概览", "类型化 Route 与 RouterProvider。"),
        section("router-navigation", "导航与历史栈", "Link、push、replace、back 与系统返回键。"),
        section("router-nested", "嵌套路由", "Outlet、参数、查询串与页面壳。"),
        section("router-transitions", "页面转场", "转场配置、生命周期与返回方向。"),
      ],
    },
    {
      title: "动画",
      sections: [
        section("animation", "动画概览", "统一时钟、target、timeline 与渲染路径。"),
        section("animation-values", "属性与关键帧", "类型化属性、easing 与 composition。"),
        section("animation-timeline", "Timeline 编排", "位置、label、call、barrier 与嵌套。"),
        section("animation-controls", "播放控制", "play、seek、reverse、cancel 与快照。"),
        section("animation-orchestration", "Stagger 与 Animatable", "分布延迟、重定向和 scope。"),
        section("animation-layout", "Layout 与 Presence", "FLIP、进退场和重排。"),
        section("animation-interactions", "Drag 与 Scroll", "手势联动、惯性、阈值与同步。"),
        section("animation-backends", "后端与性能", "sampled/native lowering、回退与诊断。"),
      ],
    },
    {
      title: "图标",
      sections: [
        section("icons", "图标概览", "Lucide 图标目录与 feature 接入。"),
        section("icon-rendering", "渲染与缓存", "尺寸、颜色、描边、raster 与缓存。"),
      ],
    },
    {
      title: "参考",
      sections: [
        section("architecture", "架构与 crate 边界", "运行时、渲染器与领域 crate 职责。"),
        section("examples", "示例索引", "workspace 示例、覆盖范围与验证命令。"),
      ],
    },
  ],
};
