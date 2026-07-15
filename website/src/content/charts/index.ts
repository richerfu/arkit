import type { ComponentType } from "react";

import { markdownComponent, type ContentCatalog } from "../types";

const modules = import.meta.glob<{ default: ComponentType }>("./*.md");
const section = (id: string, title: string, summary: string) => ({
  id,
  title,
  summary,
  Component: markdownComponent(modules, id),
});

export const chartCatalog: ContentCatalog = {
  area: "charts",
  title: "图表",
  groups: [
    {
      title: "通用指南",
      sections: [
        section("overview", "图表库介绍", "原生 renderer、支持范围和阅读路径。"),
        section("echarts-component", "ECharts 组件", "Props、尺寸、事件与 Controller 绑定。"),
        section("option-data", "Option 与数据", "typed builder、JSON、dataset 与数据形状。"),
        section("axes-layout", "坐标系与布局", "Axis、Grid、绘图区与坐标转换。"),
        section(
          "visual-components",
          "视觉组件与样式",
          "Legend、Tooltip、DataZoom、VisualMap 与状态样式。",
        ),
        section("interaction-action", "事件与 Action", "Controller、选择、缩放与事件。"),
        section("realtime-animation", "实时更新与动画", "受控更新、appendData 与统一时钟。"),
        section("export-performance", "导出与性能", "图片导出、限制、诊断和优化。"),
      ],
    },
    {
      title: "直角坐标系列",
      sections: [
        section("line", "Line 折线图", "连续趋势、平滑、区域和标记。"),
        section("bar", "Bar 柱状图", "离散比较、堆叠和柱宽。"),
        section("scatter", "Scatter 散点图", "二维/多维点和视觉映射。"),
        section("effect-scatter", "EffectScatter 涟漪散点", "重点点位和涟漪效果。"),
        section("heatmap", "Heatmap 热力图", "x/y/value 与 VisualMap。"),
        section("candlestick", "Candlestick K 线", "OHLC 数据和金融样式。"),
        section("boxplot", "Boxplot 箱线图", "五数摘要和异常点。"),
        section("pictorial-bar", "PictorialBar 象形柱", "Symbol 重复和裁切。"),
        section("parallel", "Parallel 平行坐标", "多维数据和 Brush。"),
      ],
    },
    {
      title: "统计与极坐标",
      sections: [
        section("pie", "Pie 饼图", "扇区、环图和 named data。"),
        section("radar", "Radar 雷达图", "Indicator 与多维对比。"),
        section("gauge", "Gauge 仪表盘", "单值、区间和进度。"),
        section("funnel", "Funnel 漏斗图", "阶段顺序和转化。"),
        section("theme-river", "ThemeRiver 主题河流", "时间、数值和主题流。"),
      ],
    },
    {
      title: "层级与关系",
      sections: [
        section("tree", "Tree 树图", "层级节点、边和展开。"),
        section("treemap", "Treemap 矩形树图", "面积编码与下钻。"),
        section("sunburst", "Sunburst 旭日图", "同心层级数据。"),
        section("graph", "Graph 关系图", "节点、连线和布局。"),
        section("sankey", "Sankey 桑基图", "流量节点与连接。"),
      ],
    },
    {
      title: "地理与扩展",
      sections: [
        section("map", "Map 地图", "GeoJSON、区域数据与注册。"),
        section("lines", "Lines 路径图", "路径、迁徙和增量数据。"),
        section("custom", "Custom 自定义系列", "Custom renderer 与热路径约束。"),
      ],
    },
  ],
};
