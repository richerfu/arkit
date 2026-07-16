import { ContentPage } from "./content-page";
import type { ContentArea } from "./content/types";
import { siteHref, siteRelativePath } from "./site-paths";

type SiteRoute = "home" | ContentArea;

const capabilities = [
  [
    "Dioxus 原生渲染",
    "以 VirtualDom、signals、hooks 和 rsx! 驱动 ArkUI 原生节点，不维护第二套 UI 状态树。",
  ],
  ["完整应用能力", "内置路由、国际化、动画、图表、图标、WebView 与 shadcn 风格组件库。"],
  ["原生性能路径", "事件排队、NodeAdapter 虚拟化、Drawing canvas、动画 dirty batch 与有界缓存。"],
  [
    "OpenHarmony 生命周期",
    "#[entry] 生成 N-API init/render/destroy，并接入窗口、安全区与系统返回键。",
  ],
];

const featureRows = [
  ["默认能力", "renderer、runtime、ArkUI elements、hooks、safe area、overlay、WebView"],
  ["animation", "类型化 Timeline、controls、layout/presence、drag、scroll、native lowering"],
  ["chart", "22 类原生 series、ECharts 风格 option/action/event；自动启用 animation"],
  ["router", "dioxus-router、原生 Link、系统返回键、页面转场；自动启用 animation"],
  ["i18n", "编译期 Fluent catalog、类型安全消息、响应式 locale"],
  ["icon", "内嵌 Lucide 图标、SVG raster、可配置描边与颜色"],
  ["shadcn", "主题 tokens 与 50+ 业务组件；自动启用 animation 和 icon"],
  ["full", "启用全部领域能力"],
];

type CodeTone =
  | "keyword"
  | "macro"
  | "type"
  | "function"
  | "namespace"
  | "tag"
  | "property"
  | "number"
  | "string"
  | "variable"
  | "operator";

type CodeToken = readonly [text: string, tone?: CodeTone];

const counterCode: ReadonlyArray<ReadonlyArray<CodeToken>> = [
  [["use", "keyword"], [" arkit", "namespace"], ["::prelude::"], ["*", "operator"], [";"]],
  [],
  [["#[entry]", "macro"]],
  [["fn", "keyword"], [" app", "function"], ["() -> "], ["Element", "type"], [" {"]],
  [
    ["    let", "keyword"],
    [" mut", "keyword"],
    [" count", "variable"],
    [" = "],
    ["use_signal", "function"],
    ["(|| "],
    ["0", "number"],
    [");"],
  ],
  [],
  [["    rsx!", "macro"], [" {"]],
  [["        column", "tag"], [" {"]],
  [["            percent_width", "property"], [": "], ["1.0", "number"], [","]],
  [["            percent_height", "property"], [": "], ["1.0", "number"], [","]],
  [["            align_items", "property"], [": "], ['"center"', "string"], [","]],
  [["            justify_content", "property"], [": "], ['"center"', "string"], [","]],
  [],
  [
    ["            text", "tag"],
    [" { "],
    ["font_size", "property"],
    [": "],
    ["28.0", "number"],
    [", "],
    ['"count = {count}"', "string"],
    [" }"],
  ],
  [["            button", "tag"], [" {"]],
  [
    ["                onclick", "property"],
    [": "],
    ["move", "keyword"],
    [" |_| "],
    ["count", "variable"],
    [" += ", "operator"],
    ["1", "number"],
    [","],
  ],
  [['                "increment"', "string"]],
  [["            }"]],
  [["        }"]],
  [["    }"]],
  [["}"]],
];

export function App({ path = siteRelativePath() }: { path?: string }) {
  const route = resolveRoute(path);
  return (
    <>
      <SiteHeader route={route} />
      {route === "home" ? <HomePage /> : <ContentPage area={route} path={path} />}
      <SiteFooter />
    </>
  );
}

function SiteHeader({ route }: { route: SiteRoute }) {
  return (
    <header className="site-header">
      <div className="container header-inner">
        <a className="brand" href={siteHref()}>
          <img src={siteHref("logo.svg")} alt="" />
          <span>arkit</span>
        </a>
        <div className="header-actions">
          <nav aria-label="主导航">
            <a className={route === "docs" ? "active" : ""} href={siteHref("docs/")}>
              文档
            </a>
            <a className={route === "components" ? "active" : ""} href={siteHref("components/")}>
              组件
            </a>
            <a className={route === "charts" ? "active" : ""} href={siteHref("charts/")}>
              图表
            </a>
          </nav>
          <a
            className="github-link"
            href="https://github.com/richerfu/arkit"
            target="_blank"
            rel="noreferrer"
          >
            GitHub ↗
          </a>
        </div>
      </div>
    </header>
  );
}

function HomePage() {
  return (
    <main id="top">
      <section className="hero">
        <div className="hero-glow" aria-hidden="true" />
        <div className="container hero-grid">
          <div className="hero-copy">
            <img className="hero-logo" src={siteHref("logo.svg")} alt="" />
            <p className="eyebrow">Dioxus 0.7 × OpenHarmony ArkUI</p>
            <h1>Arkit</h1>
            <p className="hero-lead">
              用 Dioxus 组件、Signals 与 Hooks 直接渲染 ArkUI， 路由、动画、图表和组件库开箱即用。
            </p>
            <div className="hero-actions">
              <a className="button primary" href={siteHref("docs/")}>
                开始使用
              </a>
              <a className="button secondary" href={siteHref("components/")}>
                浏览组件
              </a>
            </div>
          </div>
          <div className="code-window" aria-label="Arkit counter 示例">
            <div className="code-title">
              <span>counter/src/lib.rs</span>
              <span>Rust</span>
            </div>
            <HighlightedCounterCode />
          </div>
        </div>
      </section>

      <section className="section">
        <div className="container">
          <SectionTitle eyebrow="能力地图" title="一套 Dioxus 模型，覆盖完整原生应用" />
          <div className="capability-grid">
            {capabilities.map(([title, body]) => (
              <article className="panel" key={title}>
                <h3>{title}</h3>
                <p>{body}</p>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="section content-entry-section">
        <div className="container">
          <SectionTitle eyebrow="内容中心" title="按开发任务查阅，不在一条长目录里迷路" />
          <div className="content-entry-grid">
            <a className="content-entry" href={siteHref("docs/")}>
              <span>01</span>
              <h3>文档</h3>
              <p>从组件、状态与 Hooks 开始，再进入 i18n、路由、动画和平台能力。</p>
            </a>
            <a className="content-entry" href={siteHref("components/")}>
              <span>02</span>
              <h3>组件</h3>
              <p>主题、表单、内容、导航、浮层与反馈组件的独立使用手册。</p>
            </a>
            <a className="content-entry" href={siteHref("charts/")}>
              <span>03</span>
              <h3>图表</h3>
              <p>22 类原生 series、Option、交互 Action、实时更新和导出。</p>
            </a>
          </div>
        </div>
      </section>

      <section className="section architecture-band">
        <div className="container">
          <SectionTitle eyebrow="运行模型" title="Dioxus 是唯一的 UI 真相来源" />
          <div className="pipeline" aria-label="Arkit 渲染流程">
            <PipelineNode label="业务组件" detail="rsx! · signals · hooks" />
            <span>→</span>
            <PipelineNode label="VirtualDom" detail="diff · scheduler · events" />
            <span>→</span>
            <PipelineNode label="HostTree" detail="确定性 ArkUI 投影" />
            <span>→</span>
            <PipelineNode label="原生节点" detail="ArkUI · Drawing · WebView" />
          </div>
        </div>
      </section>

      <section className="section">
        <div className="container split-section">
          <div>
            <SectionTitle eyebrow="按需组合" title="小内核，领域能力显式启用" />
            <p className="section-copy">
              默认 facade 只带渲染、运行时和核心 hooks。业务按 Cargo feature
              选择领域能力，避免基础应用自动链接完整图表与组件栈。
            </p>
            <a className="text-link" href={siteHref("docs/")}>
              查看 feature 与公开 API →
            </a>
          </div>
          <div className="feature-table">
            {featureRows.map(([feature, body]) => (
              <div key={feature}>
                <code>{feature}</code>
                <span>{body}</span>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="section cta-band">
        <div className="container cta-inner">
          <div>
            <p className="eyebrow">从可运行代码开始</p>
            <h2>9 个示例覆盖从计数器到动画与图表</h2>
          </div>
          <a className="button primary" href={siteHref("docs/examples/")}>
            查看示例索引
          </a>
        </div>
      </section>
    </main>
  );
}

function PipelineNode({ label, detail }: { label: string; detail: string }) {
  return (
    <div>
      <strong>{label}</strong>
      <small>{detail}</small>
    </div>
  );
}

function HighlightedCounterCode() {
  return (
    <pre aria-label="Rust counter 源码">
      <code>
        {counterCode.map((line, lineIndex) => (
          <span className="code-line" key={lineIndex}>
            {line.map(([value, tone], tokenIndex) => (
              <span className={tone ? "syntax-" + tone : undefined} key={tokenIndex}>
                {value}
              </span>
            ))}
          </span>
        ))}
      </code>
    </pre>
  );
}

function SectionTitle({ eyebrow, title }: { eyebrow: string; title: string }) {
  return (
    <div className="section-title">
      <p className="eyebrow">{eyebrow}</p>
      <h2>{title}</h2>
    </div>
  );
}

function SiteFooter() {
  return (
    <footer className="site-footer">
      <div className="container footer-inner">
        <span>arkit · Dioxus 原生 ArkUI renderer</span>
        <span>当前文档语言：中文</span>
      </div>
    </footer>
  );
}

function resolveRoute(path: string): SiteRoute {
  const area = siteRelativePath(path).split("/")[0];
  return area === "docs" || area === "components" || area === "charts" ? area : "home";
}
