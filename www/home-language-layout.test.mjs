import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const readmes = [
  ["README.md", "English", 0],
  ["README.es.md", "Español", 1],
  ["README.pt-br.md", "Português (Brasil)", 2],
  ["README.zh-cn.md", "简体中文", 3],
];
const languageParagraphPattern = /<!-- README-LANG-START -->\s*([\s\S]*?)\s*<!-- README-LANG-END -->/;
const linkPattern = /<a href="[^"]+">[^<]+<\/a>/g;

function extractLanguageParagraph(markdown, filename) {
  const paragraph = markdown.match(languageParagraphPattern)?.[1];
  assert.ok(paragraph, `${filename} should contain the language switcher`);
  const contents = paragraph.match(/<p align="center">\s*([\s\S]*?)\s*<\/p>/)?.[1];
  assert.ok(contents, `${filename} should wrap the language switcher in a centered paragraph`);
  return contents;
}

test("language switcher spacing is independent of the hidden current locale", async () => {
  const component = await readFile(new URL("./src/components/pages/HomePage.astro", import.meta.url), "utf8");
  const switcherRule = component.match(
    /\.home-document article > p\[align="center"\]:not\(:has\(img\)\):has\(a\) \{([\s\S]*?)\n    \}/,
  )?.[1];

  assert.ok(switcherRule, "language switcher CSS rule should exist");
  assert.match(switcherRule, /display:\s*flex;/);
  assert.match(switcherRule, /column-gap:\s*0;/, "anonymous text items must not receive a visible horizontal flex gap");
  assert.match(switcherRule, /row-gap:\s*var\(--space-2\);/, "wrapped locale links should keep vertical separation");

  const linkRule = component.match(
    /\.home-document article > p\[align="center"\]:not\(:has\(img\)\):has\(a\) a:not\(:last-of-type\) \{([\s\S]*?)\n    \}/,
  )?.[1];
  assert.ok(linkRule, "language switcher link CSS rule should exist");
  assert.match(linkRule, /margin-inline-end:\s*var\(--space-2\);/);

  for (const [filename, currentLabel, currentIndex] of readmes) {
    const markdown = await readFile(new URL(`../${filename}`, import.meta.url), "utf8");
    const paragraph = extractLanguageParagraph(markdown, filename);
    assert.equal(paragraph.match(linkPattern)?.length, 3, `${filename} should expose three other locales`);

    const labels = paragraph
      .replace(/<a href="[^"]+">([^<]+)<\/a>/g, "$1")
      .split("•")
      .map((label) => label.trim())
      .filter(Boolean);
    assert.equal(labels.indexOf(currentLabel), currentIndex, `${filename} should place its current locale at index ${currentIndex}`);
  }
});
