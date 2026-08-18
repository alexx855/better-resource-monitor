import assert from "node:assert/strict";
import test from "node:test";

import { escapeRegExp } from "./badge-readme.mjs";

test("escapes backslashes in a badge base URL before regex construction", () => {
  const baseUrl = "https://example.com\\d/";
  const pattern = new RegExp("^" + escapeRegExp(baseUrl) + "appstore\\.webp$");

  assert.equal(pattern.test(baseUrl + "appstore.webp"), true);
  assert.equal(pattern.test("https://example.com5/appstore.webp"), false);
});
