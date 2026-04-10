#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

if [ -n "${IN_NIX_SHELL:-}" ] && [ -n "${TOOLKIT_ROOT:-}" ] && command -v toolkit-xtask >/dev/null 2>&1; then
  exec "$@"
fi

toolkit_flake_ref="$(
  python3 - "$repo_root/flake.lock" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    lock = json.load(handle)

node = lock["nodes"]["toolkit"]["locked"]
if node.get("type") != "github":
    raise SystemExit(f"unsupported toolkit lock type: {node.get('type')}")

ref = f"github:{node['owner']}/{node['repo']}/{node['rev']}"
nar_hash = node.get("narHash")
if nar_hash:
    ref += f"?narHash={nar_hash}"

print(ref)
PY
)"

exec nix develop "$toolkit_flake_ref" --command "$@"
