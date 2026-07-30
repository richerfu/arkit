export type ContentArea = "docs" | "components" | "charts";

export type NavSection = {
  id: string;
  title: string;
  description: string;
};

export type NavGroup = {
  title: string;
  sections: readonly string[];
};

export type ContentCatalog = {
  area: ContentArea;
  title: string;
  /** Route path segment for the first section (no slug). */
  indexId: string;
  groups: readonly NavGroup[];
};
