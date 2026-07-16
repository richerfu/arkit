import { renderToStaticMarkup } from "react-dom/server";

import { getContentCatalog } from "./content";
import { catalogSections, type ContentArea } from "./content/types";
import { App } from "./main";

export type StaticRoute = {
  path: string;
  title: string;
  description: string;
};

const contentAreas: ContentArea[] = ["docs", "components", "charts"];

export function getStaticRoutes(): StaticRoute[] {
  const routes: StaticRoute[] = [
    {
      path: "",
      title: "Arkit",
      description: "Arkit：面向 OpenHarmony ArkUI 的 Dioxus 原生渲染器与应用框架。",
    },
  ];

  for (const area of contentAreas) {
    const catalog = getContentCatalog(area);
    const sections = catalogSections(catalog);
    sections.forEach((section, index) => {
      routes.push({
        path: index === 0 ? `${area}/` : `${area}/${section.id}/`,
        title: `${section.title} · ${catalog.title} · Arkit`,
        description: section.description,
      });
    });
  }

  return routes;
}

export function renderStaticRoute(route: StaticRoute) {
  return renderToStaticMarkup(<App path={route.path} />);
}
