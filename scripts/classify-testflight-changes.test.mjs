import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, renameSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { findAppImpactingFile } from "./classify-testflight-changes.mjs";

const workflow = await readFile(new URL("../.github/workflows/pre-merge-testflight-gate.yml", import.meta.url), "utf8");

test("the gate classifies immutable event commits despite a later docs-only head", (t) => {
  assert.match(workflow, /git fetch --no-tags origin "\$BASE_SHA" "\$HEAD_SHA"/);
  assert.doesNotMatch(workflow, /pulls\/.*\/files/);
  const diff = workflow.match(/app_file="\$\((git diff [^|]+)\|/)?.[1];
  assert.ok(diff);
  const dir = mkdtempSync(join(tmpdir(), "brm-gate-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const git = (...args) => execFileSync("git", args, { cwd: dir, encoding: "utf8" }).trim();
  git("init", "-q");
  git("config", "user.name", "Fixture");
  git("config", "user.email", "fixture@example.invalid");
  mkdirSync(join(dir, "src-tauri"));
  mkdirSync(join(dir, "www"));
  writeFileSync(join(dir, "src-tauri/lib.rs"), "app");
  git("add", ".");
  git("commit", "-qm", "base");
  const base = git("rev-parse", "HEAD");
  renameSync(join(dir, "src-tauri/lib.rs"), join(dir, "www/archived-lib.rs"));
  writeFileSync(join(dir, "src-tauri/ñ.rs"), "unicode");
  writeFileSync(join(dir, "src-tauri/line\nbreak.rs"), "newline");
  git("add", "-A");
  git("commit", "-qm", "move app source");
  const head = git("rev-parse", "HEAD");
  git("checkout", "-q", base);
  mkdirSync(join(dir, "www"), { recursive: true });
  writeFileSync(join(dir, "www/README.md"), "docs only");
  git("add", ".");
  git("commit", "-qm", "later replacement head");
  const paths = execFileSync("bash", ["-c", diff], {
    cwd: dir, encoding: "utf8", env: { ...process.env, BASE_SHA: base, HEAD_SHA: head },
  }).split("\0").filter(Boolean);
  assert.deepEqual(paths, ["src-tauri/lib.rs", "src-tauri/line\nbreak.rs", "src-tauri/ñ.rs", "www/archived-lib.rs"]);
  assert.equal(findAppImpactingFile(paths), "src-tauri/lib.rs");
});

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
