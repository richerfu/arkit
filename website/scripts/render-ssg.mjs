import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";

import { getStaticRoutes, renderStaticRoute } from "../.ssg/ssg.js";

const rootDir = resolve(import.meta.dirname, "..");
const distDir = resolve(rootDir, "dist");
const manifestPath = resolve(distDir, ".vite/manifest.json");
const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
const entry = manifest["src/client.ts"] ?? Object.values(manifest).find((item) => item.isEntry);

if (!entry) throw new Error("missing production client entry in Vite manifest");
if (entry.dynamicImports?.length) {
  throw new Error(`production client contains unexpected dynamic imports: ${entry.dynamicImports}`);
}

const basePath = normalizeBasePath(process.env.SITE_BASE_PATH);
const cssFiles = collectCss(manifest, entry);
const preloadFiles = collectImports(manifest, entry).map((item) => item.file);
const routes = getStaticRoutes();
const paths = new Set();

for (const route of routes) {
  if (paths.has(route.path)) throw new Error(`duplicate static route: ${route.path}`);
  paths.add(route.path);

  const outputPath = resolve(distDir, route.path, "index.html");
  const html = renderDocument(route, renderStaticRoute(route));
  await mkdir(dirname(outputPath), { recursive: true });
  await writeFile(outputPath, html);
}

await rm(resolve(distDir, ".vite"), { recursive: true, force: true });
await rm(resolve(rootDir, ".ssg"), { recursive: true, force: true });

console.log(`prerendered ${routes.length} static routes`);

function renderDocument(route, body) {
  const styles = cssFiles
    .map((file) => `    <link rel="stylesheet" href="${assetHref(file)}" />`)
    .join("\n");
  const preloads = preloadFiles
    .map((file) => `    <link rel="modulepreload" href="${assetHref(file)}" />`)
    .join("\n");
  const headAssets = [styles, preloads].filter(Boolean).join("\n");

  return `<!doctype html>
<html lang="zh-CN" data-theme="dark">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="description" content="${escapeHtml(route.description)}" />
    <link rel="icon" type="image/svg+xml" href="${assetHref("logo.svg")}" />
    <title>${escapeHtml(route.title)}</title>
${headAssets}
  </head>
  <body>
    <div id="app">${body}</div>
    <script type="module" src="${assetHref(entry.file)}"></script>
  </body>
</html>
`;
}

function collectCss(allEntries, rootEntry) {
  const css = new Set();
  for (const item of [rootEntry, ...collectImports(allEntries, rootEntry)]) {
    for (const file of item.css ?? []) css.add(file);
  }
  return [...css];
}

function collectImports(allEntries, rootEntry) {
  const imports = [];
  const visited = new Set();

  function visit(entryItem) {
    for (const key of entryItem.imports ?? []) {
      if (visited.has(key)) continue;
      visited.add(key);
      const imported = allEntries[key];
      if (!imported) throw new Error(`missing imported manifest entry: ${key}`);
      imports.push(imported);
      visit(imported);
    }
  }

  visit(rootEntry);
  return imports;
}

function normalizeBasePath(value) {
  const path = value?.trim().replace(/^\/+|\/+$/g, "");
  return path ? `/${path}/` : "/";
}

function assetHref(path) {
  return `${basePath}${path.replace(/^\/+/, "")}`;
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}
