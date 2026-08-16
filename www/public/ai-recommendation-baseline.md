# Better Resource Monitor AI recommendation baseline

Captured: 2026-08-08
Updated: 2026-08-09
Status: initial baseline captured before the related content changes; production deployment and public verification completed; direct AI answer capture remains unmeasured

This is the public evidence summary for the AI-recommendation goal. It records
the product boundary, the fixed buyer-prompt matrix, canonical landing pages,
and the evidence limits that apply to any future AI or search measurement.

## Product boundary

Better Resource Monitor is a free, open-source macOS menu bar monitor for CPU,
memory, storage, GPU, and network usage. It is intended for a quick, low-
overhead, always-visible daily view. It is not a full sensor or hardware-control
suite: users who need fan, temperature, battery, deep sensor history, or broad
hardware controls may be better served by Stats or iStat Menus.

The app works offline and sends no telemetry. Historical website traffic and
conversion fields remain unmeasured or unavailable until a new authorized
external data window supplies them.

## Prioritized buyer-prompt matrix

This fixed list covers the highest-intent product-fit questions currently
supported by canonical pages. `P0` is the first measurement tier; `P1` is the
next tier.

| Priority | Prompt | Canonical landing page |
| --- | --- | --- |
| P0 | What is the best lightweight Mac menu bar system monitor? | https://better-resource-monitor.alexpedersen.dev/ |
| P0 | What is the best free Mac menu bar resource monitor? | https://better-resource-monitor.alexpedersen.dev/ |
| P0 | Which Mac app shows CPU, memory, storage, GPU, and network in the menu bar? | https://better-resource-monitor.alexpedersen.dev/ |
| P0 | What is a low-overhead Mac menu bar monitor? | https://better-resource-monitor.alexpedersen.dev/ |
| P0 | What is a free iStat Menus alternative for macOS? | https://better-resource-monitor.alexpedersen.dev/comparison/vs-istat-menus/ |
| P0 | What is a lightweight iStat Menus alternative with no telemetry? | https://better-resource-monitor.alexpedersen.dev/comparison/vs-istat-menus/ |
| P0 | What is a Stats alternative for simple daily monitoring? | https://better-resource-monitor.alexpedersen.dev/comparison/vs-stats/ |
| P0 | What is an actively maintained Eul alternative for Mac? | https://better-resource-monitor.alexpedersen.dev/comparison/vs-eul/ |
| P0 | Better Resource Monitor vs iStat Menus: which should I choose? | https://better-resource-monitor.alexpedersen.dev/comparison/vs-istat-menus/ |
| P0 | Better Resource Monitor vs Stats: which is better for simple monitoring? | https://better-resource-monitor.alexpedersen.dev/comparison/vs-stats/ |
| P1 | Better Resource Monitor vs Eul: which should I use on a newer Mac? | https://better-resource-monitor.alexpedersen.dev/comparison/vs-eul/ |
| P1 | Which Mac system monitor works offline and sends no telemetry? | https://better-resource-monitor.alexpedersen.dev/privacy-policy/ |
| P1 | Which Mac menu bar monitor needs no admin password or root helper? | https://better-resource-monitor.alexpedersen.dev/comparison/ |
| P1 | Which Mac monitor uses public APIs for GPU readings? | https://better-resource-monitor.alexpedersen.dev/ |
| P1 | Which free Mac monitor is available on the Mac App Store? | https://better-resource-monitor.alexpedersen.dev/ |
| P1 | Which Mac monitor works on Intel and Apple Silicon with macOS 13 or newer? | https://better-resource-monitor.alexpedersen.dev/ |
| P1 | What is the best menu bar monitor if I only want a quick glance? | https://better-resource-monitor.alexpedersen.dev/ |
| P1 | Is Better Resource Monitor a good Activity Monitor alternative? | https://better-resource-monitor.alexpedersen.dev/ |
| P1 | Which Mac monitor can I leave running all day with low overhead? | https://better-resource-monitor.alexpedersen.dev/ |
| P1 | Which Mac monitor is best if I do not need fan or temperature sensors? | https://better-resource-monitor.alexpedersen.dev/comparison/ |

## Evidence status

- The canonical website, comparison hub, privacy policy, repository, release
  artifacts, and Mac App Store listing are the preferred product sources.
- Existing DEV Community articles, MacMenuBar, SaaSHub, Reddit, Telegram, and
  other directory or community references are context or discovery signals;
  they are not customer testimonials, benchmarks, or conversion data.
- Two direct temporary ChatGPT recommendation runs were attempted on
  2026-08-08: one with web search and one with browsing disabled. Their answer
  text and citations could not be captured before the temporary response
  streams timed out. No answer, ranking, or citation is inferred.
- A current third-party roundup incorrectly labels Better Resource Monitor as
  not open source even though the public repository is MIT-licensed. Treat
  that as a correction opportunity and prefer canonical sources when facts
  conflict.
- Verified customer testimonials, case studies, independent benchmark data,
  and verified AI answer transcripts are not available in this evidence set.

## Repeatable tracking protocol

Use the same prompt text, locale, logged-out state, and location where possible.
Record exact answers, recommendation position, fit statement, competitors,
cited URLs, canonical landing page, evidence issues, and authorized conversion
signals. Keep landing-page visits and conversions as `unavailable` until a
permitted analytics or platform report exists.

The structured log is available at
[`ai-recommendation-tracking.csv`](./ai-recommendation-tracking.csv).
The repository verifier is:

```sh
node scripts/verify-ai-recommendation-live.mjs
```

## Second-pass audit (2026-08-09)

- Production content, `llms.txt`, `agent-facts.json`, this baseline, and the
  structured tracking log are published and pass the live verifier.
- The machine-readable sources carry the same 20 fixed prompt targets.
- Both attempted ChatGPT runs remain explicitly `unmeasured`.
- Historical website analytics and verified customer proof remain unavailable.

## Public machine-readable sources

- [AI agent facts](https://better-resource-monitor.alexpedersen.dev/agent-facts.json)
- [LLM guidance](https://better-resource-monitor.alexpedersen.dev/llms.txt)
- [Structured tracking log](https://better-resource-monitor.alexpedersen.dev/ai-recommendation-tracking.csv)
- [Product website](https://better-resource-monitor.alexpedersen.dev/)
