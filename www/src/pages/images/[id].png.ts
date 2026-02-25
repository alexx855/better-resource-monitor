import type { APIRoute, GetStaticPaths } from "astro";
import { renderImage, colors, trayIconBase64 } from "../../lib/renderer";

export const prerender = true;

const BASE_WIDTH = 2880;

const entries = [
  // App Store (2880x1800, 16:10 required by App Store Connect)
  { slug: "simplicity", title: "System Stats\nin Your Menu Bar", width: 2880, height: 1800 },
  { slug: "performance", title: "Under 0.1% CPU\n15 MB RAM", width: 2880, height: 1800 },
  { slug: "privacy", title: "Runs Locally\nNo Telemetry", width: 2880, height: 1800 },
  // OG images (1200x630)
  { slug: "og-index", title: "System Stats\nin Your Menu Bar", width: 1200, height: 630, showAppName: true },
  { slug: "og-faq", title: "Frequently Asked\nQuestions", width: 1200, height: 630, showAppName: true },
  { slug: "og-privacy", title: "Privacy Policy", width: 1200, height: 630, showAppName: true },
  { slug: "og-terms", title: "Terms & Conditions", width: 1200, height: 630, showAppName: true },
];

function scaled(base: number, width: number) {
  return Math.round(base * (width / BASE_WIDTH));
}

export const getStaticPaths: GetStaticPaths = () =>
  entries.map((e) => ({
    params: { id: e.slug },
    props: e,
  }));

export const GET: APIRoute = async ({ props }) => {
  const { title, width, height, showAppName } = props as (typeof entries)[number];
  const trayIcon = trayIconBase64();
  const s = (base: number) => scaled(base, width);

  const children: Record<string, unknown>[] = [];

  if (showAppName) {
    children.push({
      type: "div",
      props: {
        style: {
          fontSize: s(64),
          fontWeight: 400,
          color: colors.accent,
          textTransform: "uppercase",
          letterSpacing: "0.1em",
          marginBottom: s(48),
        },
        children: "Better Resource Monitor",
      },
    });
  }

  children.push({
    type: "div",
    props: {
      style: {
        fontSize: s(160),
        fontWeight: 700,
        color: colors.text,
        textAlign: "center",
        whiteSpace: "pre-line",
        lineHeight: 1.1,
        marginBottom: s(100),
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
        padding: `${s(80)}px ${s(140)}px`,
      },
      children,
    },
  };

  const png = await renderImage(element, width, height);
  return new Response(png, { headers: { "Content-Type": "image/png" } });
};
