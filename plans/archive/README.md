# Archived plans

These documents are retained as implementation history and evidence for work
that is no longer an active plan. The active index is one directory up at
[`../README.md`](../README.md).

## Superseded

- [001 — Align public distribution copy](001-align-public-distribution-copy.md) was superseded by Plan 006.
- [002 — Add the marketing site build to CI](002-add-website-build-to-ci.md) was superseded by Plan 009.
- [004 — Make the website CSP compatible with Astro ClientRouter](004-fix-website-csp-client-router.md) was superseded by Plan 011.

## Implemented

- [003 — Website dependency advisories](003-upgrade-www-security-advisories.md) — original remediation landed; the current audit follow-up remains active in the [main index](../README.md).
- [005 — Vendor generated-image fonts](005-vendor-generated-image-fonts.md)
- [006 — Direct-download distribution](006-direct-download-distribution-decision.md)
- [007 — Signing isolation](007-isolate-signing-from-untrusted-code.md)
- [008 — Tray-update panic hardening](008-harden-monitor-thread-against-panics.md)
- [009 — Documented CI checks](009-enforce-documented-checks-in-ci.md)
- [010 — Locale parity](010-enforce-locale-parity.md)
- [011 — Remove ClientRouter](011-remove-client-router.md)

Plan 003's original remediation also landed, but its dependency-audit gate has
since regressed as new transitive advisories appeared. The active index records
that follow-up instead of treating the old plan as permanently complete.
