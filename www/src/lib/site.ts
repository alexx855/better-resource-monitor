import { localeMeta, siteMarketingLocales, type SiteMarketingLocale } from "./translations";

export const locales = siteMarketingLocales;
export type SupportedLocale = SiteMarketingLocale;
export const localizedLocales = locales.filter((locale) => locale !== "en");
export type ComparisonPageKey = "vs-stats" | "vs-eul" | "vs-istat-menus";

export const internalSlugs = ["faq", "vs-stats", "vs-eul", "vs-istat-menus", "privacy-policy", "terms"] as const;

export type InternalSlug = (typeof internalSlugs)[number];

export type LocalizedPageDescriptor =
  | { kind: "faq" }
  | { kind: "comparison"; pageKey: ComparisonPageKey }
  | { kind: "legal"; slug: "privacy-policy" | "terms" };

const localizedPageMap: Record<InternalSlug, LocalizedPageDescriptor> = {
  faq: { kind: "faq" },
  "vs-stats": { kind: "comparison", pageKey: "vs-stats" },
  "vs-eul": { kind: "comparison", pageKey: "vs-eul" },
  "vs-istat-menus": { kind: "comparison", pageKey: "vs-istat-menus" },
  "privacy-policy": { kind: "legal", slug: "privacy-policy" },
  terms: { kind: "legal", slug: "terms" },
};

const tokenMap = {
  appStoreUrl: "https://apps.apple.com/app/better-resource-monitor/id6758237306",
  githubUrl: "https://github.com/alexx855/better-resource-monitor",
  githubIssuesUrl: "https://github.com/alexx855/better-resource-monitor/issues/new",
} as const;

function getBadgeUrls(locale: SupportedLocale) {
  const baseUrl = "https://better-resource-monitor.alexpedersen.dev/badges";

  return {
    appStoreBadgeUrl: `${baseUrl}/appstore-${locale}.webp`,
    macosBadgeUrl: `${baseUrl}/macos-${locale}.webp`,
    ubuntuBadgeUrl: `${baseUrl}/ubuntu-${locale}.webp`,
  };
}

export function getLocaleMeta(locale: SupportedLocale) {
  return localeMeta[locale];
}

export function getLocalizedPath(locale: SupportedLocale, slug = "") {
  if (locale === "en") {
    return slug ? `/${slug}/` : "/";
  }

  return slug ? `/${locale}/${slug}/` : `/${locale}/`;
}

export function getAlternateLinks(slug = "") {
  return locales.map((locale) => ({
    locale,
    hrefLang: getLocaleMeta(locale).langTag,
    href: getLocalizedPath(locale, slug),
  }));
}

export function getLocalizedInternalLinks(locale: SupportedLocale) {
  return {
    homeUrl: getLocalizedPath(locale),
    faqUrl: getLocalizedPath(locale, "faq"),
    vsStatsUrl: getLocalizedPath(locale, "vs-stats"),
    vsEulUrl: getLocalizedPath(locale, "vs-eul"),
    vsIstatMenusUrl: getLocalizedPath(locale, "vs-istat-menus"),
    privacyPolicyUrl: getLocalizedPath(locale, "privacy-policy"),
    termsUrl: getLocalizedPath(locale, "terms"),
  };
}

export function replaceContentTokens(value: string, locale: SupportedLocale) {
  const links = getLocalizedInternalLinks(locale);
  const badgeUrls = getBadgeUrls(locale);
  let result = value;

  for (const [key, replacement] of Object.entries({ ...tokenMap, ...links, ...badgeUrls })) {
    result = result.replaceAll(`{{${key}}}`, replacement);
  }

  return result;
}

export function isSupportedLocale(value: string): value is SupportedLocale {
  return locales.includes(value as SupportedLocale);
}

export function isInternalSlug(value: string): value is InternalSlug {
  return internalSlugs.includes(value as InternalSlug);
}

export function getLocalizedPageDescriptor(slug: InternalSlug) {
  return localizedPageMap[slug];
}

const markdownLink = /\[([^\]]+)\]\(([^)]+)\)/g;

function escapeHtmlChar(char: string) {
  switch (char) {
    case "&":
      return "&amp;";
    case "<":
      return "&lt;";
    case ">":
      return "&gt;";
    case '"':
      return "&quot;";
    case "'":
      return "&#39;";
    default:
      throw new Error(`Unexpected HTML escape character: ${char}`);
  }
}

function escapeHtml(value: string) {
  return value.replace(/[&<>"']/g, escapeHtmlChar);
}

function assertSafeHref(href: string) {
  if (href.startsWith("/") || href.startsWith("https://")) return;
  throw new Error(`Unsafe content link: ${href}`);
}

export function renderInlineLinks(value: string) {
  let html = "";
  let lastIndex = 0;

  for (const match of value.matchAll(markdownLink)) {
    const [raw, text, href] = match;
    if (text === undefined || href === undefined) {
      throw new Error(`Invalid markdown link: ${raw}`);
    }

    const index = match.index ?? 0;
    assertSafeHref(href);
    html += escapeHtml(value.slice(lastIndex, index));
    html += `<a href="${escapeHtml(href)}">${escapeHtml(text)}</a>`;
    lastIndex = index + raw.length;
  }

  html += escapeHtml(value.slice(lastIndex));
  return html;
}
