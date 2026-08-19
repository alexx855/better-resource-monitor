#!/usr/bin/env node

import { readFileSync } from "node:fs";

const files = readFileSync(0, "utf8")
  .split(/\r?\n/)
  .map((file) => file.trim())
  .filter(Boolean);

// Keep this allowlist conservative: anything outside the public website,
// documentation, or CI metadata is treated as app-impacting by default.
const isNonAppPath = (file) =>
  file.startsWith("www/") ||
  file.startsWith(".github/") ||
  file.startsWith("docs/") ||
  file.startsWith("plans/") ||
  /^README(?:\.[^.]+)?\.md$/.test(file) ||
  /^(?:AGENTS|CONTRIBUTING|DESIGN|SECURITY|CODE_OF_CONDUCT)\.md$/.test(file) ||
  file === "LICENSE" ||
  file === "wrangler.toml";

const requiresTestFlight =
  files.length === 0 || files.some((file) => !isNonAppPath(file));

process.stdout.write(`${requiresTestFlight}\n`);
