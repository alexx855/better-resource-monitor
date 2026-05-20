import type { APIRoute } from "astro";

export function legacyComparisonRedirect(destination: string): APIRoute {
  return ({ request }) => {
    const url = new URL(request.url);

    return new Response(null, {
      status: 301,
      headers: {
        Location: `${destination}${url.search}`,
      },
    });
  };
}
