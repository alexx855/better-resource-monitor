import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import http from "node:http";

import { verifyPagesDeployment } from "./verify-pages-deployment.mjs";

const redirects = new Map([
  ["/en/", "/"],
  ["/en/comparison/", "/comparison/"],
  ["/en/privacy-policy/", "/privacy-policy/"],
  ["/en/terms/", "/terms/"],
  ["/vs-istat-menus/", "/comparison/vs-istat-menus/"],
  ["/faq/", "/"],
  ["/sitemap.xml", "/sitemap-index.xml"],
]);

const pages = new Map([
  ["/", "Better Resource Monitor | Lightweight macOS Menu Bar Monitor"],
  ["/comparison/", "Mac Menu Bar Monitor Comparisons | Better Resource Monitor"],
  ["/privacy-policy/", "Privacy Policy"],
  ["/terms/", "Terms"],
  ["/comparison/vs-istat-menus/", "iStat Menus comparison"],
  ["/sitemap-index.xml", "sitemap"],
]);

let baseUrl;
let server;
let brokenPath;

before(async () => {
  server = http.createServer((request, response) => {
    if (request.url === brokenPath) {
      response.writeHead(200, { "content-type": "text/html" });
      response.end("<title>Wrong response</title>");
      return;
    }

    const destination = redirects.get(request.url);
    if (destination) {
      response.writeHead(301, { location: destination });
      response.end();
      return;
    }

    const title = pages.get(request.url);
    if (title) {
      response.writeHead(200, { "content-type": "text/html" });
      response.end(`<html><head><title>${title}</title></head></html>`);
      return;
    }

    response.writeHead(404);
    response.end("not found");
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  baseUrl = `http://127.0.0.1:${address.port}`;
});

after(async () => {
  await new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
  });
});

test("accepts the deployed redirect, destination, status, and title contract", async () => {
  const result = await verifyPagesDeployment(baseUrl);

  assert.equal(result.ok, true);
  assert.equal(result.redirects.length, 7);
  assert.equal(result.titles.length, 2);
  assert.deepEqual(result.errors, []);
});

test("rejects a route that no longer redirects", async () => {
  brokenPath = "/faq/";

  try {
    const result = await verifyPagesDeployment(baseUrl);
    assert.equal(result.ok, false);
    assert.match(result.errors.join("\n"), /\/faq\/: expected HTTP 301, received 200/);
  } finally {
    brokenPath = undefined;
  }
});
