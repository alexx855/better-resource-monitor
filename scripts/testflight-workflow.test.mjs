import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import test from "node:test";

const exec = promisify(execFile);
const root = fileURLToPath(new URL("../", import.meta.url));
const workflow = await readFile(join(root, ".github/workflows/testflight.yml"), "utf8");
const sha = "0123456789abcdef0123456789abcdef01234567";
function job(name) {
  const start = workflow.indexOf(`\n  ${name}:\n`);
  assert.notEqual(start, -1);
  const end = workflow.slice(start + 1).search(/\n  [a-z-]+:\n/);
  return end === -1 ? workflow.slice(start) : workflow.slice(start, start + 1 + end);
}
function runBlock(section) {
  const start = section.indexOf("        run: |\n");
  assert.notEqual(start, -1);
  return section.slice(start + "        run: |\n".length).split("\n")
    .filter((line) => line.startsWith("          ")).map((line) => line.slice(10)).join("\n");
}
async function fixture(t) {
  const dir = await mkdtemp(join(tmpdir(), "brm-testflight-"));
  t.after(() => rm(dir, { recursive: true, force: true }));
  await mkdir(join(dir, "bin"));
  await writeFile(join(dir, "bin/gh"), `#!/usr/bin/env node
const fs = require('node:fs');
fs.appendFileSync(process.env.CALLS, JSON.stringify(process.argv.slice(2)) + '\\n');
if (process.argv[3].includes('/pulls/')) console.log(process.env.PR_JSON);
else if (process.argv[3].includes('/commits/')) console.log(process.env.MOCK_SHA);
`, { mode: 0o755 });
  const env = {
    ...process.env, PATH: `${join(dir, "bin")}:${process.env.PATH}`,
    CALLS: join(dir, "calls"), GITHUB_OUTPUT: join(dir, "output"),
    GITHUB_REF: "refs/heads/main", GITHUB_REPOSITORY: "owner/repo",
    GITHUB_RUN_NUMBER: "1", INPUT_SOURCE_REF: "", INPUT_BUILD_NUMBER: "123",
    PR_NUMBER: "", MOCK_SHA: sha, GH_TOKEN: "local-test-only",
    PR_JSON: JSON.stringify({ number: 7, state: "open", base: { repo: { full_name: "owner/repo" } }, head: { sha } }),
  };
  return { dir, env };
}

test("untrusted build has no write authority, metadata outputs, or privileged reporter", () => {
  assert.match(workflow, /permissions:\n  contents: read\n\n/);
  const build = job("build");
  assert.match(build, /permissions:\n      contents: read\n/);
  assert.doesNotMatch(build, /GH_TOKEN|github\.token|secrets\.|outputs:|statuses:|issues:|pull-requests:/);
  assert.match(build, /ref: \$\{\{ needs.resolve.outputs.sha \}\}/);
  for (const name of ["pending", "report"]) {
    assert.match(job(name), /runs-on: ubuntu-latest/);
    assert.doesNotMatch(job(name), /checkout|download-artifact|needs.build.outputs/);
  }
  assert.doesNotMatch(workflow, /needs\.build\.outputs/);
  assert.match(job("report"), /always\(\).*needs.resolve.result == 'success'/);
  for (const match of workflow.matchAll(/        run: ([\s\S]*?)(?=\n      - name:|\n  [a-z-]+:|$)/g)) {
    // Expressions become shell source before execution: forbid at every run sink.
    assert.doesNotMatch(match[1], /\$\{\{/);
  }
  const sign = job("sign-upload");
  assert.ok(sign.indexOf("Validate resolved metadata") < sign.indexOf("Prepare App Store credentials"));
  assert.doesNotMatch(sign, /GH_TOKEN|statuses: write|issues: write/);
  assert.doesNotMatch(workflow, /ref: main\n/);
});

test("TestFlight cannot grant cache-write tokens to contributor code", () => {
  const workflowConfig = workflow.slice(0, workflow.indexOf("\njobs:\n"));
  assert.match(workflowConfig, /^cache-mode: read$/m);
  // No job can override the cap, including the job that runs contributor code.
  assert.doesNotMatch(workflow.slice(workflow.indexOf("\njobs:\n")), /^ +cache-mode:/m);
  // Keep restore support and dispatch/watch compatibility for release callers.
  assert.match(job("build"), /uses: swatinem\/rust-cache@/);
  assert.match(job("build"), /cache: 'pnpm'/);
  assert.match(workflowConfig, /workflow_dispatch:/);
});

test("trusted resolver preserves main, tag, PR and default build-number workflows", async (t) => {
  for (const [ref, pr] of [["", ""], ["v1.2.3", ""], ["refs/pull/7/head", "7"], ["refs/pull/7/head", ""]]) {
    const { env } = await fixture(t);
    await exec("bash", ["scripts/resolve-testflight-source.sh"], { cwd: root, env: { ...env, INPUT_SOURCE_REF: ref, PR_NUMBER: pr } });
    assert.equal(await readFile(env.GITHUB_OUTPUT, "utf8"), `sha=${sha}\nbuild_number=123\npr_number=${pr}\n`);
  }
  const { env } = await fixture(t);
  await exec("bash", ["scripts/resolve-testflight-source.sh"], { cwd: root, env: { ...env, INPUT_BUILD_NUMBER: "" } });
  assert.match(await readFile(env.GITHUB_OUTPUT, "utf8"), /build_number=[0-9]{14}\n/);
});

test("resolver rejects unrelated PR attestation, invalid API metadata and non-main dispatch", async (t) => {
  const { env } = await fixture(t);
  const cases = [
    { INPUT_SOURCE_REF: "refs/pull/7/head", PR_NUMBER: "8" },
    { INPUT_SOURCE_REF: "main", PR_NUMBER: "7" },
    { INPUT_SOURCE_REF: "v1.0.0", PR_NUMBER: "7" },
    { INPUT_SOURCE_REF: "refs/pull/07/head", PR_NUMBER: "7" },
    { INPUT_SOURCE_REF: "refs/pull/7/head", PR_NUMBER: "7\n" },
    { GITHUB_REF: "refs/heads/feature" },
    { INPUT_SOURCE_REF: "refs/pull/7/head", PR_JSON: JSON.stringify({ number: 7, state: "closed", base: { repo: { full_name: "owner/repo" } }, head: { sha } }) },
    { INPUT_SOURCE_REF: "refs/pull/7/head", PR_JSON: JSON.stringify({ number: 7, state: "open", base: { repo: { full_name: "other/repo" } }, head: { sha } }) },
    { MOCK_SHA: `${sha}$(printf injected)` },
    { INPUT_BUILD_NUMBER: "123\nsha=forged" },
  ];
  for (const overrides of cases) {
    await assert.rejects(exec("bash", ["scripts/resolve-testflight-source.sh"], { cwd: root, env: { ...env, ...overrides } }));
  }
});

test("full metadata is validated before build and upload side effects", async (t) => {
  const { dir } = await fixture(t);
  const invalid = ["", sha.slice(0, 12), sha.slice(0, 39), `${sha}0`, sha.toUpperCase(), `${sha}$(touch ${join(dir, "executed")})`, `${sha}\nother=value`];
  for (const value of invalid) {
    for (const script of ["build-appstore-bundle.sh", "upload-appstore-package.sh"]) {
      await assert.rejects(exec("bash", [join(root, "scripts", script), dir, "123", value, "unused"], { cwd: dir }));
    }
  }
  for (const value of ["1;echo injected", "1\n2", "-1", "1.0"]) {
    await assert.rejects(exec("bash", [join(root, "scripts/upload-appstore-package.sh"), dir, value, sha, "unused"], { cwd: dir }));
  }
  await assert.rejects(readFile(join(dir, "executed")), { code: "ENOENT" });
});

test("report success/failure/cancellation binds only the originally resolved head", async (t) => {
  const script = runBlock(job("report"));
  for (const [pending, build, upload, state] of [
    ["success", "success", "success", "success"],
    ["success", "failure", "skipped", "failure"],
    ["success", "success", "failure", "failure"],
    ["failure", "skipped", "skipped", "failure"],
    ["success", "cancelled", "skipped", "failure"],
    ["success", "success", "cancelled", "failure"],
  ]) {
    const { env } = await fixture(t);
    const options = { cwd: root, env: { ...env, BUILD_SHA: sha, BUILD_NUMBER: "123", PR_NUMBER: "7", PENDING_RESULT: pending, BUILD_RESULT: build, UPLOAD_RESULT: upload, GITHUB_SERVER_URL: "https://github.com", GITHUB_RUN_ID: "42" } };
    if (state === "success") await exec("bash", ["-c", script], options);
    else await assert.rejects(exec("bash", ["-c", script], options));
    const calls = (await readFile(env.CALLS, "utf8")).trim().split("\n").map(JSON.parse);
    assert.equal(calls[0][1], `repos/owner/repo/statuses/${sha}`);
    assert.ok(calls[0].includes(`state=${state}`));
    assert.equal(calls.length, state === "success" ? 2 : 1);
    if (state === "success") assert.equal(calls[1][1], "repos/owner/repo/issues/7/comments");
  }
});

test("status writers share a queued per-head lock and preserve upload provenance", async (t) => {
  const gate = await readFile(join(root, ".github/workflows/pre-merge-testflight-gate.yml"), "utf8");
  for (const section of [job("pending"), job("report"), gate]) {
    assert.match(section, /group: testflight-status-\$\{\{ github.repository \}\}-\$\{\{ (?:needs.resolve.outputs.sha|github.event.pull_request.head.sha) \}\}/);
    assert.match(section, /queue: max\n      cancel-in-progress: false/);
  }
  // Execute the real gate with local stand-ins. An existing upload lifecycle
  // status must not be rewritten; a classifier/foreign run must be classified.
  const script = runBlock(gate);
  for (const [target, owned, writes] of [
    ["https://github.com/owner/repo/actions/runs/42", "true", false],
    ["https://github.com/owner/repo/actions/runs/42", "false", true],
    ["https://github.com/other/repo/actions/runs/42", "true", true],
    ["", "false", true],
  ]) {
    const { dir, env } = await fixture(t);
    await writeFile(join(dir, "bin/gh"), `#!/usr/bin/env node
const fs = require('node:fs');
const endpoint = process.argv[3];
fs.appendFileSync(process.env.CALLS, JSON.stringify(process.argv.slice(2)) + '\\n');
if (endpoint.endsWith('/status')) console.log(process.env.MOCK_TARGET);
else if (endpoint.includes('/actions/runs/')) console.log(process.env.MOCK_OWNED);
`, { mode: 0o755 });
    await writeFile(join(dir, "bin/git"), "#!/bin/bash\nif [ \"$1\" = diff ]; then printf 'src-tauri/app.rs\\0'; fi\n", { mode: 0o755 });
    await mkdir(join(dir, "scripts"));
    await writeFile(join(dir, "scripts/classify-testflight-changes.mjs"), await readFile(join(root, "scripts/classify-testflight-changes.mjs")));
    await exec("bash", ["-c", script], { cwd: dir, env: {
      ...env, REPOSITORY: "owner/repo", BASE_SHA: sha, HEAD_SHA: sha,
      GITHUB_SERVER_URL: "https://github.com", TARGET_URL: "https://github.com/owner/repo/actions/runs/99",
      MOCK_TARGET: target, MOCK_OWNED: owned,
    } });
    const calls = (await readFile(env.CALLS, "utf8")).trim().split("\n").map(JSON.parse);
    const statusWrites = calls.filter((args) => args[1].includes("/statuses/"));
    assert.equal(statusWrites.length, writes ? 1 : 0);
    if (writes) assert.ok(statusWrites[0].includes("state=pending"));
  }
});
