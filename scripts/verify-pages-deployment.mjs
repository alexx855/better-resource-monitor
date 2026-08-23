import { pathToFileURL } from "node:url";

const redirectChecks = [
  ["/en/", "/"],
  ["/en/comparison/", "/comparison/"],
  ["/en/privacy-policy/", "/privacy-policy/"],
  ["/en/terms/", "/terms/"],
  ["/vs-istat-menus/", "/comparison/vs-istat-menus/"],
  ["/faq/", "/"],
  ["/sitemap.xml", "/sitemap-index.xml"],
];

const titleChecks = [
  ["/", "Better Resource Monitor | Lightweight macOS Menu Bar Monitor"],
  ["/comparison/", "Mac Menu Bar Monitor Comparisons | Better Resource Monitor"],
];

const fetchTimeoutMs = 15_000;

function normalizeBaseUrl(value) {
  const url = new URL(value);
  url.pathname = url.pathname.replace(/\/+$/, "");
  url.search = "";
  url.hash = "";
  return url.href.replace(/\/+$/, "");
}

function extractTitle(html) {
  return html.match(/<title>([^<]*)<\/title>/i)?.[1].trim();
}

export async function verifyPagesDeployment(baseUrlValue, { fetchImpl = fetch } = {}) {
  const baseUrl = normalizeBaseUrl(baseUrlValue);
  const errors = [];
  const redirects = [];
  const titles = [];

  for (const [path, destination] of redirectChecks) {
    const sourceUrl = new URL(path, `${baseUrl}/`);
    const expectedUrl = new URL(destination, `${baseUrl}/`);

    try {
      const redirectResponse = await fetchImpl(sourceUrl, {
        redirect: "manual",
        signal: AbortSignal.timeout(fetchTimeoutMs),
      });
      const location = redirectResponse.headers.get("location");

      if (redirectResponse.status !== 301) {
        errors.push(`${path}: expected HTTP 301, received ${redirectResponse.status}`);
      }
      if (!location) {
        errors.push(`${path}: missing Location header`);
      } else if (new URL(location, sourceUrl).href !== expectedUrl.href) {
        errors.push(`${path}: expected Location ${expectedUrl.href}, received ${new URL(location, sourceUrl).href}`);
      }

      const finalResponse = await fetchImpl(sourceUrl, {
        redirect: "follow",
        signal: AbortSignal.timeout(fetchTimeoutMs),
      });
      if (finalResponse.status !== 200) {
        errors.push(`${path}: expected final HTTP 200, received ${finalResponse.status}`);
      }
      if (finalResponse.url !== expectedUrl.href) {
        errors.push(`${path}: expected final URL ${expectedUrl.href}, received ${finalResponse.url}`);
      }

      redirects.push({
        path,
        status: redirectResponse.status,
        location,
        finalStatus: finalResponse.status,
        finalUrl: finalResponse.url,
      });
    } catch (error) {
      errors.push(`${path}: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  for (const [path, expectedTitle] of titleChecks) {
    const url = new URL(path, `${baseUrl}/`);

    try {
      const response = await fetchImpl(url, {
        redirect: "follow",
        signal: AbortSignal.timeout(fetchTimeoutMs),
      });
      const title = extractTitle(await response.text());

      if (response.status !== 200) {
        errors.push(`${path}: expected HTTP 200 for title check, received ${response.status}`);
      }
      if (title !== expectedTitle) {
        errors.push(`${path}: expected title ${JSON.stringify(expectedTitle)}, received ${JSON.stringify(title)}`);
      }

      titles.push({ path, status: response.status, title });
    } catch (error) {
      errors.push(`${path}: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  return {
    baseUrl,
    ok: errors.length === 0,
    redirects,
    titles,
    errors,
  };
}

async function main() {
  const baseUrl = process.env.PAGES_DEPLOYMENT_BASE_URL;
  if (!baseUrl) {
    console.error("PAGES_DEPLOYMENT_BASE_URL is required");
    process.exitCode = 1;
    return;
  }

  const result = await verifyPagesDeployment(baseUrl);
  console.log(JSON.stringify(result, null, 2));
  if (!result.ok) process.exitCode = 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main();
}
