import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
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

test("fork PRs omit only unsupported Pages preview steps", () => {
  const condition = "if: github.event_name != 'pull_request' || github.event.pull_request.head.repo.full_name == github.repository";
  const pagesSteps = workflow.slice(workflow.indexOf("      - name: Resolve Cloudflare Pages deployment"));
  assert.equal(pagesSteps.split(condition).length - 1, 2);
  assert.ok(!workflow.slice(0, workflow.indexOf("      - name: Resolve Cloudflare Pages deployment")).includes(condition));
});

test("main and same-repository PRs resolve the immutable URL from their checked SHA", () => {
  const resolver = workflow.match(/- name: Resolve Cloudflare Pages deployment[\s\S]*?        run: \|\n([\s\S]*?)(?=\n      - name:)/)?.[1]
    .split("\n").map((line) => line.replace(/^          /, "")).join("\n");
  assert.ok(resolver);
  const directory = mkdtempSync(join(tmpdir(), "pages-resolver-test-"));
  try {
    writeFileSync(join(directory, "gh"), '#!/bin/sh\nprintf "%s\\n" "$*" > "$MOCK_REQUEST_LOG"\nprintf "%s\\n" "$MOCK_CHECK_JSON"\n', { mode: 0o755 });
    const immutableUrl = "https://abc123.silicon-monitor.pages.dev";
    for (const ref of ["refs/heads/main", "refs/pull/187/merge"]) {
      const output = join(directory, "output");
      writeFileSync(output, "");
      const env = {
        ...process.env,
        PATH: `${directory}:${process.env.PATH}`,
        GITHUB_REF: ref,
        GITHUB_REPOSITORY: "alexx855/better-resource-monitor",
        DEPLOYMENT_SHA: "checked-commit",
        GITHUB_OUTPUT: output,
        MOCK_REQUEST_LOG: join(directory, "request"),
        MOCK_CHECK_JSON: JSON.stringify({ status: "completed", conclusion: "success", output: { summary: `Deployment: ${immutableUrl}` } }),
      };
      execFileSync("bash", ["-c", resolver], { env, encoding: "utf8" });
      assert.equal(readFileSync(output, "utf8"), `base-url=${immutableUrl}\n`);
      assert.match(readFileSync(env.MOCK_REQUEST_LOG, "utf8"), /commits\/checked-commit\/check-runs/);
      for (const check of [
        { status: "completed", conclusion: "failure", output: { summary: immutableUrl } },
        { status: "completed", conclusion: "success", output: { summary: "https://silicon-monitor.pages.dev" } },
      ]) {
        assert.throws(() => execFileSync("bash", ["-c", resolver], {
          env: { ...env, MOCK_CHECK_JSON: JSON.stringify(check) }, stdio: "pipe",
        }));
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("production verifies the checked deployment before the custom domain and fails closed", () => {
  const verifier = workflow.match(/- name: Verify deployed website[\s\S]*?        run: \|\n([\s\S]*)/)?.[1]
    .split("\n").map((line) => line.replace(/^          /, "")).join("\n");
  assert.ok(verifier);
  const directory = mkdtempSync(join(tmpdir(), "pages-verifier-test-"));
  try {
    writeFileSync(join(directory, "node"), '#!/bin/sh\nprintf "%s\\n" "$PAGES_DEPLOYMENT_BASE_URL" >> "$MOCK_URL_LOG"\n[ "$PAGES_DEPLOYMENT_BASE_URL" != "$MOCK_BROKEN_URL" ]\n', { mode: 0o755 });
    writeFileSync(join(directory, "sleep"), "#!/bin/sh\nexit 0\n", { mode: 0o755 });
    const immutableUrl = "https://abc123.silicon-monitor.pages.dev";
    const productionUrl = "https://better-resource-monitor.alexpedersen.dev";
    const log = join(directory, "urls");
    const env = {
      ...process.env, PATH: `${directory}:${process.env.PATH}`,
      PAGES_DEPLOYMENT_BASE_URL: immutableUrl, MOCK_URL_LOG: log, MOCK_BROKEN_URL: "",
    };
    for (const [ref, expected] of [
      ["refs/heads/main", [immutableUrl, productionUrl]],
      ["refs/pull/187/merge", [immutableUrl]],
    ]) {
      writeFileSync(log, "");
      execFileSync("bash", ["-c", verifier], { env: { ...env, GITHUB_REF: ref }, encoding: "utf8" });
      assert.deepEqual(readFileSync(log, "utf8").trim().split("\n"), expected);
    }
    for (const brokenUrl of [immutableUrl, productionUrl]) {
      writeFileSync(log, "");
      assert.throws(() => execFileSync("bash", ["-c", verifier], {
        env: { ...env, GITHUB_REF: "refs/heads/main", MOCK_BROKEN_URL: brokenUrl }, stdio: "pipe",
      }));
      if (brokenUrl === immutableUrl) {
        assert.ok(readFileSync(log, "utf8").trim().split("\n").every((url) => url === immutableUrl));
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
