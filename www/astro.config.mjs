// @ts-check
import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';

// https://astro.build/config
export default defineConfig({
  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'es', 'pl', 'pt-br', 'zh-cn'],
    routing: {
      prefixDefaultLocale: false,
    },
  },
  markdown: {
    shikiConfig: {
      theme: 'monokai',
    },
  },
  site: 'https://better-resource-monitor.alexpedersen.dev',
  trailingSlash: 'always',
  integrations: [sitemap()],
  vite: {
    build: {
      rollupOptions: {
        external: [/@resvg[\\/]/, /\.node$/],
      },
    },
    ssr: {
      // Keep the native image renderer in Node during prerendering.
      external: ['@resvg/resvg-js', 'node:fs', 'node:module', 'node:path'],
    },
  },
});
