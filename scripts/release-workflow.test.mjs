import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

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
    'source_ref="refs/pull/${PR_NUMBER}/head"',
  );
  const testflightWatch = versionBumpJob.indexOf(
    'gh run watch "$testflight_run_id" --exit-status',
  );
  const ciWatch = versionBumpJob.indexOf(
    'gh run watch "$ci_run_id" --exit-status',
  );
  const merge = versionBumpJob.indexOf('gh pr merge "$PR_NUMBER"');

  assert.match(versionBumpJob, /permissions:\n      actions: write\n/);
  assert.match(versionBumpJob, /existing_testflight_run_ids=/);
  assert.match(versionBumpJob, /existing_ci_run_ids=/);
  assert.notEqual(dispatch, -1, "release PR must dispatch its exact head");
  assert.match(versionBumpJob, /gh workflow run ci\.yml --ref "\$RELEASE_BRANCH"/);
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
