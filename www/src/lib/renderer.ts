import satori from "satori";
import { Resvg } from "@resvg/resvg-js";
import { readFileSync } from "node:fs";
import { join } from "node:path";

// Design tokens (matching Layout.astro CSS vars)
export const colors = {
  bg: "#181818",
  surface: "#202020",
  surfaceAlt: "#242424",
  brand: "#D14715",
  text: "#ffffff",
  textDim: "#c5c5c5",
  border: "#3a3a3a",
};

// Font cache — fetched once per build
let fontData: ArrayBuffer | null = null;

async function fetchFont(url: string): Promise<ArrayBuffer> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Failed to fetch font ${url}: ${res.status} ${res.statusText}`);
  return res.arrayBuffer();
}

async function loadFont(): Promise<ArrayBuffer> {
  if (fontData) return fontData;
  fontData = await fetchFont(
    "https://fonts.gstatic.com/s/jetbrainsmono/v24/tDbY2o-flEEny0FZhsfKu5WU4zr3E_BX0PnT8RD8yKxjPQ.ttf"
  );
  return fontData;
}

let fontBoldData: ArrayBuffer | null = null;

async function loadFontBold(): Promise<ArrayBuffer> {
  if (fontBoldData) return fontBoldData;
  fontBoldData = await fetchFont(
    "https://fonts.gstatic.com/s/jetbrainsmono/v24/tDbY2o-flEEny0FZhsfKu5WU4zr3E_BX0PnT8RD8L6tjPQ.ttf"
  );
  return fontBoldData;
}

// CJK fonts — lazy loaded only for Japanese/Chinese screenshots
let notoJPData: ArrayBuffer | null = null;

async function loadNotoJP(): Promise<ArrayBuffer> {
  if (notoJPData) return notoJPData;
  notoJPData = await fetchFont(
    "https://fonts.gstatic.com/s/notosansjp/v56/-F6jfjtqLzI2JPCgQBnw7HFyzSD-AsregP8VFPYk75s.ttf"
  );
  return notoJPData;
}

let notoSCData: ArrayBuffer | null = null;

async function loadNotoSC(): Promise<ArrayBuffer> {
  if (notoSCData) return notoSCData;
  notoSCData = await fetchFont(
    "https://fonts.gstatic.com/s/notosanssc/v40/k3kCo84MPvpLmixcA63oeAL7Iqp5IZJF9bmaGzjCnYw.ttf"
  );
  return notoSCData;
}

export async function renderImage(
  element: Record<string, unknown>,
  width: number,
  height: number,
  lang?: string
): Promise<Uint8Array> {
  const fontLoads: Promise<ArrayBuffer>[] = [loadFont(), loadFontBold()];
  if (lang === "ja") fontLoads.push(loadNotoJP());
  if (lang === "zh-Hans") fontLoads.push(loadNotoSC());

  const loaded = await Promise.all(fontLoads);

  const fonts: { name: string; data: ArrayBuffer; weight: 400 | 700; style: "normal"; lang?: string }[] = [
    { name: "JetBrains Mono", data: loaded[0], weight: 400, style: "normal" },
    { name: "JetBrains Mono", data: loaded[1], weight: 700, style: "normal" },
  ];

  if (lang === "ja") {
    fonts.push({ name: "Noto Sans JP", data: loaded[2], weight: 700, style: "normal", lang: "ja-JP" });
  }
  if (lang === "zh-Hans") {
    fonts.push({ name: "Noto Sans SC", data: loaded[2], weight: 700, style: "normal", lang: "zh-CN" });
  }

  const svg = await satori(element, { width, height, fonts });

  const resvg = new Resvg(svg, {
    fitTo: { mode: "width", value: width },
  });
  return resvg.render().asPng();
}

export function trayIconBase64(variant: "full" | "alert" = "full"): string {
  const filename = variant === "alert" ? "better-resource-monitor-alert.png" : "better-resource-monitor.png";
  const imgPath = join(process.cwd(), "public", filename);
  const buf = readFileSync(imgPath);
  return `data:image/png;base64,${buf.toString("base64")}`;
}
