import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";

import { findAppImpactingFile } from "./classify-testflight-changes.mjs";

test("identifies Tauri source and configuration changes", () => {
  assert.equal(findAppImpactingFile(["src-tauri/src/lib.rs"]), "src-tauri/src/lib.rs");
  assert.equal(findAppImpactingFile(["src-tauri/Cargo.toml"]), "src-tauri/Cargo.toml");
  assert.equal(findAppImpactingFile(["src-tauri/tauri.conf.json"]), "src-tauri/tauri.conf.json");
});

test("excludes CI/CD, scripts, and website changes", () => {
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

test("NUL-delimited classification preserves Unicode, newline, and quoted paths", () => {
  for (const path of ["src-tauri/ñ.rs", "src-tauri/line\nbreak.rs", 'src-tauri/"quoted".rs']) {
    const result = execFileSync("node", [new URL("./classify-testflight-changes.mjs", import.meta.url).pathname, "--null"], {
      input: `www/docs\0${path}\0`, encoding: "utf8",
    });
    assert.equal(result, path);
  }
  assert.equal(execFileSync("node", [new URL("./classify-testflight-changes.mjs", import.meta.url).pathname, "--null"], {
    input: "www/line\nsrc-tauri/fake\0", encoding: "utf8",
  }), "");
});
