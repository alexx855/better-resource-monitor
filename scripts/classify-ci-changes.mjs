import { pathToFileURL } from "node:url";
import { findAppImpactingFile } from "./classify-testflight-changes.mjs";

/** App code and the tooling that builds it need a real macOS runner. */
export function requiresMacOSChecks(files) {
  return Boolean(findAppImpactingFile(files)) || files.some((file) =>
    file.startsWith(".cargo/") ||
    file.startsWith("rust-toolchain") ||
    file.startsWith(".github/actions/") ||
    /^\.github\/workflows\/(ci|testflight|release|direct-download)\.yml$/.test(file) ||
    /^scripts\/(build-appstore-bundle|setup-appstore-signing|setup-developer-id-signing|upload-appstore-package)\.sh$/.test(file) ||
    ["package.json", "pnpm-lock.yaml", "pnpm-workspace.yaml", ".npmrc", "scripts/classify-ci-changes.mjs", "scripts/classify-testflight-changes.mjs"].includes(file)
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const files = Buffer.concat(chunks).toString("utf8").split("\n").filter(Boolean);
  process.stdout.write(`macos=${requiresMacOSChecks(files)}\n`);
}
