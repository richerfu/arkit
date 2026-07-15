import { chartCatalog } from "./charts";
import { componentCatalog } from "./components";
import { docsCatalog } from "./docs";
import type { ContentArea, ContentCatalog } from "./types";

const catalogs: Record<ContentArea, ContentCatalog> = {
  docs: docsCatalog,
  components: componentCatalog,
  charts: chartCatalog,
};

export function getContentCatalog(area: ContentArea) {
  return catalogs[area];
}
