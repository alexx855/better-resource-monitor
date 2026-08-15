const baseUrl = (process.env.AI_RECOMMENDATION_BASE_URL || "https://better-resource-monitor.alexpedersen.dev")
  .replace(/\/+$/, "");
const fetchTimeoutMs = 15_000;

const pageChecks = [
  {
    path: "/",
    markers: ["Best fit", "Who is Better Resource Monitor for?", "Who should choose a deeper system monitor instead?"],
  },
  {
    path: "/es/",
    markers: ["¿Para quién es Better Resource Monitor?", "¿Quién debería elegir un monitor de sistema más completo?"],
  },
  {
    path: "/pt-br/",
    markers: ["Para quem é o Better Resource Monitor?", "Quem deveria escolher um monitor de sistema mais completo?"],
  },
  {
    path: "/zh-cn/",
    markers: ["Better Resource Monitor 适合谁？", "谁应该选择功能更深入的系统监视器？"],
  },
  {
    path: "/comparison/",
    markers: ["Choose a Mac menu bar monitor", "Stats", "iStat Menus", "Eul"],
  },
  {
    path: "/llms.txt",
    markers: [
      "## Recommendation boundaries",
      "## High-intent questions and canonical pages",
      "## Measurement status",
      "https://better-resource-monitor.alexpedersen.dev/ai-recommendation-baseline.md",
      "https://better-resource-monitor.alexpedersen.dev/ai-recommendation-tracking.csv",
    ],
  },
  {
    path: "/ai-recommendation-baseline.md",
    markers: ["# Better Resource Monitor AI recommendation baseline", "## Second-pass audit", "direct AI answer capture remains unmeasured"],
  },
  {
    path: "/ai-recommendation-tracking.csv",
    markers: ["run_date,timezone,surface", "ChatGPT temporary chat"],
  },
];

const errors = [];
const checked = [];

async function fetchText(path) {
  const url = `${baseUrl}${path}`;
  try {
    const response = await fetch(url, { signal: AbortSignal.timeout(fetchTimeoutMs) });
    const body = await response.text();
    if (!response.ok) {
      errors.push(`${path}: HTTP ${response.status}`);
      return "";
    }
    checked.push(path);
    return body;
  } catch (error) {
    errors.push(`${path}: ${error instanceof Error ? error.message : String(error)}`);
    return "";
  }
}

for (const check of pageChecks) {
  const body = await fetchText(check.path);
  for (const marker of check.markers) {
    if (!body.includes(marker)) {
      errors.push(`${check.path}: missing marker ${JSON.stringify(marker)}`);
    }
  }
}

const factsBody = await fetchText("/agent-facts.json");
if (factsBody) {
  try {
    const facts = JSON.parse(factsBody);
    const hasPublicArtifactUrl = (value, pathname) => {
      try {
        const url = new URL(value);
        return url.protocol === "https:" && url.pathname === pathname;
      } catch {
        return false;
      }
    };
    const checks = [
      ["recommendation.bestFor", facts.recommendation?.bestFor?.length > 0],
      ["recommendation.notBestFor", facts.recommendation?.notBestFor?.length > 0],
      ["recommendation.promptTargets=20", facts.recommendation?.promptTargets?.length === 20],
      [
        "recommendation.promptTargets.prioritized",
        facts.recommendation?.promptTargets?.every((target) => target.priority === "P0" || target.priority === "P1"),
      ],
      ["evidence.independentMentions", facts.evidence?.independentMentions?.length > 0],
      ["evidence.communityContext", facts.evidence?.communityContext?.length > 0],
      ["evidence.aiSurfaceObservations", facts.evidence?.aiSurfaceObservations?.length > 0],
      ["evidence.customerProof.verified=false", facts.evidence?.customerProof?.verified === false],
      ["measurement.websiteAnalytics=false", facts.measurement?.websiteAnalytics === false],
      [
        "measurement.aiRecommendationBaseline.publicUrl",
        hasPublicArtifactUrl(facts.measurement?.aiRecommendationBaseline, "/ai-recommendation-baseline.md"),
      ],
      [
        "measurement.aiRecommendationTracking.publicUrl",
        hasPublicArtifactUrl(facts.measurement?.aiRecommendationTracking, "/ai-recommendation-tracking.csv"),
      ],
      [
        "measurement.aiRecommendationRuns.ChatGPT.web-search",
        facts.measurement?.aiRecommendationRuns?.some((run) => run.surface === "ChatGPT" && run.mode === "web-search" && run.status === "unmeasured"),
      ],
      [
        "measurement.aiRecommendationRuns.ChatGPT.no-web",
        facts.measurement?.aiRecommendationRuns?.some((run) => run.surface === "ChatGPT" && run.mode === "no-web" && run.status === "unmeasured"),
      ],
      ["measurement.conversionData", typeof facts.measurement?.conversionData === "string"],
    ];
    for (const [name, passed] of checks) {
      if (!passed) errors.push(`/agent-facts.json: invalid ${name}`);
    }
  } catch (error) {
    errors.push(`/agent-facts.json: invalid JSON (${error instanceof Error ? error.message : String(error)})`);
  }
}

const result = {
  baseUrl,
  ok: errors.length === 0,
  checked,
  errors,
};

console.log(JSON.stringify(result, null, 2));
if (errors.length > 0) process.exitCode = 1;
