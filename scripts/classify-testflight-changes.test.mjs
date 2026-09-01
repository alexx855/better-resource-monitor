import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

import { findAppImpactingFile } from "./classify-testflight-changes.mjs";

const workflow = await readFile(new URL("../.github/workflows/pre-merge-testflight-gate.yml", import.meta.url), "utf8");

test("requires TestFlight for Tauri source and configuration changes", () => {
  assert.equal(findAppImpactingFile(["src-tauri/src/lib.rs"]), "src-tauri/src/lib.rs");
  assert.equal(findAppImpactingFile(["src-tauri/Cargo.toml"]), "src-tauri/Cargo.toml");
  assert.equal(findAppImpactingFile(["src-tauri/tauri.conf.json"]), "src-tauri/tauri.conf.json");
});

test("skips TestFlight for CI/CD, scripts, and website changes", () => {
  assert.equal(
    findAppImpactingFile([
      ".github/workflows/ci.yml",
      "scripts/verify-pages-deployment.mjs",
      "www/src/layouts/Layout.astro",
    ]),
    undefined,
  );
});

test("keeps generated Tauri schemas out of the app-impacting set", () => {
  assert.equal(findAppImpactingFile(["src-tauri/gen/schemas/desktop-schema.json"]), undefined);
});

test("finds a Tauri change in a mixed pull request", () => {
  assert.equal(
    findAppImpactingFile([".github/workflows/ci.yml", "www/src/pages/index.astro", "src-tauri/src/main.rs"]),
    "src-tauri/src/main.rs",
  );
});

test("the trusted gate invokes the classifier from the default branch", () => {
  assert.match(workflow, /ref: \$\{\{ github\.event\.repository\.default_branch \}\}/);
  assert.match(workflow, /sparse-checkout: scripts\/classify-testflight-changes\.mjs/);
  assert.match(workflow, /node scripts\/classify-testflight-changes\.mjs/);
  assert.match(workflow, /Skipped: no Tauri app files changed/);
});
