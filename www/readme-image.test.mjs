import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const readmes = ["README.md", "README.es.md", "README.pt-br.md", "README.zh-cn.md"];
const imageFilename = "better-resource-monitor-storage.png";
const imagePattern = new RegExp(
  `<img src="https://better-resource-monitor\\.alexpedersen\\.dev/${imageFilename}" alt="Better Resource Monitor" width="(\\d+)" height="(\\d+)">`,
);

test("README tray image is generated with storage and uses its generated dimensions", async () => {
  const script = await readFile(new URL("../scripts/images-helper.sh", import.meta.url), "utf8");
  assert.match(script, new RegExp(`--out www/public/${imageFilename.replace(".", "\\.")}`));
  assert.match(script, /--show-storage true/);
  assert.match(script, /--storage "19\.5 GB"/);

  const png = await readFile(new URL(`./public/${imageFilename}`, import.meta.url));
  assert.equal(png.toString("ascii", 1, 4), "PNG");
  const width = png.readUInt32BE(16);
  const height = png.readUInt32BE(20);

  for (const filename of readmes) {
    const markdown = await readFile(new URL(`../${filename}`, import.meta.url), "utf8");
    const match = markdown.match(imagePattern);
    assert.ok(match, `${filename} should contain the tray image with explicit dimensions`);
    assert.deepEqual(
      [Number(match[1]), Number(match[2])],
      [width, height],
      `${filename} dimensions should match the generated PNG`,
    );
  }
});

test("home page tray image keeps its natural ratio and square corners", async () => {
  const component = await readFile(new URL("./src/components/pages/HomePage.astro", import.meta.url), "utf8");
  const rule = component.match(/p\[align="center"\] img\[alt="Better Resource Monitor"\] \{([^}]*)\}/)?.[1];

  assert.ok(rule, "tray image CSS rule should exist");
  assert.match(rule, /width:\s*auto;/);
  assert.match(rule, /max-width:\s*100%;/);
  assert.match(rule, /height:\s*auto;/);
  assert.match(rule, /border-radius:\s*0;/);
  assert.doesNotMatch(rule, /aspect-ratio:/);
});
