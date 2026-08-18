import assert from "node:assert/strict";
import { createRequire } from "node:module";
import test from "node:test";

const require = createRequire(import.meta.url);
// fast-uri is transitive through the website's Astro check dependency.
const fastUri = require(
  require.resolve("fast-uri", { paths: [require.resolve("@astrojs/check")] }),
);

test("rejects backslash authority introducers before URL policy can diverge", () => {
  const baseUrl = "https://allowed.com/";
  const relativeUrl = String.fromCharCode(92, 92) + "evil.com/path";

  assert.equal(new URL(relativeUrl, baseUrl).hostname, "evil.com");
  assert.throws(() => fastUri.resolve(baseUrl, relativeUrl));
});
