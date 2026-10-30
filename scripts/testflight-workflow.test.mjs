import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
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
    MOCK_SHA: sha, GH_TOKEN: "local-test-only",
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
  assert.doesNotMatch(workflow, /statuses:|issues:|needs\.build\.outputs|Pre-merge TestFlight/);
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
  // Manual builds still restore caches maintained by trusted push CI.
  assert.match(job("build"), /uses: swatinem\/rust-cache@/);
  assert.match(job("build"), /cache: 'pnpm'/);
  assert.match(workflowConfig, /workflow_dispatch:/);
});

test("trusted resolver preserves main, tag, PR and default build-number workflows", async (t) => {
  for (const ref of ["", "main", "v1.2.3", "refs/pull/7/head"]) {
    const { env } = await fixture(t);
    await exec("bash", ["scripts/resolve-testflight-source.sh"], { cwd: root, env: { ...env, INPUT_SOURCE_REF: ref } });
    assert.equal(await readFile(env.GITHUB_OUTPUT, "utf8"), `sha=${sha}\nbuild_number=123\n`);
  }
  const { env } = await fixture(t);
  await exec("bash", ["scripts/resolve-testflight-source.sh"], { cwd: root, env: { ...env, INPUT_BUILD_NUMBER: "" } });
  assert.match(await readFile(env.GITHUB_OUTPUT, "utf8"), /build_number=[0-9]{14}\n/);
});

test("resolver rejects invalid refs, API metadata and non-main dispatch", async (t) => {
  const { env } = await fixture(t);
  const cases = [
    { INPUT_SOURCE_REF: "refs/pull/07/head" },
    { INPUT_SOURCE_REF: "feature" },
    { INPUT_SOURCE_REF: "refs/pull/7/merge" },
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

test("TestFlight is manual, serialized across refs, and detached from PR gates and release dispatches", async () => {
  const triggers = workflow.match(/^on:\n([\s\S]*?)(?=^\S)/m)?.[1];
  assert.ok(triggers);
  assert.deepEqual([...triggers.matchAll(/^  (\w+):/gm)].map((match) => match[1]), ["workflow_dispatch"]);
  assert.match(workflow, /concurrency:\n  group: testflight\n  cancel-in-progress: false/);
  assert.doesNotMatch(workflow, /pr_number:|  pending:|  report:/);
  const workflowsDir = join(root, ".github/workflows");
  const names = await readdir(workflowsDir);
  assert.ok(!names.includes("pre-merge-testflight-gate.yml"));
  for (const name of names.filter((name) => /\.ya?ml$/.test(name) && name !== "testflight.yml")) {
    assert.doesNotMatch(await readFile(join(workflowsDir, name), "utf8"), /testflight\.yml|Pre-merge TestFlight/, name);
  }
});
