import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const workflow = await readFile(
  new URL("../.github/workflows/release.yml", import.meta.url),
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
  const watch = versionBumpJob.indexOf('gh run watch "$run_id" --exit-status');
  const merge = versionBumpJob.indexOf('gh pr merge "$PR_NUMBER"');

  assert.match(versionBumpJob, /permissions:\n      actions: write\n/);
  assert.notEqual(dispatch, -1, "release PR must dispatch its exact head");
  assert.ok(watch > dispatch, "release must wait for TestFlight processing");
  assert.ok(merge > watch, "release PR must merge only after TestFlight passes");
  assert.match(versionBumpJob, /--match-head-commit "\$PR_HEAD_SHA"/);
  assert.match(
    versionBumpJob,
    /RELEASE_SHA="\$\(gh pr view "\$PR_NUMBER" --json mergeCommit -q '\.mergeCommit\.oid'\)"/,
  );
});
