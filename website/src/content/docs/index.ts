import { markdownSection, type ContentCatalog, type MarkdownModule } from "../types";

const modules = import.meta.glob<MarkdownModule>("./*.md", { eager: true });
const section = (id: string) => markdownSection(modules, id);

export const docsCatalog: ContentCatalog = {
  area: "docs",
  title: "文档",
  groups: [
    {
      title: "开始使用",
      sections: [section("getting-started"), section("arkts-integration")],
    },
    {
      title: "基础开发模式",
      sections: [
        section("application-model"),
        section("elements-layout"),
        section("styling"),
        section("events"),
        section("state-management"),
        section("hooks-lifecycle"),
        section("context"),
        section("async-runtime"),
      ],
    },
    {
      title: "基础能力",
      sections: [
        section("native-hooks"),
        section("virtualization"),
        section("window-metrics"),
        section("safe-area-overlay"),
        section("camera"),
        section("webview"),
      ],
    },
    {
      title: "国际化 i18n",
      sections: [section("i18n"), section("i18n-catalog"), section("i18n-runtime")],
    },
    {
      title: "路由",
      sections: [
        section("router"),
        section("router-navigation"),
        section("router-nested"),
        section("router-transitions"),
      ],
    },
    {
      title: "动画",
      sections: [
        section("animation"),
        section("animation-values"),
        section("animation-timeline"),
        section("animation-controls"),
        section("animation-orchestration"),
        section("animation-layout"),
        section("animation-interactions"),
        section("animation-backends"),
      ],
    },
    {
      title: "图标",
      sections: [section("icons"), section("icon-rendering")],
    },
    {
      title: "参考",
      sections: [section("architecture"), section("examples")],
    },
  ],
};
