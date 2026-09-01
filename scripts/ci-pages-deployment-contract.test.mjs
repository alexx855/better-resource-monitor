import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

const workflow = await readFile(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");

test("the required Rust checks job waits for and verifies the exact Pages deployment", () => {
  assert.match(workflow, /checks: read/);
  assert.match(workflow, /scripts\/classify-testflight-changes\.test\.mjs/);
  assert.match(workflow, /scripts\/verify-pages-deployment\.test\.mjs/);
  assert.match(workflow, /scripts\/ci-pages-deployment-contract\.test\.mjs/);
  assert.match(workflow, /DEPLOYMENT_SHA: \$\{\{ github\.event\.pull_request\.head\.sha \|\| github\.sha \}\}/);
  assert.match(workflow, /repos\/\$\{GITHUB_REPOSITORY\}\/commits\/\$\{DEPLOYMENT_SHA\}\/check-runs/);
  assert.match(workflow, /select\(\.name == "Cloudflare Pages"\)/);
  assert.match(workflow, /conclusion.*success/);
  assert.match(workflow, /https:\/\/better-resource-monitor\.alexpedersen\.dev/);
  assert.match(workflow, /node scripts\/verify-pages-deployment\.mjs/);

  const resolverIndex = workflow.indexOf("Resolve Cloudflare Pages deployment");
  const verifierIndex = workflow.indexOf("Verify deployed website");
  assert.ok(resolverIndex > -1 && verifierIndex > resolverIndex);
});
