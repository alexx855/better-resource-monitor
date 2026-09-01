import { pathToFileURL } from "node:url";

export function findAppImpactingFile(files) {
  return files.find(
    (file) => file.startsWith("src-tauri/") && !file.startsWith("src-tauri/gen/schemas/"),
  );
}

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
