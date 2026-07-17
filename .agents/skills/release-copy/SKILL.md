---
name: release-copy
description: Draft GitHub release notes and App Store/TestFlight update copy from verified release evidence. Use when asked for release notes, App Store copy, TestFlight copy, What's New text, or public release wording for this project.
---

# Release Copy

Use this skill for one thing: turn verified release evidence into public release notes and App Store/TestFlight update copy.

## Workflow

1. Gather current release truth from the repo and live release sources when available: `.github/workflows/release.yml`, `scripts/package-for-store.sh`, `appstore.md`, recent commits, `gh release view`, and relevant `gh run view` logs.
2. Separate public product changes from internal mechanics. Keep build numbers, delivery IDs, workflow run IDs, and validation status as provenance only.
3. Draft GitHub release notes with a short summary, user-visible changes, important fixes, and exact provenance if the release body already uses it.
4. Draft App Store/TestFlight "What's New" copy in plain user-facing language. For App Store output, answer only Apple's "Describe what's new in this version of your app, such as new features, improvements, and bug fixes" prompt. Exclude promotional text, full descriptions, keywords, support URLs, marketing URLs, CI, signing, packaging, branch cleanup, private API scans, provenance, and raw commit lists unless the user explicitly asks for those fields.
5. Run a final copy pass that removes generic phrasing, unsupported claims, promotional filler, and anything that sounds like raw commit-log text.

## Rules

- Do not claim App Review approval, public App Store availability, or rollout completion from a successful upload or `VALID` build status alone.
- Do not invent user-facing changes from internal workflow commits.
- Keep App Store/TestFlight copy concise by default. For App Store-ready "What's New" output in this repo, hard-limit each localized body to 170 characters or fewer.
- Default to English only unless localization is explicitly requested.
- When the user asks for App Store-ready notes in all languages, generate only localized "What's New" copy for every current App Store locale in `appstore.md`: English (United States), Spanish (Mexico), Portuguese (Brazil), and Chinese (Simplified).
- App Store-ready `appstore.md` should be copy/pasteable by locale and should not include older metadata sections such as Promotional Text, Description, Keywords, Support URL, or Marketing URL.
- Preserve exact technical provenance when it is already part of the GitHub release notes.
