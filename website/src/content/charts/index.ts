import { markdownSection, type ContentCatalog, type MarkdownModule } from "../types";

const modules = import.meta.glob<MarkdownModule>("./*.md", { eager: true });
const section = (id: string) => markdownSection(modules, id);

export const chartCatalog: ContentCatalog = {
  area: "charts",
  title: "图表",
  groups: [
    {
      title: "通用指南",
      sections: [
        section("overview"),
        section("echarts-component"),
        section("option-data"),
        section("axes-layout"),
        section("visual-components"),
        section("interaction-action"),
        section("realtime-animation"),
        section("export-performance"),
      ],
    },
    {
      title: "直角坐标系列",
      sections: [
        section("line"),
        section("bar"),
        section("scatter"),
        section("effect-scatter"),
        section("heatmap"),
        section("candlestick"),
        section("boxplot"),
        section("pictorial-bar"),
        section("parallel"),
      ],
    },
    {
      title: "统计与极坐标",
      sections: [
        section("pie"),
        section("radar"),
        section("gauge"),
        section("funnel"),
        section("theme-river"),
      ],
    },
    {
      title: "层级与关系",
      sections: [
        section("tree"),
        section("treemap"),
        section("sunburst"),
        section("graph"),
        section("sankey"),
      ],
    },
    {
      title: "地理与扩展",
      sections: [section("map"), section("lines"), section("custom")],
    },
  ],
};
