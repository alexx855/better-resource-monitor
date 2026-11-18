import { defineCollection } from "astro:content";
import { glob } from "astro/loaders";
import { z } from "astro/zod";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { getAlternateLinks, getLocaleMeta } from "./lib/site";
import { siteMarketingLocales, type SiteMarketingLocale } from "./lib/translations";

const productionOrigin = "https://better-resource-monitor.alexpedersen.dev";
const executableHtmlPatterns = [
  { label: "script tags", pattern: /<\s*script\b/i },
  { label: "inline event handlers", pattern: /\son[a-z]+\s*=/i },
  { label: "javascript URLs", pattern: /javascript\s*:/i },
  { label: "embedded frames", pattern: /<\s*(iframe|object|embed)\b/i },
];

function assertNoExecutableHtml(html: string, source: string) {
  for (const { label, pattern } of executableHtmlPatterns) {
    if (pattern.test(html)) {
      throw new Error(`${source} contains unsafe HTML: ${label}`);
    }
  }
}

const homeReadmeLabels = {
  en: {
    languages: "Choose language",
    comparison: "Menu bar monitor comparison",
    preview: "Example menu bar readings for CPU, memory, storage, GPU, and network in normal and warning states",
  },
  es: {
    languages: "Elegir idioma",
    comparison: "Comparación de monitores para la barra de menús",
    preview: "Ejemplos en la barra de menús de CPU, memoria, almacenamiento, GPU y red en estado normal y de alerta",
  },
  pl: {
    languages: "Wybierz język",
    comparison: "Porównanie monitorów na pasek menu",
    preview: "Przykładowe odczyty CPU, pamięci, dysku, GPU i sieci na pasku menu w stanie normalnym i ostrzegawczym",
  },
  "pt-br": {
    languages: "Escolher idioma",
    comparison: "Comparação de monitores de barra de menus",
    preview: "Exemplos na barra de menus de CPU, memória, armazenamento, GPU e rede em estado normal e de alerta",
  },
  "zh-cn": {
    languages: "选择语言",
    comparison: "菜单栏监视器对比",
    preview: "菜单栏示例，展示正常和警告状态下的 CPU、内存、存储、GPU 和网络读数",
  },
} satisfies Record<SiteMarketingLocale, { languages: string; comparison: string; preview: string }>;

const ui = defineCollection({
  loader: glob({ pattern: "*.json", base: "./src/content/ui" }),
  schema: z.object({
    backToHome: z.string(),

    footer: z.object({
      internalNavigation: z.string(),
      externalNavigation: z.string(),
      comparison: z.string(),
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
        const labels = homeReadmeLabels[locale];
        const languageLinks = getAlternateLinks()
          .filter((link) => link.locale !== locale)
          .map(({ locale, hrefLang, href }) =>
            `<a href="${href}" lang="${hrefLang}" hreflang="${hrefLang}">${getLocaleMeta(locale).label}</a>`,
          );
        const body = raw
          // Use the shared site locale metadata for switcher links.
          .replace(/<!-- README-LANG-START -->[\s\S]*?<!-- README-LANG-END -->/,
            `<nav class="language-switcher" aria-label="${labels.languages}">${languageLinks.join(" ")}</nav>`,
          )
          .replace(/<p align="center">\n {2}<a href="#[\s\S]*?<\/p>\n\n/s, "")
          .trim();

        const rendered = await renderMarkdown(body);
        let headingId: string | undefined;
        const html = rendered.html
          .replace(/(<(?:h1|p|td)) align="center"/g, "$1")
          .replace(
            "<table>",
            `<div class="table-scroll" tabindex="0" role="region" aria-labelledby="home-comparison-caption"><table>\n<caption id="home-comparison-caption" class="sr-only">${labels.comparison}</caption>`,
          )
          .replace("</table>", "</table></div>")
          .replace(/<th width="20%">/g, '<th scope="col">')
          .replaceAll(`src="${productionOrigin}/`, 'src="/')
          .replaceAll(`href="${productionOrigin}/`, 'href="/')
          .replace(/target="_blank"(?![^>]*\srel=)/g, 'target="_blank" rel="noopener noreferrer"')
          .replace(
            '<img src="/better-resource-monitor-storage.png" alt="Better Resource Monitor"',
            `<img src="/better-resource-monitor-storage.png" alt="${labels.preview}" fetchpriority="high" decoding="async"`,
          )
          // Name scrollable code blocks after the preceding Markdown heading.
          .replace(/<h[1-6](?: id="([^"]+)")?[^>]*>|<pre\b/g, (tag, id: string | undefined) => {
            if (tag !== "<pre") {
              headingId = id;
              return tag;
            }
            return headingId ? `<pre role="region" aria-labelledby="${headingId}"` : tag;
          });
        assertNoExecutableHtml(html, filename);
        store.set({ id: locale, data: {}, rendered: { html } });
      }
    },
  },
});

const faqItems = z.array(
  z.object({
    id: z.string().optional(),
    question: z.string(),
    answer: z.string(),
  }),
);

const faq = defineCollection({
  loader: glob({ pattern: "*.json", base: "./src/content/faq" }),
  schema: z.object({
    items: faqItems,
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
    faqItems: faqItems.optional(),
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
