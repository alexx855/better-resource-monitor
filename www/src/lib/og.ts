import satori from "satori";
import { Resvg } from "@resvg/resvg-js";
import { readFileSync } from "node:fs";
import { join } from "node:path";

// Design tokens (matching Layout.astro CSS vars)
export const colors = {
  bg: "#181818",
  surface: "#202020",
  surfaceAlt: "#242424",
  accent: "#edbc63",
  text: "#ffffff",
  textDim: "#c5c5c5",
  border: "#3a3a3a",
};

// Font cache — fetched once per build
let fontData: ArrayBuffer | null = null;

async function loadFont(): Promise<ArrayBuffer> {
  if (fontData) return fontData;
  const res = await fetch(
    "https://fonts.gstatic.com/s/jetbrainsmono/v24/tDbY2o-flEEny0FZhsfKu5WU4zr3E_BX0PnT8RD8yKxjPQ.ttf"
  );
  fontData = await res.arrayBuffer();
  return fontData;
}

let fontBoldData: ArrayBuffer | null = null;

async function loadFontBold(): Promise<ArrayBuffer> {
  if (fontBoldData) return fontBoldData;
  const res = await fetch(
    "https://fonts.gstatic.com/s/jetbrainsmono/v24/tDbY2o-flEEny0FZhsfKu5WU4zr3E_BX0PnT8RD8L6tjPQ.ttf"
  );
  fontBoldData = await res.arrayBuffer();
  return fontBoldData;
}

export async function renderImage(
  element: Record<string, unknown>,
  width: number,
  height: number
): Promise<Uint8Array> {
  const [font, fontBold] = await Promise.all([loadFont(), loadFontBold()]);

  const svg = await satori(element as React.ReactNode, {
    width,
    height,
    fonts: [
      { name: "JetBrains Mono", data: font, weight: 400, style: "normal" },
      { name: "JetBrains Mono", data: fontBold, weight: 700, style: "normal" },
    ],
  });

  const resvg = new Resvg(svg, {
    fitTo: { mode: "width", value: width },
  });
  return resvg.render().asPng();
}

export function trayIconBase64(): string {
  const imgPath = join(process.cwd(), "public", "better-resource-monitor.png");
  const buf = readFileSync(imgPath);
  return `data:image/png;base64,${buf.toString("base64")}`;
}
