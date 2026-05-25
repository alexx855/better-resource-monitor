import type { APIRoute } from "astro";
import { getLocalizedPath } from "../lib/site";

export const prerender = false;

export const GET: APIRoute = () =>
  new Response(null, {
    status: 301,
    headers: {
      Location: getLocalizedPath("en"),
    },
  });
