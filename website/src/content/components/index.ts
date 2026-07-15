import type { ComponentType } from "react";

import { markdownComponent, type ContentCatalog } from "../types";

const modules = import.meta.glob<{ default: ComponentType }>("./*.md");
const section = (id: string, title: string, summary: string) => ({
  id,
  title,
  summary,
  Component: markdownComponent(modules, id),
});

export const componentCatalog: ContentCatalog = {
  area: "components",
  title: "组件",
  groups: [
    {
      title: "通用指南",
      sections: [
        section("overview", "组件库介绍", "安装、导入、组件边界与完整索引。"),
        section("theme", "主题系统", "预设、明暗模式、tokens 与运行时切换。"),
        section("state-model", "状态模型", "受控、非受控、事件与表单状态。"),
        section("layout-overlay", "布局与浮层", "尺寸、SafeArea、OverlayRoot 与键盘。"),
        section("accessibility", "交互与可访问性", "触控、焦点、反馈和移动端适配。"),
      ],
    },
    {
      title: "基础组件",
      sections: [
        section("button", "Button", "按钮变体、尺寸、禁用和点击。"),
        section("text", "Text", "主题化文本层级与排版。"),
        section("badge", "Badge", "紧凑状态标签。"),
        section("label", "Label", "表单与控件标签。"),
        section("avatar", "Avatar", "头像图片与 fallback。"),
        section("separator", "Separator", "水平和垂直分隔。"),
        section("aspect-ratio", "AspectRatio", "固定媒体宽高比。"),
      ],
    },
    {
      title: "内容与反馈",
      sections: [
        section("alert", "Alert", "提示容器及标题、说明和列表。"),
        section("card", "Card", "卡片及 Header、Content、Footer。"),
        section("markdown", "Markdown", "高性能原生 CommonMark/GFM 渲染。"),
        section("skeleton", "Skeleton", "结构化加载占位。"),
        section("spinner", "Spinner", "不确定进度指示器。"),
        section("progress", "Progress", "确定性进度展示。"),
      ],
    },
    {
      title: "输入与表单",
      sections: [
        section("input", "Input", "单行受控文本输入。"),
        section("textarea", "Textarea", "多行受控文本输入。"),
        section("input-otp", "InputOtp", "验证码输入、slot 与分隔。"),
        section("checkbox", "Checkbox", "受控和非受控复选。"),
        section("switch", "Switch", "即时二元设置。"),
        section("radio-group", "RadioGroup", "单选选项组。"),
        section("slider", "Slider", "单值、范围和多 thumb 滑块。"),
        section("toggle", "Toggle", "单个 pressed 状态。"),
        section("toggle-group", "ToggleGroup", "成组互斥或多选切换。"),
        section("form", "Form", "Form 与完整 Field primitive。"),
      ],
    },
    {
      title: "日期与选择",
      sections: [
        section("calendar", "Calendar", "月视图日期选择。"),
        section("date-picker", "DatePicker", "底部面板日期选择器。"),
        section("select", "Select", "锚点下拉选择。"),
        section("combobox", "Combobox", "可检索选项选择。"),
        section("command", "Command", "命令搜索与执行列表。"),
      ],
    },
    {
      title: "布局与数据",
      sections: [
        section("accordion", "Accordion", "多项折叠内容。"),
        section("collapsible", "Collapsible", "单区域展开收起。"),
        section("tabs", "Tabs", "页内标签视图。"),
        section("carousel", "Carousel", "轮播、控制器和指示器。"),
        section("resizable", "Resizable", "双栏内容与分隔布局。"),
        section("scroll-area", "ScrollArea", "主题化滚动容器。"),
        section("table", "Table", "行列数据展示。"),
        section("chart", "Chart", "轻量主题化图表与 ChartCard。"),
      ],
    },
    {
      title: "导航组件",
      sections: [
        section("breadcrumb", "Breadcrumb", "页面层级路径。"),
        section("pagination", "Pagination", "页码和前后翻页。"),
        section("navigation-menu", "NavigationMenu", "主导航菜单。"),
        section("sidebar", "Sidebar", "侧边导航及 SidebarItem。"),
        section("bottom-navigation", "BottomNavigation", "移动端底部主导航。"),
      ],
    },
    {
      title: "浮层与菜单",
      sections: [
        section("dialog", "Dialog", "居中模态框及 Header、Footer。"),
        section("alert-dialog", "AlertDialog", "强确认模态框。"),
        section("sheet", "Sheet", "四边侧滑面板。"),
        section("drawer", "Drawer", "抽屉式面板。"),
        section("bottom-sheet", "BottomSheet", "移动端底部面板与输入框。"),
        section("popover", "Popover", "锚点交互浮层。"),
        section("hover-card", "HoverCard", "悬浮预览卡片。"),
        section("tooltip", "Tooltip", "简短说明浮层。"),
        section("floating-layer", "FloatingLayer", "底层浮层定位 primitive。"),
        section("dropdown-menu", "DropdownMenu", "触发器下拉菜单。"),
        section("context-menu", "ContextMenu", "上下文操作菜单。"),
        section("menubar", "Menubar", "多菜单命令栏。"),
        section("toast", "Toast", "单条操作反馈。"),
        section("sonner", "Sonner", "安全区感知的 Toast 队列。"),
      ],
    },
  ],
};
