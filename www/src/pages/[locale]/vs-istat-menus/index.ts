import type { APIRoute } from "astro";
import { legacyComparisonRedirect } from "../../../lib/legacy-comparison-redirect";
import { isSupportedLocale } from "../../../lib/site";

export const prerender = false;

export const GET: APIRoute = (context) => {
  const { locale } = context.params;
  if (!locale || !isSupportedLocale(locale) || locale === "en") {
    return new Response(null, { status: 404 });
  }

  return legacyComparisonRedirect(`/${locale}/comparison/vs-istat-menus/`)(context);
};
