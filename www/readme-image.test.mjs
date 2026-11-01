import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const readmes = ["README.md", "README.es.md", "README.pl.md", "README.pt-br.md", "README.zh-cn.md"];
const imageFilename = "better-resource-monitor-storage.png";
const imagePattern = new RegExp(
  `<img src="https://better-resource-monitor\\.alexpedersen\\.dev/${imageFilename}" alt="Better Resource Monitor" width="(\\d+)" height="(\\d+)">`,
);

test("README tray image dimensions match the published PNG", async () => {
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
