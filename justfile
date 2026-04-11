default:
    @just --list

# Publish toolkit crates to crates.io and cut a release tag.
# Usage:
#   just publish <version> [dry_run] [no_tag] [push] [allow_dirty] [no_require_main]
publish \
  version="" \
  dry_run="false" \
  no_tag="false" \
  push="false" \
  allow_dirty="false" \
  no_require_main="false":
    #!/usr/bin/env bash
    set -euo pipefail
    args=()
    if [ -n "{{version}}" ]; then
      args+=(--version "{{version}}")
    fi
    if [ "{{dry_run}}" = "true" ]; then
      args+=(--dry-run)
    fi
    if [ "{{no_tag}}" = "true" ]; then
      args+=(--no-tag)
    fi
    if [ "{{push}}" = "true" ]; then
      args+=(--push)
    fi
    if [ "{{allow_dirty}}" = "true" ]; then
      args+=(--allow-dirty)
    fi
    if [ "{{no_require_main}}" = "true" ]; then
      args+=(--no-require-main)
    fi
    ./scripts/release-publish.sh "${args[@]}"
