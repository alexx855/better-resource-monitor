import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { requiresMacOSChecks } from "./classify-ci-changes.mjs";

const workflow = await readFile(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");

test("website, documentation and generated schemas avoid macOS builds", () => {
  assert.equal(requiresMacOSChecks([
    "www/src/pages/index.astro", "README.es.md", "CONTRIBUTING.md",
    "images/appstore/en/simplicity.png", "src-tauri/gen/schemas/desktop-schema.json",
    ".github/dependabot.yml", "scripts/verify-pages-deployment.mjs",
  ]), false);
  assert.equal(requiresMacOSChecks([]), false);
});

test("app code, dependencies and build tooling retain macOS validation", () => {
  for (const file of [
    "src-tauri/src/lib.rs", "src-tauri/Cargo.lock", "src-tauri/PrivacyInfo.xcprivacy",
    "src-tauri/icons/icon.icns", "package.json", "pnpm-lock.yaml", "pnpm-workspace.yaml",
    ".npmrc", ".cargo/config.toml", "rust-toolchain.toml",
    ".github/workflows/ci.yml", ".github/workflows/testflight.yml",
    ".github/workflows/release.yml", ".github/workflows/direct-download.yml",
    ".github/actions/build-direct-download/action.yml", "scripts/build-appstore-bundle.sh",
    "scripts/setup-appstore-signing.sh", "scripts/setup-developer-id-signing.sh",
    "scripts/upload-appstore-package.sh", "scripts/classify-ci-changes.mjs",
    "scripts/classify-testflight-changes.mjs",
  ]) {
    assert.equal(requiresMacOSChecks(["www/src/pages/index.astro", file]), true, file);
  }
});

test("the classifier emits workflow outputs and preserves removed app paths", () => {
  for (const [paths, expected] of [
    ["www/archived-lib.rs\nsrc-tauri/src/lib.rs\n", "macos=true\n"],
    ["README.md\n", "macos=false\n"],
  ]) {
    assert.equal(execFileSync(process.execPath, [new URL("./classify-ci-changes.mjs", import.meta.url).pathname], {
      input: paths, encoding: "utf8",
    }), expected);
  }
});

test("manual dispatches and new branches fail open to full macOS checks", () => {
  const classifier = workflow.match(/- name: Classify macOS checks[\s\S]*?        run: \|\n([\s\S]*?)(?=\n      - name:)/)?.[1]
    .split("\n").map((line) => line.replace(/^          /, "")).join("\n");
  assert.ok(classifier);
  assert.match(classifier, /git diff --name-only --no-renames/);
  assert.match(classifier, /range="\$BASE_SHA\.\.\.\$HEAD_SHA"/);
  assert.match(classifier, /range="\$BASE_SHA\.\.\$HEAD_SHA"/);
  const directory = mkdtempSync(join(tmpdir(), "ci-classifier-test-"));
  try {
    const output = join(directory, "output");
    for (const env of [
      { EVENT_NAME: "workflow_dispatch", BASE_SHA: "existing" },
      { EVENT_NAME: "push", BASE_SHA: "0000000000000000000000000000000000000000" },
      { EVENT_NAME: "push", BASE_SHA: "" },
    ]) {
      writeFileSync(output, "");
      execFileSync("bash", ["-c", classifier], {
        env: { ...process.env, ...env, GITHUB_OUTPUT: output }, encoding: "utf8",
      });
      assert.equal(readFileSync(output, "utf8"), "macos=true\n");
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("the existing required check rejects failures, cancellation and unexpected skips", () => {
  const gate = workflow.match(/- name: Require all applicable checks[\s\S]*?        run: \|\n([\s\S]*?)(?=\n  [a-z][\w-]*:|$)/)?.[1]
    .split("\n").map((line) => line.replace(/^          /, "")).join("\n");
  assert.ok(gate);
  assert.match(workflow, /name: Rust checks\n    needs: \[checks, rust\]\n    if: \$\{\{ always\(\) \}\}/);
  for (const checks of ["success", "failure", "cancelled", "skipped"]) {
    for (const macos of ["true", "false", ""]) {
      for (const rust of ["success", "failure", "cancelled", "skipped"]) {
        const run = () => execFileSync("bash", ["-c", gate], {
          env: { ...process.env, CHECKS_RESULT: checks, MACOS_REQUIRED: macos, RUST_RESULT: rust }, stdio: "pipe",
        });
        const passes = checks === "success" && (
          (macos === "true" && rust === "success") || (macos === "false" && rust === "skipped")
        );
        if (passes) assert.doesNotThrow(run);
        else assert.throws(run, `${checks}/${macos}/${rust}`);
      }
    }
  }
});
