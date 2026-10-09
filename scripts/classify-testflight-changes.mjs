import { pathToFileURL } from "node:url";

/** Return the first non-generated Tauri path that requires app validation. */
export function findAppImpactingFile(files) {
  return files.find(
    (file) => file.startsWith("src-tauri/") && !file.startsWith("src-tauri/gen/schemas/"),
  );
}

/** Classify newline-delimited current and previous PR file paths from stdin. */
async function main() {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);

  const files = Buffer.concat(chunks)
    .toString("utf8")
    .split("\n")
    .map((file) => file.trim())
    .filter(Boolean);
  const appFile = findAppImpactingFile(files);
  if (appFile) process.stdout.write(appFile);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main();
}
