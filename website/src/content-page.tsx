import { Suspense, useEffect, useRef, useState } from "react";
import { lastUpdatedByContent } from "virtual:content-last-updated";

import { getContentCatalog } from "./content";
import { catalogSections, type ContentArea } from "./content/types";
import { siteHref, siteRelativePath } from "./site-paths";

type TocHeading = { depth: number; slug: string; text: string };

export function ContentPage({ area }: { area: ContentArea }) {
  const catalog = getContentCatalog(area);
  const sections = catalogSections(catalog);
  const requestedId = resolveActiveId(area, sections[0].id);
  const activeSection = sections.find((section) => section.id === requestedId) ?? sections[0];
  const ActiveComponent = activeSection.Component;
  const activeIndex = sections.findIndex((section) => section.id === activeSection.id);
  const previousSection = activeIndex > 0 ? sections[activeIndex - 1] : undefined;
  const nextSection = activeIndex < sections.length - 1 ? sections[activeIndex + 1] : undefined;
  const markdownRef = useRef<HTMLDivElement>(null);
  const [tocHeadings, setTocHeadings] = useState<TocHeading[]>([]);
  const lastUpdated = formatLastUpdated(lastUpdatedByContent[`${area}/${activeSection.id}`]);

  useEffect(() => {
    const markdown = markdownRef.current;
    if (!markdown) return;

    const collectHeadings = () => {
      setTocHeadings(
        Array.from(markdown.querySelectorAll<HTMLHeadingElement>("h2[id], h3[id]")).map(
          (heading) => ({
            depth: Number(heading.tagName.slice(1)),
            slug: heading.id,
            text: heading.textContent?.trim() ?? "",
          }),
        ),
      );
    };

    collectHeadings();
    const observer = new MutationObserver(collectHeadings);
    observer.observe(markdown, { childList: true, subtree: true });
    return () => observer.disconnect();
  }, [activeSection.id]);

  return (
    <div className="docs-shell">
      <details className="mobile-doc-nav">
        <summary>
          {catalog.title}目录 <span aria-hidden="true">▾</span>
        </summary>
        <ContentNavigation area={area} activeId={activeSection.id} mobile />
      </details>
      <div className="docs-columns">
        <aside className="docs-sidebar">
          <div className="sticky-nav">
            <p className="nav-caption">{catalog.title}</p>
            <ContentNavigation area={area} activeId={activeSection.id} />
          </div>
        </aside>
        <main className="docs-main">
          <article className="void-md docs-markdown">
            {lastUpdated ? <p className="last-updated">最后更新：{lastUpdated}</p> : null}
            <div className="markdown-body" ref={markdownRef}>
              <Suspense fallback={<p className="content-loading">正在载入章节…</p>}>
                <ActiveComponent />
              </Suspense>
            </div>
            <nav className="doc-pager" aria-label={`${catalog.title}翻页`}>
              {previousSection ? (
                <a href={contentHref(area, previousSection.id)}>
                  <span>上一篇</span>
                  <strong>{previousSection.title}</strong>
                </a>
              ) : (
                <span />
              )}
              {nextSection ? (
                <a className="next" href={contentHref(area, nextSection.id)}>
                  <span>下一篇</span>
                  <strong>{nextSection.title}</strong>
                </a>
              ) : (
                <span />
              )}
            </nav>
          </article>
          {tocHeadings.length > 0 ? (
            <aside className="docs-toc">
              <nav className="sticky-nav" aria-label="本页目录">
                <p className="nav-caption">本页目录</p>
                {tocHeadings.map((heading) => (
                  <a
                    className={`toc-depth-${heading.depth}`}
                    href={`#${heading.slug}`}
                    key={heading.slug}
                  >
                    {heading.text}
                  </a>
                ))}
              </nav>
            </aside>
          ) : null}
        </main>
      </div>
    </div>
  );
}

function ContentNavigation({
  area,
  activeId,
  mobile = false,
}: {
  area: ContentArea;
  activeId: string;
  mobile?: boolean;
}) {
  const catalog = getContentCatalog(area);
  return (
    <nav
      className={mobile ? "doc-navigation mobile" : "doc-navigation"}
      aria-label={`${catalog.title}章节`}
    >
      {catalog.groups.map((group) => (
        <section key={group.title}>
          <p>{group.title}</p>
          {group.sections.map((section) => (
            <a
              className={section.id === activeId ? "active" : ""}
              href={contentHref(area, section.id)}
              key={section.id}
            >
              {section.title}
            </a>
          ))}
        </section>
      ))}
    </nav>
  );
}

function resolveActiveId(area: ContentArea, defaultId: string) {
  const parts = siteRelativePath().split("/").filter(Boolean);
  return parts[0] === area && parts[1] ? parts[1] : defaultId;
}

function contentHref(area: ContentArea, id: string) {
  const firstSection = catalogSections(getContentCatalog(area))[0];
  return siteHref(id === firstSection.id ? `${area}/` : `${area}/${id}/`);
}

function formatLastUpdated(value: string | undefined) {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.valueOf())) return "";
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "long",
    day: "numeric",
  }).format(date);
}
