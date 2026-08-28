//! Demo registry — the single source of truth for the home page list and the
//! `/demo/:slug` dispatcher.

#[derive(Clone, Copy, PartialEq)]
pub struct DemoSpec {
    pub slug: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

pub struct DemoGroup {
    pub title: &'static str,
    pub demos: &'static [DemoSpec],
}

pub const DEMO_GROUPS: &[DemoGroup] = &[
    DemoGroup {
        title: "UI 组件与可视化",
        demos: &[
            DemoSpec {
                slug: "shadcn_showcase",
                name: "shadcn 组件库",
                description: "50+ 组件全览:按钮、表单、日历、弹层、表格",
            },
            DemoSpec {
                slug: "chart",
                name: "图表",
                description: "ECharts 兼容 JSON 图表,支持多种系列",
            },
            DemoSpec {
                slug: "canvas",
                name: "Canvas 2D",
                description: "W3C 风格 Canvas 2D API 与绘图示例",
            },
            DemoSpec {
                slug: "lottie",
                name: "Lottie 动画",
                description: "高性能 Lottie 渲染,本地与网络资源",
            },
            DemoSpec {
                slug: "animation",
                name: "动画引擎",
                description: "缓动、时间线、交互与生命周期动画实验室",
            },
            DemoSpec {
                slug: "complex_cases",
                name: "虚拟化列表",
                description: "List / Grid / WaterFlow 按需虚拟渲染",
            },
        ],
    },
    DemoGroup {
        title: "框架能力",
        demos: &[
            DemoSpec {
                slug: "counter",
                name: "计数器",
                description: "最简示例:rsx! 与 use_signal 信号驱动",
            },
            DemoSpec {
                slug: "async_task",
                name: "异步任务",
                description: "use_resource 异步加载与 Tokio 运行时",
            },
            DemoSpec {
                slug: "router",
                name: "路由",
                description: "枚举路由、全屏页面切换与返回恢复",
            },
            DemoSpec {
                slug: "i18n",
                name: "国际化",
                description: "类型安全 t! 宏与运行时语言切换",
            },
        ],
    },
    DemoGroup {
        title: "系统能力",
        demos: &[
            DemoSpec {
                slug: "camera",
                name: "相机",
                description: "CameraKit 原生预览、拍照与扫码",
            },
            DemoSpec {
                slug: "barcode",
                name: "条码",
                description: "条码 / QR 生成、扫描与 PNG 导出",
            },
            DemoSpec {
                slug: "terminal",
                name: "终端",
                description: "VT 终端组件,本地 Shell 与 SSH",
            },
            DemoSpec {
                slug: "webview",
                name: "WebView",
                description: "插件化 WebView,由 dioxus 布局定位",
            },
        ],
    },
];

/// Resolve a demo spec by slug, used by the demo page to validate the route.
pub fn find_demo(slug: &str) -> Option<&'static DemoSpec> {
    DEMO_GROUPS
        .iter()
        .flat_map(|group| group.demos.iter())
        .find(|spec| spec.slug == slug)
}
