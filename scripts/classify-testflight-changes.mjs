import { pathToFileURL } from "node:url";

/** Return the first non-generated Tauri path that requires app validation. */
export function findAppImpactingFile(files) {
  return files.find(
    (file) => file.startsWith("src-tauri/") && !file.startsWith("src-tauri/gen/schemas/"),
  );
}

/** Classify file paths from stdin; --null accepts Git's lossless path format. */
async function main() {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);

  const nulDelimited = process.argv.includes("--null");
  const files = Buffer.concat(chunks)
    .toString("utf8")
    .split(nulDelimited ? "\0" : "\n")
    .map((file) => nulDelimited ? file : file.trim())
    .filter(Boolean);
  const appFile = findAppImpactingFile(files);
  if (appFile) process.stdout.write(appFile);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main();
}
