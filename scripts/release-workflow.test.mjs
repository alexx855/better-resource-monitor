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

test("release version PR passes exact-head TestFlight before merge", () => {
  const versionBumpJob = workflow.slice(
    workflow.indexOf("  version-bump:"),
    workflow.indexOf("  dispatch-testflight:"),
  );

  const dispatch = versionBumpJob.indexOf(
    '"inputs[source_ref]=refs/pull/${PR_NUMBER}/head"',
  );
  const testflightWatch = versionBumpJob.indexOf(
    'gh run watch "$testflight_run_id" --exit-status',
  );
  const ciWatch = versionBumpJob.indexOf(
    'gh run watch "$ci_run_id" --exit-status',
  );
  const merge = versionBumpJob.indexOf('gh pr merge "$PR_NUMBER"');

  assert.match(versionBumpJob, /permissions:\n      actions: write\n/);
  assert.notEqual(dispatch, -1, "release PR must dispatch its exact head");
  assert.match(versionBumpJob, /dispatch_workflow ci\.yml "\$RELEASE_BRANCH"/);
  assert.ok(
    testflightWatch > dispatch,
    "release must wait for TestFlight processing",
  );
  assert.ok(ciWatch > testflightWatch, "release must wait for trusted CI");
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

async function runReleaseScript(t, script, {
  returnedRunId = "101", ciRunId = "202", dispatchExitCode = 0, watchExitCode = 0,
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
  if (!process.exitCode) process.stdout.write(args.some(arg => arg.endsWith("/ci.yml/dispatches"))
    ? process.env.FIXTURE_CI_RUN_ID : process.env.FIXTURE_RUN_ID);
}
else if (args[0] === "run" && args[1] === "watch") {
  if (!["101", "202"].includes(args[2]) || args[3] !== "--exit-status") process.exitCode = 2;
  else process.exitCode = Number(process.env.FIXTURE_WATCH_EXIT_CODE);
} else process.exitCode = 3;
`, { mode: 0o755 });
  let result;
  try {
    result = await execFileAsync("bash", ["-c", script], {
      env: {
        ...process.env,
        PATH: `${fixtureDir}:${process.env.PATH}`,
        GH_TOKEN: "fixture-token",
        GITHUB_REPOSITORY: "example/monitor",
        GITHUB_OUTPUT: join(fixtureDir, "output"),
        RELEASE_REF: "v1.2.3",
        PR_NUMBER: "7",
        RELEASE_BRANCH: "release/v1.2.3",
        APPSTORE_BUILD_NUMBER: "12345",
        FIXTURE_CALLS: callsPath,
        FIXTURE_RUN_ID: returnedRunId,
        FIXTURE_CI_RUN_ID: ciRunId,
        FIXTURE_DISPATCH_EXIT_CODE: String(dispatchExitCode),
        FIXTURE_WATCH_EXIT_CODE: String(watchExitCode),
      },
    });
  } catch (error) {
    result = error;
  }
  const calls = (await readFile(callsPath, "utf8")).trim().split("\n").map(JSON.parse);
  return { result, calls, outputPath: join(fixtureDir, "output") };
}

function runTestFlightDispatch(t, options) {
  const step = workflow.slice(
    workflow.indexOf("      - name: Dispatch and watch TestFlight upload"),
    workflow.indexOf("  create-github-release:"),
  );
  const script = step.slice(step.indexOf("        run: |\n") + "        run: |\n".length)
    .split("\n").map((line) => line.slice(10)).join("\n");
  return runReleaseScript(t, script, options);
}

function runPremergeDispatch(t, options) {
  const script = workflow.slice(
    workflow.indexOf("          dispatch_workflow() {"),
    workflow.indexOf('          MERGE_STATE=""'),
  ).split("\n").map((line) => line.slice(10)).join("\n");
  assert.notEqual(script, "", "premerge dispatch block must be present");
  return runReleaseScript(t, `set -euo pipefail\n${script}`, options);
}

test("release PR watches its exact dispatched runs without selecting concurrent uploads", async (t) => {
  const { result, calls } = await runPremergeDispatch(t);
  assert.equal(result.code, undefined, result.stdout + result.stderr);
  const [testflight, ci] = calls.filter((args) => args[0] === "api");
  for (const expected of [
    "repos/example/monitor/actions/workflows/testflight.yml/dispatches",
    "POST",
    "Accept: application/vnd.github+json",
    "X-GitHub-Api-Version: 2026-03-10",
    "ref=main",
    "inputs[source_ref]=refs/pull/7/head",
    "inputs[pr_number]=7",
    ".workflow_run_id",
  ]) assert.ok(testflight.includes(expected), `missing TestFlight argument ${expected}`);
  assert.ok(ci.includes("repos/example/monitor/actions/workflows/ci.yml/dispatches"));
  assert.ok(ci.includes("ref=release/v1.2.3"));
  assert.deepEqual(calls.slice(-2), [
    ["run", "watch", "101", "--exit-status"],
    ["run", "watch", "202", "--exit-status"],
  ]);
});

test("release PR stops when CI dispatch returns no run identity", async (t) => {
  const { result, calls } = await runPremergeDispatch(t, { ciRunId: "null" });
  assert.equal(result.code, 1);
  assert.match(result.stderr, /ci.yml dispatch did not return a numeric run ID/);
  assert.equal(calls.length, 2);
});

test("release PR stops when its TestFlight run fails", async (t) => {
  const { result, calls } = await runPremergeDispatch(t, { watchExitCode: 8 });
  assert.equal(result.code, 8);
  assert.deepEqual(calls.at(-1), ["run", "watch", "101", "--exit-status"]);
});

test("release watches its returned TestFlight run for the requested tag and build", async (t) => {
  const job = workflow.slice(workflow.indexOf("  dispatch-testflight:"), workflow.indexOf("  create-github-release:"));
  assert.doesNotMatch(job, /headSha/);
  const { result, calls, outputPath } = await runTestFlightDispatch(t);
  assert.equal(result.code, undefined, result.stdout + result.stderr);
  const dispatch = calls.find((args) => args[0] === "api");
  assert.ok(dispatch, "dispatch must return its own run identity");
  for (const expected of [
    "repos/example/monitor/actions/workflows/testflight.yml/dispatches",
    "POST",
    "Accept: application/vnd.github+json",
    "X-GitHub-Api-Version: 2026-03-10",
    "ref=main",
    "inputs[source_ref]=v1.2.3",
    "inputs[build_number]=12345",
    ".workflow_run_id",
  ]) assert.ok(dispatch.includes(expected), `missing dispatch argument ${expected}`);
  assert.deepEqual(calls.at(-1), ["run", "watch", "101", "--exit-status"]);
  assert.equal(await readFile(outputPath, "utf8"), "testflight_run_id=101\n");
});

test("release fails closed when TestFlight dispatch returns no run identity", async (t) => {
  const { result, calls } = await runTestFlightDispatch(t, { returnedRunId: "null" });
  assert.equal(result.code, 1);
  assert.match(result.stdout + result.stderr, /did not return a numeric run ID/);
  assert.ok(!calls.some((args) => args[0] === "run" && args[1] === "watch"));
});

test("release stops when TestFlight dispatch fails", async (t) => {
  const { result, calls } = await runTestFlightDispatch(t, { dispatchExitCode: 7 });
  assert.equal(result.code, 7);
  assert.equal(calls.length, 1, "a failed dispatch must not select another run");
});

test("release propagates the selected TestFlight run failure", async (t) => {
  const { result, calls } = await runTestFlightDispatch(t, { watchExitCode: 8 });
  assert.equal(result.code, 8);
  assert.deepEqual(calls.at(-1), ["run", "watch", "101", "--exit-status"]);
});
