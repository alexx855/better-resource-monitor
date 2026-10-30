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
  assert.match(versionBumpJob, /existing_ci_run_ids=/);
  assert.match(versionBumpJob, /gh workflow run ci\.yml --ref "\$RELEASE_BRANCH"/);
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
