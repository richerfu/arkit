import type { ContentArea, ContentCatalog, NavGroup } from "./types";

const docsGroups: readonly NavGroup[] = [
  {
    title: "开始使用",
    sections: ["getting-started", "arkts-integration"],
  },
  {
    title: "基础开发模式",
    sections: [
      "application-model",
      "elements-layout",
      "styling",
      "events",
      "state-management",
      "hooks-lifecycle",
      "context",
      "async-runtime",
    ],
  },
  {
    title: "基础能力",
    sections: [
      "native-hooks",
      "virtualization",
      "window-metrics",
      "safe-area-overlay",
      "canvas",
      "camera",
      "lottie",
      "terminal",
      "webview",
    ],
  },
  {
    title: "国际化 i18n",
    sections: ["i18n", "i18n-catalog", "i18n-runtime"],
  },
  {
    title: "路由",
    sections: ["router", "router-navigation", "router-nested", "router-transitions"],
  },
  {
    title: "动画",
    sections: [
      "animation",
      "animation-values",
      "animation-timeline",
      "animation-controls",
      "animation-orchestration",
      "animation-layout",
      "animation-interactions",
      "animation-backends",
    ],
  },
  {
    title: "图标",
    sections: ["icons", "icon-rendering"],
  },
  {
    title: "参考",
    sections: ["architecture", "examples"],
  },
];

const componentGroups: readonly NavGroup[] = [
  {
    title: "通用指南",
    sections: ["overview", "theme", "state-model", "layout-overlay", "accessibility"],
  },
  {
    title: "基础组件",
    sections: ["button", "text", "badge", "label", "avatar", "separator", "aspect-ratio"],
  },
  {
    title: "内容与反馈",
    sections: [
      "alert",
      "card",
      "barcode",
      "code",
      "markdown",
      "watermark",
      "skeleton",
      "spinner",
      "progress",
    ],
  },
  {
    title: "输入与表单",
    sections: [
      "input",
      "textarea",
      "input-otp",
      "checkbox",
      "switch",
      "radio-group",
      "slider",
      "toggle",
      "toggle-group",
      "form",
    ],
  },
  {
    title: "日期与选择",
    sections: ["calendar", "date-picker", "select", "combobox", "command"],
  },
  {
    title: "布局与数据",
    sections: [
      "accordion",
      "collapsible",
      "tabs",
      "carousel",
      "resizable",
      "scroll-area",
      "table",
      "chart",
    ],
  },
  {
    title: "导航组件",
    sections: ["breadcrumb", "pagination", "navigation-menu", "sidebar", "bottom-navigation"],
  },
  {
    title: "浮层与菜单",
    sections: [
      "guide",
      "dialog",
      "alert-dialog",
      "sheet",
      "drawer",
      "bottom-sheet",
      "popover",
      "hover-card",
      "tooltip",
      "floating-layer",
      "dropdown-menu",
      "context-menu",
      "menubar",
      "toast",
      "sonner",
    ],
  },
];

const chartGroups: readonly NavGroup[] = [
  {
    title: "通用指南",
    sections: [
      "overview",
      "echarts-component",
      "option-data",
      "axes-layout",
      "visual-components",
      "interaction-action",
      "realtime-animation",
      "export-performance",
    ],
  },
  {
    title: "直角坐标系列",
    sections: [
      "line",
      "bar",
      "scatter",
      "effect-scatter",
      "heatmap",
      "candlestick",
      "boxplot",
      "pictorial-bar",
      "parallel",
    ],
  },
  {
    title: "统计与极坐标",
    sections: ["pie", "radar", "gauge", "funnel", "theme-river"],
  },
  {
    title: "层级与关系",
    sections: ["tree", "treemap", "sunburst", "graph", "sankey"],
  },
  {
    title: "地理与扩展",
    sections: ["map", "lines", "custom"],
  },
];

const catalogs: Record<ContentArea, ContentCatalog> = {
  docs: {
    area: "docs",
    title: "文档",
    indexId: "getting-started",
    groups: docsGroups,
  },
  components: {
    area: "components",
    title: "组件",
    indexId: "overview",
    groups: componentGroups,
  },
  charts: {
    area: "charts",
    title: "图表",
    indexId: "overview",
    groups: chartGroups,
  },
};

export function getContentCatalog(area: ContentArea): ContentCatalog {
  return catalogs[area];
}

export function catalogSectionIds(catalog: ContentCatalog): string[] {
  return catalog.groups.flatMap((group) => [...group.sections]);
}

export function contentHref(area: ContentArea, id: string, base = import.meta.env.BASE_URL) {
  const catalog = getContentCatalog(area);
  const suffix = id === catalog.indexId ? `${area}/` : `${area}/${id}/`;
  return joinBase(base, suffix);
}

export function joinBase(base: string, path = "") {
  const normalizedBase = base.endsWith("/") ? base : `${base}/`;
  return `${normalizedBase}${path.replace(/^\/+/, "")}`;
}

export const contentAreas: ContentArea[] = ["docs", "components", "charts"];
