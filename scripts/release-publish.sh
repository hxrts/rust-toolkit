#!/usr/bin/env bash
# Publish toolkit crates to crates.io in dependency order.
# Supports dry-run, version validation, git tagging, GitHub release, and push.
set -euo pipefail

# ── Setup ──────────────────────────────────────────────────────────────
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

source "${ROOT_DIR}/scripts/release-packages.sh"

# ── Defaults ───────────────────────────────────────────────────────────
DRY_RUN=0
CREATE_TAG=1
PUSH=0
ALLOW_DIRTY=0
REQUIRE_MAIN=1
VERSION=""
TAG_PREFIX="v"
TAG_NAME=""

# ── Helpers ────────────────────────────────────────────────────────────
usage() {
  cat <<'EOF'
Usage:
  ./scripts/release-publish.sh --version <version> [options]

Options:
  --version <version>   Release version (must match all crate Cargo.toml versions)
  --dry-run             Run publishing steps with --dry-run; skip tag and push
  --no-tag              Skip git tag and GitHub release creation
  --push                Push branch and tag after successful publish
  --allow-dirty         Allow a dirty git working tree
  --no-require-main     Allow releasing from non-main branches
  -h, --help            Show this help text
EOF
}

die() {
  echo "error: $*" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || die "$1 is required"
}

extract_manifest_version() {
  local manifest_path="$1"
  awk '
    /^\[package\]/ { in_package = 1; next }
    /^\[/ { in_package = 0 }
    in_package && $1 == "version" {
      gsub(/ /, "", $0)
      sub(/^version="/, "", $0)
      sub(/"$/, "", $0)
      print $0
      exit
    }
  ' "${manifest_path}"
}

# ── Validation ─────────────────────────────────────────────────────────
assert_version_format() {
  local version="$1"
  if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
    die "invalid release version '${version}'"
  fi
}

assert_clean_tree() {
  if [[ "${ALLOW_DIRTY}" -eq 1 ]]; then
    return
  fi
  if ! git diff --quiet || ! git diff --cached --quiet; then
    git status --short
    die "working tree is not clean; use --allow-dirty if intentional"
  fi
}

assert_branch() {
  local branch
  branch="$(git rev-parse --abbrev-ref HEAD)"
  if [[ "${branch}" == "HEAD" ]]; then
    die "refusing to release from detached HEAD"
  fi
  if [[ "${REQUIRE_MAIN}" -eq 1 && "${branch}" != "main" ]]; then
    die "releases must be run from main unless --no-require-main is passed"
  fi
}

assert_versions_match() {
  local package="$1"
  local mpath version
  mpath="$(manifest_path "${package}")" || die "unknown package: ${package}"
  version="$(extract_manifest_version "${mpath}")"
  if [[ -z "${version}" ]]; then
    die "unable to read version for ${package} from ${mpath}"
  fi
  if [[ "${version}" != "${VERSION}" ]]; then
    die "version mismatch for ${package}: ${version} != ${VERSION}"
  fi
}

# ── Publish Pipeline ───────────────────────────────────────────────────
publish_package() {
  local package="$1"
  local mpath cmd
  mpath="$(manifest_path "${package}")"

  if [[ "${DRY_RUN}" -eq 0 ]]; then
    if cargo search "${package}" --limit 1 2>/dev/null | grep -q "\"${VERSION}\""; then
      echo "== ${package}@${VERSION} already published, skipping =="
      return
    fi
    cmd=(cargo publish --manifest-path "${mpath}")
  else
    cmd=(cargo publish --manifest-path "${mpath}" --dry-run)
  fi

  if [[ "${ALLOW_DIRTY}" -eq 1 ]]; then
    cmd+=(--allow-dirty)
  fi

  echo "== ${cmd[*]} =="
  "${cmd[@]}"
}

# ── Tagging, GitHub Release & Push ─────────────────────────────────────
create_release_tag() {
  if [[ "${CREATE_TAG}" -eq 0 ]]; then
    return
  fi
  TAG_NAME="${TAG_PREFIX}${VERSION}"
  if git rev-parse "${TAG_NAME}" >/dev/null 2>&1; then
    local existing current
    existing="$(git rev-parse "${TAG_NAME}")"
    current="$(git rev-parse HEAD)"
    if [[ "${existing}" == "${current}" ]]; then
      echo "== tag ${TAG_NAME} already exists and points to HEAD; reusing =="
      return
    fi
    die "tag ${TAG_NAME} already exists at ${existing}, expected ${current}"
  fi
  git tag -a "${TAG_NAME}" -m "Release ${TAG_NAME}"
  echo "== created git tag ${TAG_NAME} =="
}

create_github_release() {
  if [[ "${CREATE_TAG}" -eq 0 || "${DRY_RUN}" -eq 1 ]]; then
    return
  fi
  if ! command -v gh >/dev/null 2>&1; then
    echo "== gh not found; skipping GitHub release =="
    return
  fi
  if gh release view "${TAG_NAME}" >/dev/null 2>&1; then
    echo "== GitHub release ${TAG_NAME} already exists; skipping =="
    return
  fi
  gh release create "${TAG_NAME}" \
    --title "${TAG_NAME}" \
    --notes "Release ${TAG_NAME}"
  echo "== created GitHub release ${TAG_NAME} =="
}

push_git_refs() {
  if [[ "${PUSH}" -eq 0 ]]; then
    return
  fi
  local branch
  branch="$(git rev-parse --abbrev-ref HEAD)"
  echo "== pushing branch ${branch} =="
  git push origin "${branch}"
  if [[ -n "${TAG_NAME}" ]]; then
    echo "== pushing tag ${TAG_NAME} =="
    git push origin "${TAG_NAME}"
  fi
}

# ── Main ───────────────────────────────────────────────────────────────
main() {
  require_command cargo
  require_command git

  while [[ "$#" -gt 0 ]]; do
    case "$1" in
      --version)
        [[ "$#" -lt 2 ]] && die "--version requires a value"
        VERSION="$2"; shift 2 ;;
      --version=*)
        VERSION="${1#*=}"; shift ;;
      --dry-run)
        DRY_RUN=1; shift ;;
      --no-tag)
        CREATE_TAG=0; shift ;;
      --push)
        PUSH=1; shift ;;
      --allow-dirty)
        ALLOW_DIRTY=1; shift ;;
      --no-require-main)
        REQUIRE_MAIN=0; shift ;;
      -h|--help)
        usage; exit 0 ;;
      *)
        die "unknown argument: $1" ;;
    esac
  done

  if [[ -z "${VERSION}" ]]; then
    VERSION="$(extract_manifest_version "${ROOT_DIR}/$(manifest_path "${RELEASE_PACKAGES[0]}")")"
  fi

  assert_version_format "${VERSION}"
  assert_branch
  assert_clean_tree

  for package in "${RELEASE_PACKAGES[@]}"; do
    echo "== validating version for ${package} =="
    assert_versions_match "${package}"
  done

  if [[ "${DRY_RUN}" -eq 0 && -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
    die "CARGO_REGISTRY_TOKEN is not set; publishing will fail"
  fi

  for package in "${RELEASE_PACKAGES[@]}"; do
    publish_package "${package}"
  done

  create_release_tag
  push_git_refs
  create_github_release

  echo "== release completed for ${VERSION} =="
}

main "$@"
