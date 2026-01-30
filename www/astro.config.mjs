// @ts-check
import { defineConfig, fontProviders } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import cloudflare from '@astrojs/cloudflare';

// https://astro.build/config
export default defineConfig({
  site: 'https://better-resource-monitor.alexpedersen.dev',
  integrations: [sitemap()],
  adapter: cloudflare({
    platformProxy: {
      enabled: true
    }
  }),
  experimental: {
    fonts: [
      {
        provider: fontProviders.google(),
        name: 'Inter',
        cssVariable: '--font-sans',
        weights: [400, 500, 600, 700, 800],
        fallbacks: ['system-ui', 'sans-serif']
      },
      {
        provider: fontProviders.google(),
        name: 'JetBrains Mono',
        cssVariable: '--font-mono',
        weights: [400],
        fallbacks: ['ui-monospace', 'monospace']
      }
    ]
  }
});