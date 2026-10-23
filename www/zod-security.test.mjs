import assert from "node:assert/strict";
import { createRequire } from "node:module";
import test from "node:test";

const require = createRequire(import.meta.url);

// Exercise each consumer's installed copy, including pnpm's isolated dependencies.
for (const dependency of ["astro", "@astrojs/sitemap"]) {
  const requireDependency = createRequire(require.resolve(dependency));
  const { z } = requireDependency("zod");

  for (const [kind, schemas, oversized, valid, malformed] of [
    [
      "array",
      [z.array(z.string()).max(2), z.array(z.string()).length(2)],
      [0, 1, 2],
      ["a", "b"],
      ["a", 0],
    ],
    [
      "set",
      [z.set(z.string()).max(2), z.set(z.string()).size(2)],
      new Set([0, 1, 2]),
      new Set(["a", "b"]),
      new Set(["a", 0]),
    ],
    [
      "map",
      [z.map(z.string(), z.string()).max(2), z.map(z.string(), z.string()).size(2)],
      new Map([[0, 0], [1, 1], [2, 2]]),
      new Map([["a", "a"], ["b", "b"]]),
      new Map([["a", "a"], ["b", 0]]),
    ],
  ]) {
    test(`${dependency} rejects oversized ${kind} inputs before accumulating element errors`, () => {
      for (const schema of schemas) {
        const result = schema.safeParse(oversized);
        assert.equal(result.success, false);
        assert.equal(result.error.issues.length, 1);
        assert.equal(result.error.issues[0].code, "too_big");

        assert.deepEqual(schema.parse(valid), valid);
        const invalid = schema.safeParse(malformed);
        assert.equal(invalid.success, false);
        assert.equal(invalid.error.issues[0].code, "invalid_type");
      }
    });
  }
}
