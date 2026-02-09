import type { APIRoute } from "astro";
import { renderImage, colors, trayIconBase64 } from "../lib/og";

export const prerender = true;

export const GET: APIRoute = async () => {
  const trayIcon = trayIconBase64();

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
        padding: "48px 64px",
      },
      children: [
        // App name — uppercase section-title style
        {
          type: "div",
          props: {
            style: {
              fontSize: 14,
              fontWeight: 400,
              color: colors.accent,
              textTransform: "uppercase",
              letterSpacing: "0.1em",
              marginBottom: 24,
            },
            children: "Better Resource Monitor",
          },
        },
        // Tray icon in a bordered surface card
        {
          type: "div",
          props: {
            style: {
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              backgroundColor: colors.surface,
              border: `1px solid ${colors.border}`,
              padding: "24px 48px",
              marginBottom: 32,
            },
            children: {
              type: "img",
              props: {
                src: trayIcon,
                width: 700,
                height: 36,
                style: { objectFit: "contain" },
              },
            },
          },
        },
        // Thick gold accent bar
        {
          type: "div",
          props: {
            style: {
              width: 64,
              height: 2,
              backgroundColor: colors.accent,
              marginBottom: 28,
            },
          },
        },
        // Title
        {
          type: "div",
          props: {
            style: {
              fontSize: 40,
              fontWeight: 700,
              color: colors.text,
              textAlign: "center",
              lineHeight: 1.2,
              marginBottom: 16,
            },
            children: "System Monitor for macOS & Linux",
          },
        },
        // Subtitle
        {
          type: "div",
          props: {
            style: {
              fontSize: 18,
              color: colors.textDim,
              textAlign: "center",
              lineHeight: 1.5,
            },
            children:
              "CPU, memory, GPU, and network — always visible in your menu bar",
          },
        },
      ],
    },
  };

  const png = await renderImage(element, 1200, 630);

  return new Response(png, {
    headers: { "Content-Type": "image/png" },
  });
};
