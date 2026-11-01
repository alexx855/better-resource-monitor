import satori from "satori";
import { createRequire } from "node:module";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

// Colors used by the generated marketing images.
export const colors = {
  bg: "#101214",
  brand: "#D14715",
  text: "#F8FAFC",
};

export const imageBackgroundStyle = {
  backgroundColor: colors.bg,
  backgroundImage:
    "url(\"data:image/svg+xml,%3Csvg width='56' height='56' viewBox='0 0 56 56' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M0 55.5H56M55.5 0V56' stroke='%2322272F' stroke-width='1'/%3E%3C/svg%3E\")",
  backgroundSize: "56px 56px",
};

const require = createRequire(import.meta.url);

type FontAsset =
  | "JetBrainsMono-Regular.ttf"
  | "JetBrainsMono-Bold.ttf"
  | "NotoSansSC-Bold.ttf";

const fontCache = new Map<FontAsset, Buffer>();

// Module-relative resolution covers Astro/Vite prerender builds; the
// process.cwd() fallback matches the trayIconBase64() pattern below for
// plain-Node callers running from www/.
const FONT_DIR_CANDIDATES = [
  typeof import.meta.dirname === "string"
    ? join(import.meta.dirname, "..", "assets", "fonts")
    : null,
  join(process.cwd(), "src", "assets", "fonts"),
].filter((dir): dir is string => dir !== null);

function readFontAsset(filename: FontAsset): Buffer {
  const cached = fontCache.get(filename);
  if (cached) return cached;
  const fontPath = FONT_DIR_CANDIDATES.map((dir) => join(dir, filename)).find(existsSync);
  if (!fontPath) {
    throw new Error(
      `Missing vendored font ${filename}; looked in: ${FONT_DIR_CANDIDATES.join(", ")}`
    );
  }
  const data = readFileSync(fontPath);
  fontCache.set(filename, data);
  return data;
}

export async function renderImage(
  element: Record<string, unknown>,
  width: number,
  height: number,
  lang?: string
): Promise<Uint8Array> {
  const fonts: { name: string; data: Buffer; weight: 400 | 700; style: "normal"; lang?: string }[] = [
    { name: "JetBrains Mono", data: readFontAsset("JetBrainsMono-Regular.ttf"), weight: 400, style: "normal" },
    { name: "JetBrains Mono", data: readFontAsset("JetBrainsMono-Bold.ttf"), weight: 700, style: "normal" },
  ];

  if (lang === "zh-Hans") {
    fonts.push({ name: "Noto Sans SC", data: readFontAsset("NotoSansSC-Bold.ttf"), weight: 700, style: "normal", lang: "zh-CN" });
  }

  const svg = await satori(element, { width, height, fonts });

  const resvgPackage = "@resvg/resvg-js";
  const { Resvg } = require(resvgPackage);
  const resvg = new Resvg(svg, {
    fitTo: { mode: "width", value: width },
  });
  return resvg.render().asPng();
}

export function trayIconBase64(variant: "full" | "alert" = "full"): string {
  const filename = variant === "alert" ? "better-resource-monitor-alert.png" : "better-resource-monitor-storage.png";
  const imgPath = join(process.cwd(), "public", filename);
  const buf = readFileSync(imgPath);
  return `data:image/png;base64,${buf.toString("base64")}`;
}
