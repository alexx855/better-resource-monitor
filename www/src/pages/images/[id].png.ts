import type { APIRoute, GetStaticPaths } from "astro";
import { renderImage, colors, imageBackgroundStyle, trayIconBase64 } from "../../lib/renderer";
import { marketingCopy, screenshotKeys, siteMarketingLocales } from "../../lib/translations";

export const prerender = true;

const BASE_WIDTH = 2880;
const OG_TEXT_BASE = 2100;

// App Store screenshots: 4 languages x 3 screenshots = 12 entries (2880x1800, 16:10)
const appStoreEntries = siteMarketingLocales.flatMap((locale) =>
  screenshotKeys.map((key) => ({
    slug: `${key}-${marketingCopy[locale].screenshotLang}`,
    title: marketingCopy[locale].screenshots[key],
    width: 2880,
    height: 1800,
    lang: marketingCopy[locale].screenshotLang,
  }))
);

// OG images: 6 pages x 4 locales = 24 entries (1200x630)
const ogEntries = siteMarketingLocales.flatMap((locale) =>
  Object.entries(marketingCopy[locale].ogTitles).map(([key, title]) => ({
    slug: `og-${key}-${locale}`,
    title,
    width: 1200,
    height: 630,
    showAppName: true,
    textBase: OG_TEXT_BASE,
    lang: marketingCopy[locale].screenshotLang,
  }))
);

const entries = [...appStoreEntries, ...ogEntries];

function scaled(base: number, width: number, baseWidth = BASE_WIDTH) {
  return Math.round(base * (width / baseWidth));
}

export const getStaticPaths: GetStaticPaths = () =>
  entries.map((e) => ({
    params: { id: e.slug },
    props: e,
  }));

export const GET: APIRoute = async ({ props }) => {
  const { title, width, height, showAppName, textBase, lang } = props;
  const trayIcon = trayIconBase64(showAppName ? "alert" : "full");
  const s = (base: number) => scaled(base, width);
  const st = (base: number) => scaled(base, width, textBase);

  const children: Record<string, unknown>[] = [];

  if (showAppName) {
    children.push({
      type: "div",
      props: {
        style: {
          alignSelf: "stretch",
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          fontSize: st(60),
          fontWeight: 700,
          color: colors.text,
          backgroundColor: colors.brand,
          textTransform: "uppercase",
          letterSpacing: "0.1em",
          marginBottom: st(48),
          padding: `${st(8)}px ${st(24)}px`,
        },
        children: "Better Resource Monitor",
      },
    });

    children.push({
      type: "div",
      props: {
        style: {
          fontSize: st(160),
          fontWeight: 700,
          color: colors.text,
          textAlign: "center",
          whiteSpace: "pre-line",
          lineHeight: 1.1,
          marginBottom: st(100),
        },
        children: title,
      },
    });

    children.push({
      type: "img",
      props: {
        src: trayIcon,
        style: {
          width: "100%",
        },
      },
    });
  } else {
    children.push({
      type: "img",
      props: {
        src: trayIcon,
        style: {
          width: "100%",
          objectFit: "cover",
        },
      },
    });

    children.push({
      type: "div",
      props: {
        style: {
          flex: 1,
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          alignItems: "center",
          width: "100%",
          boxSizing: "border-box",
          padding: `0 ${s(140)}px`,
        },
        children: {
          type: "div",
          props: {
            style: {
              fontSize: st(200),
              fontWeight: 700,
              color: colors.text,
              textAlign: "center",
              whiteSpace: "pre-line",
              lineHeight: 1.1,
            },
            children: title,
          },
        },
      },
    });
  }

  const element = {
    type: "div",
    props: {
      style: {
        width: "100%",
        height: "100%",
        boxSizing: "border-box",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: showAppName ? "center" : "flex-start",
        ...imageBackgroundStyle,
        fontFamily: "JetBrains Mono",
        padding: showAppName ? `0 ${s(140)}px` : "0",
      },
      children,
    },
  };

  const png = await renderImage(element, width, height, lang);
  return new Response(Buffer.from(png), { headers: { "Content-Type": "image/png" } });
};
