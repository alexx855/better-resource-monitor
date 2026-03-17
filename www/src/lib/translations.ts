import marketingCopy from "./marketing-copy.json";

const siteMarketingCopy = marketingCopy;

export type SiteMarketingLocale = keyof typeof siteMarketingCopy;
export type Lang = (typeof siteMarketingCopy)[SiteMarketingLocale]["screenshotLang"];
export type ScreenshotKey = keyof (typeof siteMarketingCopy)["en"]["screenshots"];
export type BadgeKey = keyof (typeof siteMarketingCopy)["en"]["badges"];
export type LocaleMeta = (typeof siteMarketingCopy)[SiteMarketingLocale]["meta"];

export const siteMarketingLocales = Object.keys(marketingCopy) as SiteMarketingLocale[];

export const screenshotKeys: ScreenshotKey[] = ["simplicity", "performance", "privacy"];

export const screenshotLocales = siteMarketingLocales.map((locale) => ({
  locale,
  lang: siteMarketingCopy[locale].screenshotLang,
})) satisfies Array<{ locale: SiteMarketingLocale; lang: Lang }>;

export const appStoreScreenshots: Record<Lang, typeof siteMarketingCopy.en.screenshots> = {
  en: siteMarketingCopy.en.screenshots,
  es: siteMarketingCopy.es.screenshots,
  "pt-BR": siteMarketingCopy["pt-br"].screenshots,
  "zh-Hans": siteMarketingCopy["zh-cn"].screenshots,
};

export const supportedLangs: Lang[] = screenshotLocales.map(({ lang }) => lang);
export const localizedBadges = {
  en: siteMarketingCopy.en.badges,
  es: siteMarketingCopy.es.badges,
  "pt-br": siteMarketingCopy["pt-br"].badges,
  "zh-cn": siteMarketingCopy["zh-cn"].badges,
} satisfies Record<SiteMarketingLocale, (typeof siteMarketingCopy)[SiteMarketingLocale]["badges"]>;

export const localeMeta = {
  en: siteMarketingCopy.en.meta,
  es: siteMarketingCopy.es.meta,
  "pt-br": siteMarketingCopy["pt-br"].meta,
  "zh-cn": siteMarketingCopy["zh-cn"].meta,
} satisfies Record<SiteMarketingLocale, LocaleMeta>;

export const ogTitles = {
  en: siteMarketingCopy.en.ogTitles,
  es: siteMarketingCopy.es.ogTitles,
  "pt-br": siteMarketingCopy["pt-br"].ogTitles,
  "zh-cn": siteMarketingCopy["zh-cn"].ogTitles,
} satisfies Record<SiteMarketingLocale, (typeof siteMarketingCopy)[SiteMarketingLocale]["ogTitles"]>;

export const screenshotLangByLocale: Record<SiteMarketingLocale, Lang> = {
  en: siteMarketingCopy.en.screenshotLang,
  es: siteMarketingCopy.es.screenshotLang,
  "pt-br": siteMarketingCopy["pt-br"].screenshotLang,
  "zh-cn": siteMarketingCopy["zh-cn"].screenshotLang,
};
