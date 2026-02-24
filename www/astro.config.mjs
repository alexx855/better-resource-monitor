// @ts-check
import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import cloudflare from '@astrojs/cloudflare';

// https://astro.build/config
export default defineConfig({
  markdown: {
    shikiConfig: {
      theme: 'github-dark',
    },
  },
  site: 'https://better-resource-monitor.alexpedersen.dev',
  integrations: [sitemap()],
  adapter: cloudflare({
    platformProxy: {
      enabled: true
    }
  }),
  vite: {
    ssr: {
      external: ['@resvg/resvg-js', 'node:fs', 'node:path'],
    },
  },
});
