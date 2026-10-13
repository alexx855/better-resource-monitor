#!/bin/bash
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/validate-appstore-metadata.sh"

if [ "$GITHUB_REF" != refs/heads/main ]; then
  echo 'Error: dispatch TestFlight from main' >&2
  exit 1
fi
source_ref="${INPUT_SOURCE_REF:-main}"
pr_number="${PR_NUMBER:-}"
if [ -n "$pr_number" ] && ! [[ "$pr_number" =~ ^[1-9][0-9]*$ ]]; then
  echo 'Error: pr_number must be a positive integer' >&2
  exit 1
fi
if [[ "$source_ref" =~ ^refs/pull/([1-9][0-9]*)/head$ ]]; then
  source_pr="${BASH_REMATCH[1]}"
  if [ -n "$pr_number" ] && [ "$pr_number" != "$source_pr" ]; then
    echo 'Error: pr_number must match source_ref' >&2
    exit 1
  fi
  # Resolve from GitHub on a trusted runner, never from the build runner.
  pr="$(gh api "repos/${GITHUB_REPOSITORY}/pulls/${source_pr}")"
  sha="$(jq -er --arg repo "$GITHUB_REPOSITORY" --argjson number "$source_pr" \
    'select(.number == $number and .base.repo.full_name == $repo and .state == "open") | .head.sha' <<< "$pr")"
else
  if [ -n "$pr_number" ]; then
    echo 'Error: pr_number requires its matching refs/pull/<number>/head' >&2
    exit 1
  fi
  if [ "$source_ref" != main ] && ! [[ "$source_ref" =~ ^v[0-9][0-9A-Za-z._-]*$ ]]; then
    echo 'Error: source_ref must be main, a v* tag, or refs/pull/<number>/head' >&2
    exit 1
  fi
  sha="$(gh api "repos/${GITHUB_REPOSITORY}/commits/${source_ref}" --jq .sha)"
fi
build_number="${INPUT_BUILD_NUMBER:-}"
if [ -z "$build_number" ]; then
  timestamp_floor="$(date -u +%Y%m%d%H%M%S)"
  if [ "$GITHUB_RUN_NUMBER" -lt "$timestamp_floor" ]; then
    build_number="$timestamp_floor"
  else
    build_number="$GITHUB_RUN_NUMBER"
  fi
fi
validate_appstore_metadata "$sha" "$build_number"
printf 'sha=%s\nbuild_number=%s\npr_number=%s\n' "$sha" "$build_number" "$pr_number" >> "$GITHUB_OUTPUT"
