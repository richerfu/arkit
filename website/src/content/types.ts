import type { ComponentType } from "react";

export type ContentArea = "docs" | "components" | "charts";

export type ContentSection = {
  id: string;
  title: string;
  description: string;
  Component: ComponentType;
  headings: MarkdownHeading[];
};

export type MarkdownHeading = {
  depth: number;
  slug: string;
  text: string;
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

export type MarkdownModule = {
  default: ComponentType;
  frontmatter: Record<string, unknown>;
  headings: MarkdownHeading[];
};

export function markdownSection(
  modules: Record<string, MarkdownModule>,
  id: string,
): ContentSection {
  const module = modules[`./${id}.md`];
  if (!module) throw new Error(`missing markdown module: ${id}`);
  const { title, description } = module.frontmatter;
  if (typeof title !== "string" || !title.trim()) {
    throw new Error(`missing Markdown title: ${id}`);
  }
  if (typeof description !== "string" || !description.trim()) {
    throw new Error(`missing Markdown description: ${id}`);
  }
  return { id, title, description, Component: module.default, headings: module.headings };
}

export function catalogSections(catalog: ContentCatalog) {
  return catalog.groups.flatMap((group) => group.sections);
}
