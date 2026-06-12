// @ts-check
import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import cloudflare from '@astrojs/cloudflare';

const externalNativeRenderer = (id) => /@resvg[\\/]/.test(id) || id.endsWith('.node');

// https://astro.build/config
export default defineConfig({
  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'es', 'pt-br', 'zh-cn'],
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
  adapter: cloudflare({
    imageService: 'passthrough',
    platformProxy: {
      enabled: true
    },
    prerenderEnvironment: 'node',
  }),
  vite: {
    build: {
      rollupOptions: {
        external: externalNativeRenderer,
      },
    },
    ssr: {
      external: [/^@resvg\/resvg-js(?:-.+)?$/, 'node:fs', 'node:module', 'node:path'],
    },
  },
});
