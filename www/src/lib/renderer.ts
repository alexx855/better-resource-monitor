import satori from "satori";
import { createRequire } from "node:module";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

// Design tokens (matching Layout.astro CSS vars)
export const colors = {
  bg: "#101214",
  surface: "#171A1D",
  surfaceAlt: "#22272F",
  brand: "#D14715",
  brandMuted: "#321D16",
  text: "#F8FAFC",
  textDim: "#C6CED8",
  textMuted: "#8993A1",
  border: "#3A414A",
  borderStrong: "#566170",
  chrome: "#0B0D0F",
};

export const imageBackgroundStyle = {
  backgroundColor: colors.bg,
  backgroundImage:
    "url(\"data:image/svg+xml,%3Csvg width='56' height='56' viewBox='0 0 56 56' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M0 55.5H56M55.5 0V56' stroke='%2322272F' stroke-width='1'/%3E%3C/svg%3E\")",
  backgroundSize: "56px 56px",
};

// Font cache — vendored files read once per build
let fontData: Buffer | null = null;
const require = createRequire(import.meta.url);

type FontAsset =
  | "JetBrainsMono-Regular.ttf"
  | "JetBrainsMono-Bold.ttf"
  | "NotoSansJP-Bold.ttf"
  | "NotoSansSC-Bold.ttf";

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
  const fontPath = FONT_DIR_CANDIDATES.map((dir) => join(dir, filename)).find(existsSync);
  if (!fontPath) {
    throw new Error(
      `Missing vendored font ${filename}; looked in: ${FONT_DIR_CANDIDATES.join(", ")}`
    );
  }
  return readFileSync(fontPath);
}

async function loadFont(): Promise<Buffer> {
  if (fontData) return fontData;
  fontData = readFontAsset("JetBrainsMono-Regular.ttf");
  return fontData;
}

let fontBoldData: Buffer | null = null;

async function loadFontBold(): Promise<Buffer> {
  if (fontBoldData) return fontBoldData;
  fontBoldData = readFontAsset("JetBrainsMono-Bold.ttf");
  return fontBoldData;
}

// CJK fonts — lazy loaded only for Japanese/Chinese screenshots
let notoJPData: Buffer | null = null;

async function loadNotoJP(): Promise<Buffer> {
  if (notoJPData) return notoJPData;
  notoJPData = readFontAsset("NotoSansJP-Bold.ttf");
  return notoJPData;
}

let notoSCData: Buffer | null = null;

async function loadNotoSC(): Promise<Buffer> {
  if (notoSCData) return notoSCData;
  notoSCData = readFontAsset("NotoSansSC-Bold.ttf");
  return notoSCData;
}

export async function renderImage(
  element: Record<string, unknown>,
  width: number,
  height: number,
  lang?: string
): Promise<Uint8Array> {
  const fontLoads: Promise<Buffer>[] = [loadFont(), loadFontBold()];
  if (lang === "ja") fontLoads.push(loadNotoJP());
  if (lang === "zh-Hans") fontLoads.push(loadNotoSC());

  const loaded = await Promise.all(fontLoads);

  const fonts: { name: string; data: Buffer; weight: 400 | 700; style: "normal"; lang?: string }[] = [
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

  const resvgPackage = "@resvg/resvg-js";
  const { Resvg } = require(resvgPackage);
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
