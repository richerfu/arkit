import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";

function normalizeBase(value) {
  const path = value?.trim().replace(/^\/+|\/+$/g, "");
  return path ? `/${path}` : undefined;
}

export default defineConfig({
  site: "https://richerfu.github.io",
  base: normalizeBase(process.env.SITE_BASE_PATH),
  trailingSlash: "always",
  integrations: [],
  markdown: {
    shikiConfig: {
      themes: {
        light: "github-light",
        dark: "github-dark",
      },
      defaultColor: false,
    },
  },
  vite: {
    plugins: [tailwindcss()],
  },
});
