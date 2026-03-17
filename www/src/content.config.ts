import { defineCollection } from "astro:content";
import { glob } from "astro/loaders";
import { z } from "astro/zod";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { siteMarketingLocales } from "./lib/translations";

const ui = defineCollection({
  loader: glob({ pattern: "*.json", base: "./src/content/ui" }),
  schema: z.object({
    backToHome: z.string(),

    footer: z.object({
      faq: z.string(),
      privacyPolicy: z.string(),
      terms: z.string(),
      license: z.string(),
      support: z.string(),
      github: z.string(),
    }),
  }),
});

const home = defineCollection({
  loader: glob({ pattern: "*.json", base: "./src/content/home" }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    ogImage: z.string(),
  }),
});

const homeBody = defineCollection({
  loader: {
    name: "readme-loader",
    load: async ({ store, renderMarkdown }) => {
      const rootDir = resolve(import.meta.dirname, "../..");
      store.clear();

      for (const locale of siteMarketingLocales) {
        const filename = locale === "en" ? "README.md" : `README.${locale}.md`;
        const raw = await readFile(resolve(rootDir, filename), "utf-8");

        const body = raw
          .replace(/<!-- README-LANG-START -->\n?/s, "")
          .replace(/\n?<!-- README-LANG-END -->\n*/s, "")
          .replace(/<p align="center">\n {2}<a href="#[\s\S]*?<\/p>\n\n/s, "")
          .trim();

        const rendered = await renderMarkdown(body);
        const html = rendered.html
          .replace(/href="README\.md"/g, 'href="/"')
          .replace(/href="README\.([\w-]+)\.md"/g, (_, loc) => `href="/${loc}/"`);
        store.set({ id: locale, data: {}, rendered: { html } });
      }
    },
  },
});

const faq = defineCollection({
  loader: glob({ pattern: "*.json", base: "./src/content/faq" }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    ogImage: z.string(),
    heading: z.string(),
    items: z.array(
      z.object({
        question: z.string(),
        answer: z.string(),
      }),
    ),
  }),
});

const comparisonTable = z.object({
  featureLabel: z.string(),
  productLabel: z.string(),
  competitorLabel: z.string(),
  features: z.array(z.string()),
  productValues: z.array(z.string()),
  competitorValues: z.array(z.string()),
});

const comparisons = defineCollection({
  loader: glob({ pattern: "**/*.json", base: "./src/content/comparisons" }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    ogImage: z.string(),
    competitorName: z.string(),
    competitorPrice: z.string(),
    heading: z.string(),
    intro: z.string(),
    table: comparisonTable,
    sections: z.array(
      z.object({
        heading: z.string(),
        body: z.string(),
      }),
    ),
    cta: z.string(),
  }),
});

const legal = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/legal" }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    ogImage: z.string(),
  }),
});

export const collections = { ui, home, homeBody, faq, comparisons, legal };
