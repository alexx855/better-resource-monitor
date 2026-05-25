import type { APIRoute } from "astro";
import { getLocalizedPath, isSupportedLocale } from "../../lib/site";

export const prerender = false;

export const GET: APIRoute = ({ params }) => {
  const { locale } = params;
  if (!locale || !isSupportedLocale(locale) || locale === "en") {
    return new Response(null, { status: 404 });
  }

  return new Response(null, {
    status: 301,
    headers: {
      Location: getLocalizedPath(locale),
    },
  });
};
