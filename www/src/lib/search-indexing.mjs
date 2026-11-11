const excludedPaths = new Set([
  "/es/privacy-policy/",
  "/pt-br/comparison/vs-eul/",
]);

/** @param {string} pathname */
export function isIndexablePath(pathname) {
  return !excludedPaths.has(pathname.endsWith("/") ? pathname : `${pathname}/`);
}
