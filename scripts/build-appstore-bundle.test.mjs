import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import test from "node:test";

const exec = promisify(execFile);
const buildScript = fileURLToPath(new URL("./build-appstore-bundle.sh", import.meta.url));

test("unsigned builds of older tags use the trusted override without a provisioning profile", async (t) => {
  const fixture = await mkdtemp(join(tmpdir(), "brm-appstore-build-"));
  t.after(() => rm(fixture, { recursive: true, force: true }));
  const source = join(fixture, "old release");
  const bin = join(fixture, "bin");
  for (const directory of [bin, "src-tauri/src", ".github", "scripts"].map((path) =>
    path === bin ? bin : join(source, path))) {
    await mkdir(directory, { recursive: true });
  }
  await writeFile(join(source, "src-tauri/Cargo.toml"), "");
  await writeFile(join(source, "src-tauri/tauri.conf.json"), JSON.stringify({ version: "1.0.0" }));
  await writeFile(join(source, "src-tauri/tauri.appstore.conf.json"), JSON.stringify({
    bundle: { macOS: { files: { "embedded.provisionprofile": "embedded.provisionprofile" } } },
  }));
  // Stand in for the expensive compiler; reject the old tag's profile-dependent
  // config and leave an executable marker for the build script's real checks.
  await writeFile(join(bin, "pnpm"), `#!/usr/bin/env node
const { readFileSync, mkdirSync, writeFileSync } = require('node:fs');
const { join } = require('node:path');
const args = process.argv.slice(2);
const config = JSON.parse(readFileSync(args[args.indexOf('--config') + 1], 'utf8'));
if (config.bundle.macOS.files || config.bundle.macOS.entitlements !== 'Entitlements.appstore.plist') process.exit(1);
const output = join('src-tauri/target/universal-apple-darwin/release/bundle/macos', 'Better Resource Monitor.app/Contents/MacOS');
mkdirSync(output, { recursive: true });
writeFileSync(join(output, 'better-resource-monitor'), process.env.BRM_BUILD_COMMIT);
`, { mode: 0o755 });
  const { stdout } = await exec("bash", [buildScript, source, "123", "0123456789abcdef0123456789abcdef01234567"], {
    cwd: fixture,
    env: { ...process.env, PATH: `${bin}:${process.env.PATH}` },
  });
  assert.match(stdout, /Build commit embedded successfully/);
  assert.match(stdout, /No private GPU sampling executable references found/);
});
