import { markdownSection, type ContentCatalog, type MarkdownModule } from "../types";

const modules = import.meta.glob<MarkdownModule>("./*.md", { eager: true });
const section = (id: string) => markdownSection(modules, id);

export const componentCatalog: ContentCatalog = {
  area: "components",
  title: "组件",
  groups: [
    {
      title: "通用指南",
      sections: [
        section("overview"),
        section("theme"),
        section("state-model"),
        section("layout-overlay"),
        section("accessibility"),
      ],
    },
    {
      title: "基础组件",
      sections: [
        section("button"),
        section("text"),
        section("badge"),
        section("label"),
        section("avatar"),
        section("separator"),
        section("aspect-ratio"),
      ],
    },
    {
      title: "内容与反馈",
      sections: [
        section("alert"),
        section("card"),
        section("barcode"),
        section("code"),
        section("markdown"),
        section("skeleton"),
        section("spinner"),
        section("progress"),
      ],
    },
    {
      title: "输入与表单",
      sections: [
        section("input"),
        section("textarea"),
        section("input-otp"),
        section("checkbox"),
        section("switch"),
        section("radio-group"),
        section("slider"),
        section("toggle"),
        section("toggle-group"),
        section("form"),
      ],
    },
    {
      title: "日期与选择",
      sections: [
        section("calendar"),
        section("date-picker"),
        section("select"),
        section("combobox"),
        section("command"),
      ],
    },
    {
      title: "布局与数据",
      sections: [
        section("accordion"),
        section("collapsible"),
        section("tabs"),
        section("carousel"),
        section("resizable"),
        section("scroll-area"),
        section("table"),
        section("chart"),
      ],
    },
    {
      title: "导航组件",
      sections: [
        section("breadcrumb"),
        section("pagination"),
        section("navigation-menu"),
        section("sidebar"),
        section("bottom-navigation"),
      ],
    },
    {
      title: "浮层与菜单",
      sections: [
        section("dialog"),
        section("alert-dialog"),
        section("sheet"),
        section("drawer"),
        section("bottom-sheet"),
        section("popover"),
        section("hover-card"),
        section("tooltip"),
        section("floating-layer"),
        section("dropdown-menu"),
        section("context-menu"),
        section("menubar"),
        section("toast"),
        section("sonner"),
      ],
    },
  ],
};
