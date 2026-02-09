import type { APIRoute, GetStaticPaths } from "astro";
import { renderImage, colors, trayIconBase64 } from "../../lib/og";

export const prerender = true;

const headlines = [
  {
    slug: "simplicity",
    title: "Your System at a Glance",
    subtitle:
      "CPU, memory, GPU, and network stats — always visible, never in the way.",
  },
  {
    slug: "performance",
    title: "Monitor Everything. Use Nothing.",
    subtitle:
      "Under 0.1% CPU and 15 MB RAM. Built in Rust for near-zero overhead.",
  },
  {
    slug: "privacy",
    title: "No Windows. No Bloat. Just Stats.",
    subtitle:
      "100% local monitoring with zero telemetry. Lives entirely in your menu bar.",
  },
];

const sizes = [
  { w: 2880, h: 1800 },
  { w: 2560, h: 1600 },
  { w: 1440, h: 900 },
  { w: 1280, h: 800 },
];

export const getStaticPaths: GetStaticPaths = () =>
  headlines.flatMap((h) =>
    sizes.map((s) => ({
      params: { id: `${h.slug}-${s.w}x${s.h}` },
      props: { title: h.title, subtitle: h.subtitle, width: s.w, height: s.h },
    }))
  );

export const GET: APIRoute = async ({ props }) => {
  const { title, subtitle, width, height } = props as {
    title: string;
    subtitle: string;
    width: number;
    height: number;
  };

  const trayIcon = trayIconBase64();
  const s = width / 2880;

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
        padding: `${Math.round(120 * s)}px ${Math.round(160 * s)}px`,
      },
      children: [
        // Title — hero headline
        {
          type: "div",
          props: {
            style: {
              fontSize: Math.round(128 * s),
              fontWeight: 700,
              color: colors.text,
              textAlign: "center",
              lineHeight: 1.15,
              marginBottom: Math.round(48 * s),
            },
            children: title,
          },
        },
        // App name — small gold uppercase label
        {
          type: "div",
          props: {
            style: {
              fontSize: Math.round(24 * s),
              fontWeight: 400,
              color: colors.accent,
              textTransform: "uppercase",
              letterSpacing: "0.12em",
              marginBottom: Math.round(56 * s),
            },
            children: "Better Resource Monitor",
          },
        },
        // Tray icon in bordered surface card
        {
          type: "div",
          props: {
            style: {
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              backgroundColor: colors.surface,
              border: `${Math.round(2 * s)}px solid ${colors.border}`,
              padding: `${Math.round(40 * s)}px ${Math.round(64 * s)}px`,
              marginBottom: Math.round(56 * s),
            },
            children: {
              type: "img",
              props: {
                src: trayIcon,
                width: Math.round(1600 * s),
                height: Math.round(82 * s),
                style: { objectFit: "contain" },
              },
            },
          },
        },
        // Subtitle
        {
          type: "div",
          props: {
            style: {
              fontSize: Math.round(40 * s),
              color: colors.textDim,
              textAlign: "center",
              lineHeight: 1.5,
              maxWidth: `${Math.round(2200 * s)}px`,
            },
            children: subtitle,
          },
        },
      ],
    },
  };

  const png = await renderImage(element, width, height);

  return new Response(png, {
    headers: { "Content-Type": "image/png" },
  });
};
