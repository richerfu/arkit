import { lazy, type ComponentType, type LazyExoticComponent } from "react";

export type ContentArea = "docs" | "components" | "charts";

export type ContentSection = {
  id: string;
  title: string;
  summary: string;
  Component: LazyExoticComponent<ComponentType>;
};

export type ContentGroup = {
  title: string;
  sections: ContentSection[];
};

export type ContentCatalog = {
  area: ContentArea;
  title: string;
  groups: ContentGroup[];
};

type MarkdownModule = { default: ComponentType };

export function markdownComponent(
  modules: Record<string, () => Promise<MarkdownModule>>,
  id: string,
) {
  const load = modules[`./${id}.md`];
  if (!load) throw new Error(`missing markdown module: ${id}`);
  return lazy(load);
}

export function catalogSections(catalog: ContentCatalog) {
  return catalog.groups.flatMap((group) => group.sections);
}
