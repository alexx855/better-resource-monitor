// @ts-check
import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import cloudflare from '@astrojs/cloudflare';

/** @param {string} id */
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
    // platformProxy was removed in @astrojs/cloudflare v13; the underlying
    // @cloudflare/vite-plugin provides bindings proxying in dev automatically.
    prerenderEnvironment: 'node',
  }),
  vite: {
    build: {
      rollupOptions: {
        external: externalNativeRenderer,
      },
    },
    ssr: {
      // @ts-expect-error resvg native module matched by regex; Vite types
      // ssr.external as string[] but RegExp entries work at runtime and the
      // resvg externalization is load-bearing for the image routes.
      external: [/^@resvg\/resvg-js(?:-.+)?$/, 'node:fs', 'node:module', 'node:path'],
    },
  },
});
