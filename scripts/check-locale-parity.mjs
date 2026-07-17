#!/usr/bin/env node
// Enforces the structural half of the locale invariant documented in
// AGENTS.md: website locales (www/src/lib/marketing-copy.json) drive the
// canonical set, and every other locale-bearing surface (content collections,
// READMEs, the app's i18n prefix map) must cover exactly that set.
// Translated wording still needs human review.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(fileURLToPath(import.meta.url), "..", "..");

function fail(message) {
  console.error(`Locale parity FAILED: ${message}`);
  process.exit(1);
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

// 1. Canonical locale set comes from marketing-copy.json.
const marketingCopyPath = join(ROOT, "www", "src", "lib", "marketing-copy.json");
const marketingCopy = readJson(marketingCopyPath);
const locales = Object.keys(marketingCopy);
if (locales.length === 0) {
  fail(`no locales found in ${marketingCopyPath}`);
}
if (!locales.includes("en")) {
  fail(`canonical locale set ${JSON.stringify(locales)} must include "en"`);
}
console.log(`canonical locales: ${locales.join(", ")}`);

// 2. Every locale object mirrors en's key set (one level deep for nested objects).
const enCopy = marketingCopy.en;
for (const locale of locales) {
  const copy = marketingCopy[locale];
  const expectedKeys = Object.keys(enCopy).sort().join(",");
  const actualKeys = Object.keys(copy).sort().join(",");
  if (actualKeys !== expectedKeys) {
    fail(
      `marketing-copy.json "${locale}" keys [${actualKeys}] differ from "en" keys [${expectedKeys}]`
    );
  }
  for (const key of Object.keys(enCopy)) {
    if (typeof enCopy[key] !== "object" || enCopy[key] === null) continue;
    const expectedNested = Object.keys(enCopy[key]).sort().join(",");
    const actualNested = Object.keys(copy[key] ?? {}).sort().join(",");
    if (actualNested !== expectedNested) {
      fail(
        `marketing-copy.json "${locale}.${key}" keys [${actualNested}] differ from "en.${key}" keys [${expectedNested}]`
      );
    }
  }
}
console.log("marketing-copy.json locale shapes match");

// 3. Content collections have exactly one file per canonical locale.
function assertLocaleFiles(dir, extension, label) {
  const expected = locales.map((locale) => `${locale}${extension}`).sort();
  let actual;
  try {
    actual = readdirSync(dir)
      .filter((name) => name.endsWith(extension))
      .sort();
  } catch {
    fail(`${label}: directory not found at ${dir}`);
  }
  const missing = expected.filter((name) => !actual.includes(name));
  const extra = actual.filter((name) => !expected.includes(name));
  if (missing.length > 0) {
    fail(`${label}: missing locale file(s): ${missing.join(", ")}`);
  }
  if (extra.length > 0) {
    fail(`${label}: unexpected locale file(s) not in marketing-copy.json: ${extra.join(", ")}`);
  }
}

const contentRoot = join(ROOT, "www", "src", "content");
for (const collection of ["home", "faq", "ui"]) {
  assertLocaleFiles(join(contentRoot, collection), ".json", `content/${collection}`);
}
for (const slug of readdirSync(join(contentRoot, "comparisons"))) {
  assertLocaleFiles(join(contentRoot, "comparisons", slug), ".json", `content/comparisons/${slug}`);
}
for (const doc of readdirSync(join(contentRoot, "legal"))) {
  assertLocaleFiles(join(contentRoot, "legal", doc), ".md", `content/legal/${doc}`);
}
console.log("content collections cover every locale");

// 4. Root READMEs (they feed the website home body).
for (const locale of locales) {
  const readme = locale === "en" ? "README.md" : `README.${locale}.md`;
  if (!existsSync(join(ROOT, readme))) {
    fail(`missing ${readme} for locale "${locale}"`);
  }
}
console.log("READMEs cover every locale");

// 5. FAQ item ids are identical (same ids, same order) across locales.
const enFaqIds = readJson(join(contentRoot, "faq", "en.json")).items.map((item) => item.id);
for (const locale of locales) {
  const ids = readJson(join(contentRoot, "faq", `${locale}.json`)).items.map((item) => item.id);
  if (ids.join(",") !== enFaqIds.join(",")) {
    fail(
      `faq/${locale}.json item ids [${ids.join(", ")}] differ from en [${enFaqIds.join(", ")}]`
    );
  }
}
console.log(`faq item ids match across locales (${enFaqIds.length} items)`);

// 6. Comparison tables have aligned row counts within each file and across locales.
for (const slug of readdirSync(join(contentRoot, "comparisons"))) {
  const enTable = readJson(join(contentRoot, "comparisons", slug, "en.json")).table;
  for (const locale of locales) {
    const file = `comparisons/${slug}/${locale}.json`;
    const { table } = readJson(join(contentRoot, "comparisons", slug, `${locale}.json`));
    if (
      table.features.length !== table.productValues.length ||
      table.features.length !== table.competitorValues.length
    ) {
      fail(
        `${file}: table lengths differ (features=${table.features.length}, productValues=${table.productValues.length}, competitorValues=${table.competitorValues.length})`
      );
    }
    if (table.features.length !== enTable.features.length) {
      fail(
        `${file}: ${table.features.length} table rows but en has ${enTable.features.length}`
      );
    }
  }
}
console.log("comparison table rows match across locales");

// 7. The app's i18n prefix map covers every canonical locale. This is a
// source-text check on language_for_locale in src-tauri/src/i18n.rs.
const i18nSource = readFileSync(join(ROOT, "src-tauri", "src", "i18n.rs"), "utf8");
const matchBlock = i18nSource.match(/match prefix\.as_str\(\) \{([\s\S]*?)\}/);
const appPrefixes = [...(matchBlock?.[1] ?? "").matchAll(/"([a-z-]+)" => Some\(/g)].map(
  (m) => m[1]
);
if (appPrefixes.length === 0) {
  fail(
    "could not parse language_for_locale in src-tauri/src/i18n.rs — update scripts/check-locale-parity.mjs"
  );
}
for (const locale of locales) {
  const prefix = locale.split("-")[0];
  if (!appPrefixes.includes(prefix)) {
    fail(
      `app i18n.rs language_for_locale has no mapping for prefix "${prefix}" (site locale "${locale}"); known prefixes: ${appPrefixes.join(", ")}`
    );
  }
}
console.log(`app i18n prefixes cover every locale (${appPrefixes.join(", ")})`);

console.log(`Locale parity OK (${locales.length} locales)`);
