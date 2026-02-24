import type { APIRoute, GetStaticPaths } from "astro";
import { renderImage, colors, trayIconBase64 } from "../../lib/og";

export const prerender = true;

const headlines = [
  { slug: "simplicity", title: "System Stats\nin Your Menu Bar" },
  { slug: "performance", title: "Under 0.1% CPU.\n15 MB RAM." },
  { slug: "privacy", title: "Runs Locally.\nNo Telemetry." },
];

const WIDTH = 2880;
const HEIGHT = 1800;

export const getStaticPaths: GetStaticPaths = () =>
  headlines.map((h) => ({
    params: { id: h.slug },
    props: { title: h.title },
  }));

export const GET: APIRoute = async ({ props }) => {
  const { title } = props as { title: string };

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
        padding: "140px 180px",
      },
      children: [
        {
          type: "div",
          props: {
            style: {
              fontSize: 220,
              fontWeight: 700,
              color: colors.text,
              textAlign: "center",
              lineHeight: 1.1,
              marginBottom: 80,
            },
            children: title,
          },
        },
        {
          type: "div",
          props: {
            style: {
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              backgroundColor: "#252525",
              borderRadius: 24,
              border: `3px solid ${colors.border}`,
              padding: "60px 80px",
            },
            children: {
              type: "img",
              props: {
                src: trayIcon,
                width: 2400,
                height: 124,
                style: { objectFit: "contain" },
              },
            },
          },
        },
      ],
    },
  };

  const png = await renderImage(element, WIDTH, HEIGHT);

  return new Response(png, {
    headers: { "Content-Type": "image/png" },
  });
};
