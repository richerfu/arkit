import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync, statSync } from "node:fs";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import tailwindcss from "@tailwindcss/vite";
import { voidMarkdown } from "@void/md/plugin";
import { voidReact } from "@void/react/plugin";
import { defineConfig, type Plugin } from "vite";

const rootDir = dirname(fileURLToPath(import.meta.url));
const siteBasePath = normalizeBasePath(process.env.SITE_BASE_PATH);
const CONTENT_LAST_UPDATED_ID = "virtual:content-last-updated";
const RESOLVED_CONTENT_LAST_UPDATED_ID = `\0${CONTENT_LAST_UPDATED_ID}`;
const contentAreas = ["docs", "components", "charts"] as const;
const contentLandingByArea = {
  docs: "getting-started",
  components: "overview",
  charts: "overview",
} as const;
const contentMarkdownFiles = contentAreas.flatMap((area) =>
  readdirSync(resolve(rootDir, "src/content", area))
    .filter((file) => file.endsWith(".md"))
    .map((file) => [area, basename(file, ".md"), file] as const),
);

function contentFallback(): Plugin {
  return {
    name: "content-subpath-fallback",
    apply: "serve",
    configureServer(server) {
      server.middlewares.use((req, _res, next) => {
        const area = contentAreas.find((candidate) => req.url?.startsWith(`/${candidate}/`));
        if (area && !req.url?.includes(".")) req.url = `/${area}/index.html`;
        next();
      });
    },
    configurePreviewServer(server) {
      server.middlewares.use((req, _res, next) => {
        const area = contentAreas.find((candidate) => req.url?.startsWith(`/${candidate}/`));
        if (area && !req.url?.includes(".")) req.url = `/${area}/index.html`;
        next();
      });
    },
  };
}

function contentLastUpdated(): Plugin {
  return {
    name: "content-last-updated",
    resolveId(id) {
      return id === CONTENT_LAST_UPDATED_ID ? RESOLVED_CONTENT_LAST_UPDATED_ID : null;
    },
    load(id) {
      if (id !== RESOLVED_CONTENT_LAST_UPDATED_ID) return null;
      const updated = Object.fromEntries(
        contentMarkdownFiles.map(([area, sectionId, file]) => {
          const filePath = resolve(rootDir, "src/content", area, file);
          return [`${area}/${sectionId}`, lastUpdatedIso(filePath)];
        }),
      );
      return `export const lastUpdatedByContent = ${JSON.stringify(updated, null, 2)};`;
    },
  };
}

function contentStaticRoutes(): Plugin {
  return {
    name: "content-static-routes",
    apply: "build",
    closeBundle() {
      for (const [area, sectionId] of contentMarkdownFiles) {
        if (sectionId === contentLandingByArea[area]) continue;
        const areaEntry = resolve(rootDir, "dist", area, "index.html");
        const routeDir = resolve(rootDir, "dist", area, sectionId);
        mkdirSync(routeDir, { recursive: true });
        copyFileSync(areaEntry, resolve(routeDir, "index.html"));
      }
    },
  };
}

function normalizeBasePath(value: string | undefined) {
  const path = value?.trim().replace(/^\/+|\/+$/g, "");
  return path ? `/${path}/` : "/";
}

function lastUpdatedIso(filePath: string) {
  try {
    const committed = execFileSync("git", ["log", "-1", "--format=%cI", "--", filePath], {
      cwd: rootDir,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
    if (committed) return committed;
  } catch {
    // New files have no git history; use the filesystem timestamp until committed.
  }
  return statSync(filePath).mtime.toISOString();
}

export default defineConfig({
  base: siteBasePath,
  plugins: [
    voidReact(),
    voidMarkdown({
      shiki: { themes: { light: "github-light", dark: "github-dark" } },
    }),
    contentFallback(),
    contentLastUpdated(),
    contentStaticRoutes(),
    tailwindcss(),
  ],
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(rootDir, "index.html"),
        docs: resolve(rootDir, "docs/index.html"),
        components: resolve(rootDir, "components/index.html"),
        charts: resolve(rootDir, "charts/index.html"),
      },
    },
  },
});
