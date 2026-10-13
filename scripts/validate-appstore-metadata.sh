#!/bin/bash
# Shared by trusted resolution, build and signing. Validate before shortening.
validate_appstore_metadata() {
  if ! [[ "${1:-}" =~ ^[0-9a-f]{40}$ ]]; then
    echo 'Error: build commit must be a full 40-character lowercase git SHA' >&2
    return 1
  fi
  if ! [[ "${2:-}" =~ ^[0-9]+$ ]]; then
    echo 'Error: build number must be numeric' >&2
    return 1
  fi
}
