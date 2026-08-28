import { glob } from "astro/loaders";
import { z } from "astro/zod";
import { defineCollection } from "astro:content";

const pageSchema = z.object({
  title: z.string().min(1),
  description: z.string().min(1),
});

const docs = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/docs" }),
  schema: pageSchema,
});

const components = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/components" }),
  schema: pageSchema,
});

const charts = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/charts" }),
  schema: pageSchema,
});

export const collections = { docs, components, charts };
