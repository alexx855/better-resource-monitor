import type { APIRoute, GetStaticPaths } from "astro";
import { renderImage, colors, trayIconBase64 } from "../../lib/renderer";

export const prerender = true;

const BASE_WIDTH = 2880;
const OG_TEXT_BASE = 2100;

const entries = [
  // App Store (2880x1800, 16:10 required by App Store Connect)
  { slug: "simplicity", title: "System Stats\nin Your Menu Bar", width: 2880, height: 1800 },
  { slug: "performance", title: "Under 0.1% CPU\n15 MB RAM", width: 2880, height: 1800 },
  { slug: "privacy", title: "Runs Locally\nNo Telemetry", width: 2880, height: 1800 },
  // OG images (1200x630) — textBase scales text ~36% larger for mobile readability
  { slug: "og-index", title: "System Stats\nin Your Menu Bar", width: 1200, height: 630, showAppName: true, textBase: OG_TEXT_BASE },
  { slug: "og-faq", title: "Frequently Asked\nQuestions", width: 1200, height: 630, showAppName: true, textBase: OG_TEXT_BASE },
  { slug: "og-privacy", title: "Privacy Policy", width: 1200, height: 630, showAppName: true, textBase: OG_TEXT_BASE },
  { slug: "og-terms", title: "Terms & Conditions", width: 1200, height: 630, showAppName: true, textBase: OG_TEXT_BASE },
  // Comparison page OG images
  { slug: "og-vs-stats", title: "Better Resource Monitor\nvs Stats", width: 1200, height: 630, showAppName: true, textBase: OG_TEXT_BASE },
  { slug: "og-vs-istat-menus", title: "Better Resource Monitor\nvs iStat Menus", width: 1200, height: 630, showAppName: true, textBase: OG_TEXT_BASE },
  { slug: "og-vs-eul", title: "Better Resource Monitor\nvs Eul", width: 1200, height: 630, showAppName: true, textBase: OG_TEXT_BASE },
];

function scaled(base: number, width: number, baseWidth = BASE_WIDTH) {
  return Math.round(base * (width / baseWidth));
}

export const getStaticPaths: GetStaticPaths = () =>
  entries.map((e) => ({
    params: { id: e.slug },
    props: e,
  }));

export const GET: APIRoute = async ({ props }) => {
  const { title, width, height, showAppName, textBase } = props as (typeof entries)[number];
  const trayIcon = trayIconBase64();
  const s = (base: number) => scaled(base, width);
  const st = (base: number) => scaled(base, width, textBase);

  const children: Record<string, unknown>[] = [];

  if (showAppName) {
    children.push({
      type: "div",
      props: {
        style: {
          fontSize: st(96),
          fontWeight: 700,
          color: colors.accent,
          textTransform: "uppercase",
          letterSpacing: "0.1em",
          marginBottom: st(48),
        },
        children: "Better Resource Monitor",
      },
    });
  }

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
    type: "div",
    props: {
      style: {
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        backgroundColor: "#252525",
        borderRadius: s(24),
        border: `${s(3)}px solid ${colors.border}`,
        padding: `${s(40)}px ${s(70)}px`,
      },
      children: {
        type: "img",
        props: {
          src: trayIcon,
          width: s(2400),
          height: s(124),
          style: { objectFit: "contain" },
        },
      },
    },
  });

  const element = {
    type: "div",
    props: {
      style: {
        width: "100%",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        backgroundColor: colors.bg,
        fontFamily: "JetBrains Mono",
        padding: `0 ${s(140)}px`,
      },
      children,
    },
  };

  const png = await renderImage(element, width, height);
  return new Response(png, { headers: { "Content-Type": "image/png" } });
};
