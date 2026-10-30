import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

const workflow = await readFile(
  new URL("../.github/workflows/release.yml", import.meta.url),
  "utf8",
);
const ciWorkflow = await readFile(
  new URL("../.github/workflows/ci.yml", import.meta.url),
  "utf8",
);

test("release version PR passes CI before merge without a TestFlight upload", () => {
  const versionBumpJob = workflow.slice(
    workflow.indexOf("  version-bump:"),
    workflow.indexOf("  create-github-release:"),
  );

  const ciWatch = versionBumpJob.indexOf(
    'gh run watch "$ci_run_id" --exit-status',
  );
  const merge = versionBumpJob.indexOf('gh pr merge "$PR_NUMBER"');

  assert.match(versionBumpJob, /permissions:\n      actions: write\n/);
  assert.match(versionBumpJob, /actions\/workflows\/ci\.yml\/dispatches/);
  assert.ok(ciWatch > -1, "release must wait for trusted CI");
  assert.doesNotMatch(workflow, /testflight|TESTFLIGHT|APPSTORE_BUILD_NUMBER/i);
  assert.ok(merge > ciWatch, "release PR must merge only after all gates pass");
  assert.match(versionBumpJob, /--match-head-commit "\$PR_HEAD_SHA"/);
  assert.match(
    versionBumpJob,
    /if \[ "\$PR_STATE" = "MERGED" \] && \[ -n "\$RELEASE_SHA" \]/,
  );
});

test("CI supports trusted release-branch dispatch", () => {
  assert.match(ciWorkflow, /on:\n  pull_request:\n  workflow_dispatch:\n/);
});

async function runPremergeDispatch(t, {
  returnedRunId = "202", dispatchExitCode = 0, watchExitCode = 0,
} = {}) {
  const fixtureDir = await mkdtemp(join(tmpdir(), "brm-release-dispatch-"));
  t.after(() => rm(fixtureDir, { recursive: true, force: true }));
  const callsPath = join(fixtureDir, "calls.jsonl");
  await writeFile(join(fixtureDir, "gh"), `#!${process.execPath}
const fs = require("node:fs");
const args = process.argv.slice(2);
fs.appendFileSync(process.env.FIXTURE_CALLS, JSON.stringify(args) + "\\n");
if (args[0] === "api") {
  process.exitCode = Number(process.env.FIXTURE_DISPATCH_EXIT_CODE);
  if (!process.exitCode) process.stdout.write(process.env.FIXTURE_RUN_ID);
} else if (args[0] === "run" && args[1] === "watch") {
  if (args[2] !== "202" || args[3] !== "--exit-status") process.exitCode = 2;
  else process.exitCode = Number(process.env.FIXTURE_WATCH_EXIT_CODE);
} else process.exitCode = 3;
`, { mode: 0o755 });
  const start = workflow.indexOf('          ci_run_id="$(gh api');
  const end = workflow.indexOf('          MERGE_STATE=""', start);
  assert.ok(start !== -1 && end > start, "premerge dispatch block must be present");
  const script = workflow.slice(start, end)
    .split("\n").map((line) => line.slice(10)).join("\n");
  let result;
  try {
    result = await execFileAsync("bash", ["-c", `set -euo pipefail\n${script}`], {
      env: {
        ...process.env,
        PATH: `${fixtureDir}:${process.env.PATH}`,
        GH_TOKEN: "fixture-token",
        GITHUB_REPOSITORY: "example/monitor",
        RELEASE_BRANCH: "release/v1.2.3",
        FIXTURE_CALLS: callsPath,
        FIXTURE_RUN_ID: returnedRunId,
        FIXTURE_DISPATCH_EXIT_CODE: String(dispatchExitCode),
        FIXTURE_WATCH_EXIT_CODE: String(watchExitCode),
      },
    });
  } catch (error) {
    result = error;
  }
  const calls = (await readFile(callsPath, "utf8")).trim().split("\n").map(JSON.parse);
  return { result, calls };
}

test("release PR watches only the CI run returned by its dispatch", async (t) => {
  const { result, calls } = await runPremergeDispatch(t);
  assert.equal(result.code, undefined, result.stdout + result.stderr);
  assert.equal(calls.length, 2);
  for (const expected of [
    "repos/example/monitor/actions/workflows/ci.yml/dispatches",
    "POST",
    "Accept: application/vnd.github+json",
    "X-GitHub-Api-Version: 2026-03-10",
    "ref=release/v1.2.3",
    ".workflow_run_id",
  ]) assert.ok(calls[0].includes(expected), `missing CI dispatch argument ${expected}`);
  assert.deepEqual(calls[1], ["run", "watch", "202", "--exit-status"]);
});

test("release PR stops when CI dispatch returns no valid run identity", async (t) => {
  for (const returnedRunId of ["", "null", "not-a-run-id", "202\n203"]) {
    const { result, calls } = await runPremergeDispatch(t, { returnedRunId });
    assert.equal(result.code, 1);
    assert.match(result.stderr, /CI dispatch did not return a numeric run ID/);
    assert.equal(calls.length, 1, "invalid dispatch must not select another run");
  }
});

test("release PR stops when CI dispatch fails", async (t) => {
  const { result, calls } = await runPremergeDispatch(t, { dispatchExitCode: 7 });
  assert.equal(result.code, 7);
  assert.equal(calls.length, 1);
});

test("release PR propagates its CI run failure", async (t) => {
  const { result, calls } = await runPremergeDispatch(t, { watchExitCode: 8 });
  assert.equal(result.code, 8);
  assert.deepEqual(calls.at(-1), ["run", "watch", "202", "--exit-status"]);
});
