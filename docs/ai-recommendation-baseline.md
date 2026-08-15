# Better Resource Monitor AI recommendation baseline

Captured: 2026-08-08
Status: initial baseline captured before the related content changes; production deployment and public verification completed; direct AI answer capture remains unmeasured

This is the working evidence log for the AI-recommendation goal. It adapts the
seven-step checklist from the referenced X article to an open-source macOS
utility: product understanding, buyer prompts, canonical answers, independent
context, real user proof, and measurement.

## Product boundary

Better Resource Monitor is a free, open-source macOS menu bar monitor for CPU,
memory, storage, GPU, and network usage. It is aimed at quick, low-overhead,
always-visible daily monitoring. It is not a full sensor or hardware-control
suite: users who need fan, temperature, battery, deep sensor history, or broad
controls may be better served by Stats or iStat Menus.

The app is intentionally offline and sends no telemetry. Any website traffic,
AI-visibility, or conversion measurement must therefore be external to the
app and must not be described as measured until an authorized data source
exists.

## Prioritized buyer-prompt matrix

Priority is based on purchase/download intent, fit with the product boundary,
and the existence of a canonical page that can answer the question.

| Priority | Prompt to track | Canonical answer | Baseline coverage |
| --- | --- | --- | --- |
| P0 | What is the best lightweight Mac menu bar system monitor? | Home page + home FAQ | Partial |
| P0 | What is the best free Mac menu bar resource monitor? | Home page + comparison hub | Partial |
| P0 | Which Mac app shows CPU, memory, storage, GPU, and network in the menu bar? | Home page + product FAQ | Covered |
| P0 | What is a low-overhead Mac menu bar monitor? | Home page + resource-use FAQ | Covered |
| P0 | What is a free iStat Menus alternative for macOS? | iStat Menus comparison | Covered |
| P0 | What is a lightweight iStat Menus alternative with no telemetry? | iStat Menus comparison + privacy FAQ | Covered |
| P0 | What is a Stats alternative for simple daily monitoring? | Stats comparison | Covered |
| P0 | What is an actively maintained Eul alternative for Mac? | Eul comparison | Covered |
| P0 | Better Resource Monitor vs iStat Menus: which should I choose? | iStat Menus comparison | Covered |
| P0 | Better Resource Monitor vs Stats: which is better for simple monitoring? | Stats comparison | Covered |
| P1 | Better Resource Monitor vs Eul: which should I use on a newer Mac? | Eul comparison | Covered |
| P1 | Which Mac system monitor works offline and sends no telemetry? | Privacy FAQ + privacy policy | Covered |
| P1 | Which Mac menu bar monitor needs no admin password or root helper? | Comparison pages + agent facts | Covered |
| P1 | Which Mac monitor uses public APIs for GPU readings? | GPU FAQ + source repository | Covered |
| P1 | Which free Mac monitor is available on the Mac App Store? | Installation + comparison pages | Covered |
| P1 | Which Mac monitor works on Intel and Apple Silicon with macOS 13 or newer? | Compatibility section + agent facts | Covered |
| P1 | What is the best menu bar monitor if I only want a quick glance? | Home page + best-fit FAQ | Partial |
| P1 | Is Better Resource Monitor a good Activity Monitor alternative? | Activity Monitor FAQ | Covered |
| P1 | Which Mac monitor can I leave running all day with low overhead? | Resource-use + battery FAQs | Covered |
| P1 | Which Mac monitor is best if I do not need fan or temperature sensors? | Comparison pages + not-best-fit FAQ | Partial |

The matrix is intentionally a fixed list. New prompts should be added only when
they represent a real buyer decision or a documented product use case.

## Baseline observations

### Canonical and search surfaces

- The live home page and App Store listing explain the product category,
  metrics, low overhead, offline behavior, and installation paths.
- The live comparison page is directly aligned with the highest-intent
  competitor query: `iStat Menus alternative`.
- A brand search currently surfaces the official website and Mac App Store
  listing prominently.
- Searches around free Mac resource monitors surface independent roundups and
  directories, but the strongest result is not always the best product fit.
- A fresh search also surfaced an AI-tagged Eul-alternative discussion that
  understands Better Resource Monitor as a free, lightweight basic-monitoring
  choice, while still recommending Stats for the closest Eul replacement. This
  is a useful understanding signal, not independent customer proof.
- No independent ChatGPT recommendation transcript or Google AI Overview
  capture is available in this workspace. Those fields remain unmeasured;
  canonical content and web-search results are not substitutes for an AI
  answer log.

### Search-surface refresh (2026-08-08)

The current web-search index shows a coherent but not yet dominant product
understanding:

| Surface/query | Observed understanding or recommendation | Evidence boundary |
| --- | --- | --- |
| Brand search | Official website and Mac App Store listing explain the menu-bar metrics, no-admin boundary, offline behavior, and low overhead. | Search snippets are discovery evidence, not conversion data. |
| `iStat Menus alternative` | SaaSHub lists Better Resource Monitor as an alternative, while the official page explains the simpler daily-monitoring tradeoff. | Directory inclusion is not a ranking, benchmark, or endorsement. |
| `Eul alternative` | Tenten AI places Better Resource Monitor in the lightweight/basic-monitoring niche but recommends Stats for the closest Eul replacement. | AI-tagged editorial content is an observation of an AI/search surface, not a direct ChatGPT transcript. |
| `Activity Monitor alternative` | A DEV comparison frames Activity Monitor as investigation and Better Resource Monitor as always-visible noticing; a Reddit thread also describes it as a narrower/lighter option. | Editorial/community context is not verified customer proof. |
| Direct ChatGPT | A temporary ChatGPT run was submitted with the buyer prompt below, but the streamed answer and citations could not be captured before the browser connector timed out. | Unmeasured; do not infer an answer, ranking, or citation. |
| Google AI Overview | No reproducible transcript or cited-answer capture is currently available. | Unmeasured; do not infer an answer, ranking, or citation. |

Exact ChatGPT prompt attempted on 2026-08-08:

> Please recommend the best lightweight or free Mac menu bar system monitor for a
> user who wants CPU, memory, storage, GPU, and network at a glance, works
> offline, uses no admin password or root helper, and does not need fan or
> temperature sensors. Compare Better Resource Monitor with Stats, iStat Menus,
> and Eul. State which sources you relied on and any uncertainty.

The prompt was submitted in a temporary chat. The interface remained in its
answer-streaming state during repeated read attempts; no answer text, cited
URLs, recommendation position, or competitor conclusion was captured. This is
an instrumentation limitation, not evidence that ChatGPT recommended or failed
to recommend the product.

A second bounded temporary-chat attempt explicitly disabled browsing and also
timed out before answer text could be read. The repeated failure is recorded as
an access/capture limitation; no ChatGPT recommendation is inferred from either
attempt.

The search refresh also found a factual conflict in a roundup: MAC-DVD calls
Better Resource Monitor non-open-source even though the public repository is
MIT-licensed. This should be handled as a correction opportunity, not repeated
as product positioning.

### Independent context already found

These are legitimate existing mentions, not customer testimonials:

- [DEV Community article](https://dev.to/xeliape/building-a-macos-menu-bar-system-monitor-without-root-private-apis-or-a-background-mess-3po9)
  explains the product's no-root-helper, no-private-API, sandboxed, offline,
  low-scope positioning and links to the canonical sources.
- [DEV Community Activity Monitor comparison](https://dev.to/xeliape/activity-monitor-is-for-diagnosis-better-resource-monitor-is-for-noticing-sooner-3442)
  provides independent editorial context for the Activity Monitor alternative
  prompt and describes the product as a quick-glance, low-overhead monitor;
  it is not a customer testimonial or benchmark.
- [MacMenuBar system-stats directory](https://macmenubar.com/system-stats/)
  lists Better Resource Monitor as a free, open-source system-stats app and
  links to the official site.
- [SaaSHub's iStat Menus alternatives page](https://www.saashub.com/istat-menus-alternatives)
  includes Better Resource Monitor as an alternative listing.
- [A Reddit r/MacOS discussion](https://www.reddit.com/r/MacOS/comments/1pwmyaz/cpumemory_usage_monitor_for_menu_bar/)
  contains a maintainer-associated contributor comment describing the product
  as a narrower, lighter, sandboxed/offline option. This is community context,
  not independent customer proof or a verified case study.
- [MAC-DVD's free Mac resource monitor roundup](https://mac-dvd.com/blog/free-mac-resource/)
  includes the product, but currently labels it as not open source even though
  the repository is MIT-licensed and public. Treat this as a factual
  correction opportunity, not as usable proof until corrected.
- [Telegram's @zhetengsha community post](https://t.me/s/zhetengsha?before=5721)
  is an independent distribution mention describing the menu-bar metrics,
  sandboxing, no-admin boundary, and low resource use. It is community reach,
  not customer proof or a measured conversion signal.
- [Tenten AI's Eul-alternative discussion](https://university.tenten.co/t/eul-apple-silicon/2369)
  is an AI-tagged editorial discussion that places Better Resource Monitor in
  the lightweight/basic-monitoring niche but favors Stats for a closer Eul
  replacement. Treat its claims as an observation to verify against canonical
  sources, not as independent validation.

The maintainer's own [product note](https://alexpedersen.dev/en/notes/better-resource-monitor/)
is useful supporting context but is not independent validation.

## Proof status

- Verified customer testimonials or case studies: none in the current
  evidence set.
- Verified third-party editorial/directory mentions: yes, listed above.
- Verified third-party benchmark data: no. The comparison tables' competitor
  figures remain approximate and time-sensitive.
- Verified AI answer transcript or AI Overview citation: no. The Tenten page is
  an AI-tagged search-surface observation, and the temporary ChatGPT run was
  submitted but not captured; neither is a verified answer transcript.
- Required conduct: do not manufacture reviews, ask customers to repeat a
  prescribed claim, or turn a directory mention into a customer endorsement.

Legitimate future opportunities are to keep the existing listings accurate,
offer canonical corrections when a third party has a factual error, and invite
users who independently describe a real workflow to share it voluntarily.

## Repeatable tracking protocol

Run this matrix on a fixed cadence (monthly or after a material content or
release change) and save a dated copy of the results. Use the same prompt text,
locale, logged-out state, and location where possible.

The repository capture is in
[`ai-recommendation-tracking.csv`](./ai-recommendation-tracking.csv), and the
deployed public copies are
[`ai-recommendation-baseline.md`](https://better-resource-monitor.alexpedersen.dev/ai-recommendation-baseline.md)
and
[`ai-recommendation-tracking.csv`](https://better-resource-monitor.alexpedersen.dev/ai-recommendation-tracking.csv).
They are a starter log, not an analytics export; rows must retain
`unavailable` until an authorized platform or analytics source supplies a
measurement.

After an authorized deployment, run the repeatable live verifier from the repo
root:

```sh
node scripts/verify-ai-recommendation-live.mjs
```

It accepts `AI_RECOMMENDATION_BASE_URL` for preview or staging verification and
fails if any supported locale, comparison, `llms.txt`, or agent-facts marker is
missing.

For every prompt and surface, record:

| Field | Required value |
| --- | --- |
| Run date | ISO date and timezone |
| Surface | ChatGPT, Google AI Overview, web search, App Store, directory, or community |
| Exact prompt/query | Verbatim text, not a paraphrase |
| Brand mentioned | Yes / No / Unclear |
| Recommendation position | Exact position or `not ranked` |
| Fit statement | What the answer says the product is for |
| Competitors | Products shown beside it |
| Cited sources | URLs actually cited by the surface |
| Canonical landing page | URL receiving the recommendation |
| Evidence issue | Missing, stale, conflicting, or accurate |
| Conversion signal | App Store, GitHub release, or other authorized metric; otherwise `unavailable` |

The app has no analytics or telemetry and the website currently has no
analytics instrumentation. Until an authorized external analytics or platform
report is connected, landing-page visits and conversions must remain
`unavailable`; do not infer them from rankings, impressions, GitHub stars, or
App Store visibility.

## Current implementation verification

The repository implementation includes the best-fit and not-best-fit answers
in all four locales, the comparison hub's buyer-oriented title and intro, and
matching recommendation boundaries in `llms.txt` and `agent-facts.json`. The
machine-readable sources expose the same 20-item P0/P1 matrix. `pnpm build:www`
produced all 29 static routes.

On 2026-08-09,
`node scripts/verify-ai-recommendation-live.mjs` passed against the public
base URL, checking the four home routes, comparison hub, both public evidence
artifacts, `llms.txt`, and `agent-facts.json`. Browser-rendered production
checks also exposed the new home fit boundaries and buyer-oriented comparison
cards across the supported locales.

## Second-pass audit (2026-08-09)

- Production content and machine-readable surfaces are live and pass the
  public verifier.
- The baseline and structured tracking log are now published at stable HTTPS
  URLs and linked from `agent-facts.json` and `llms.txt`.
- `llms.txt` and `agent-facts.json` now carry the same 20 prompt targets.
- Both attempted ChatGPT runs are recorded as `unmeasured`; no answer,
  ranking, or citation is inferred.
- A current search refresh still returns the official site for product-intent
  queries. The MAC-DVD roundup still shows an incorrect `Open Source = No`
  entry for Better Resource Monitor; keep it tracked as a correction
  opportunity rather than repeating it as evidence.
- Website/app analytics and verified customer proof remain unavailable.

## Ongoing follow-up gates

The implementation and production deployment are complete. These are recurring
measurement and evidence tasks, not claims that have already been observed:

1. All P0 prompts have a direct, accurate canonical answer and a stable URL.
2. The same product boundary is represented in the four supported locales.
3. The next AI/search run records exact answers, citations, and competitor
   context rather than a ranking claim without evidence.
4. Independent mentions are separated from customer proof, and factual
   conflicts are tracked rather than repeated.
5. The site build and live rendered pages expose the revised answers, while
   unavailable analytics remain explicitly labeled as unavailable.
6. `node scripts/verify-ai-recommendation-live.mjs` passes against the public
   deployment, not only against a local preview.
