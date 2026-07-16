/// <reference types="vite/client" />

declare module "*.md" {
  import type { ComponentType } from "react";

  export const headings: Array<{
    depth: number;
    slug: string;
    text: string;
  }>;
  export const frontmatter: Record<string, unknown>;

  const MarkdownComponent: ComponentType;
  export default MarkdownComponent;
}

declare module "virtual:content-last-updated" {
  export const lastUpdatedByContent: Record<string, string>;
}
