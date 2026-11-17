import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import test from "node:test";

const workflow = await readFile(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");

test("the required check rejects failures, cancellation, and unexpected skips", () => {
  const gate = workflow.match(/- name: Require website and macOS checks[\s\S]*?        run: \|\n([\s\S]*?)(?=\n  [a-z][\w-]*:|$)/)?.[1]
    .split("\n").map((line) => line.replace(/^          /, "")).join("\n");
  assert.ok(gate);
  assert.match(workflow, /name: Rust checks\n    needs: \[checks, rust\]\n    if: \$\{\{ always\(\) \}\}/);
  for (const checks of ["success", "failure", "cancelled", "skipped"]) {
    for (const rust of ["success", "failure", "cancelled", "skipped"]) {
      const run = () => execFileSync("bash", ["-c", gate], {
        env: { ...process.env, CHECKS_RESULT: checks, RUST_RESULT: rust }, stdio: "pipe",
      });
      if (checks === "success" && rust === "success") assert.doesNotThrow(run);
      else assert.throws(run, `${checks}/${rust}`);
    }
  }
});
