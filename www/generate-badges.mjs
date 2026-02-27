#!/usr/bin/env node
// Generates download badge images as WebP and saves to www/public/badges/
// Usage: node www/generate-badges.mjs

import satori from "satori";
import { Resvg } from "@resvg/resvg-js";
import sharp from "sharp";
import { mkdirSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = dirname(fileURLToPath(import.meta.url));
const OUT_DIR = join(ROOT, "public", "badges");

// Apple logo — official paths extracted from Apple's Mac App Store badge SVG
const appleIcon = `data:image/svg+xml;base64,${Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="7.25 7.5 24 24" fill="white"><path d="M24.76888,20.30068a4.94881,4.94881,0,0,1,2.35656-4.15206,5.06566,5.06566,0,0,0-3.99116-2.15768c-1.67924-.17626-3.30719,1.00483-4.1629,1.00483-.87227,0-2.18977-.98733-3.6085-.95814a5.31529,5.31529,0,0,0-4.47292,2.72787c-1.934,3.34842-.49141,8.26947,1.3612,10.97608.9269,1.32535,2.01018,2.8058,3.42763,2.7533,1.38706-.05753,1.9051-.88448,3.5794-.88448,1.65876,0,2.14479.88448,3.591.8511,1.48838-.02416,2.42613-1.33124,3.32051-2.66914a10.962,10.962,0,0,0,1.51842-3.09251A4.78205,4.78205,0,0,1,24.76888,20.30068Z"/><path d="M22.03725,12.21089a4.87248,4.87248,0,0,0,1.11452-3.49062,4.95746,4.95746,0,0,0-3.20758,1.65961,4.63634,4.63634,0,0,0-1.14371,3.36139A4.09905,4.09905,0,0,0,22.03725,12.21089Z"/></svg>`).toString("base64")}`;

// GitHub mark (Invertocat) — white
const githubIcon = `data:image/svg+xml;base64,${Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="white"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>`).toString("base64")}`;

// Ubuntu Circle of Friends logo — official Canonical SVG, white
const ubuntuIcon = `data:image/svg+xml;base64,${Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 60.45 57.87"><circle fill="#fff" cx="9.66" cy="27.41" r="8.12"/><circle fill="#fff" cx="43.48" cy="9.61" r="8.12"/><path fill="#fff" d="M7.34,38.71A26.67,26.67,0,0,0,32.58,55.55a11.47,11.47,0,0,1-2.49-7Q29,48.4,28,48.19A19.62,19.62,0,0,1,14.55,37.83,11.49,11.49,0,0,1,7.34,38.71Z"/><circle fill="#fff" cx="41.65" cy="48.35" r="8.12"/><path fill="#fff" d="M48.9,39.31a11.58,11.58,0,0,1,4,6.25,26.64,26.64,0,0,0,1.36-31.63,11.47,11.47,0,0,1-4.7,5.54A19.73,19.73,0,0,1,48.9,39.31Z"/><path fill="#fff" d="M9.68,15.85a11.44,11.44,0,0,1,2.4.25,11.72,11.72,0,0,1,3.77,1.54,19.73,19.73,0,0,1,16-8.56,12.15,12.15,0,0,1,.22-1.9h0a11.75,11.75,0,0,1,2.34-4.89A26.9,26.9,0,0,0,8.82,15.88C9.11,15.86,9.39,15.85,9.68,15.85Z"/></svg>`).toString("base64")}`;

const badges = {
  appstore: { icon: appleIcon, topText: "DOWNLOAD FROM", bottomText: "Mac App Store" },
  macos: { icon: githubIcon, topText: "DOWNLOAD FROM", bottomText: "GitHub Releases" },
  ubuntu: { icon: ubuntuIcon, topText: "DOWNLOAD FROM", bottomText: "GitHub Releases" },
};

const WIDTH = 1080;
const HEIGHT = 260;
const ICON_SIZE = 128;
const BORDER = 6;
const RADIUS = 26;

async function loadFonts() {
  const bold = await fetch("https://fonts.gstatic.com/s/jetbrainsmono/v24/tDbY2o-flEEny0FZhsfKu5WU4zr3E_BX0PnT8RD8L6tjPQ.ttf").then((r) => r.arrayBuffer());
  return [{ name: "JetBrains Mono", data: bold, weight: 700, style: "normal" }];
}

function buildElement(badge) {
  return {
    type: "div",
    props: {
      style: {
        width: "100%",
        height: "100%",
        display: "flex",
        alignItems: "center",
        fontFamily: "JetBrains Mono",
        backgroundColor: "#000",
        border: `${BORDER}px solid #fff`,
        borderRadius: RADIUS,
        padding: "0 56px",
        gap: 40,
      },
      children: [
        { type: "img", props: { src: badge.icon, width: ICON_SIZE, height: ICON_SIZE } },
        {
          type: "div",
          props: {
            style: { display: "flex", flexDirection: "column" },
            children: [
              {
                type: "div",
                props: { style: { fontSize: 48, fontWeight: 700, color: "#fff", lineHeight: 1.3 }, children: badge.topText },
              },
              {
                type: "div",
                props: { style: { fontSize: 80, fontWeight: 700, color: "#fff", lineHeight: 1.3 }, children: badge.bottomText },
              },
            ],
          },
        },
      ],
    },
  };
}

async function main() {
  const fonts = await loadFonts();
  mkdirSync(OUT_DIR, { recursive: true });

  for (const [name, badge] of Object.entries(badges)) {
    const svg = await satori(buildElement(badge), { width: WIDTH, height: HEIGHT, fonts });
    const png = new Resvg(svg, { fitTo: { mode: "width", value: WIDTH } }).render().asPng();
    const webp = await sharp(png).webp({ lossless: true }).toBuffer();
    const outPath = join(OUT_DIR, `${name}.webp`);
    writeFileSync(outPath, webp);
    console.log(`${name}.webp (${webp.length} bytes)`);
  }
}

main();
