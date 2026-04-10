# Consuming The Toolkit

Repositories that adopt the toolkit should keep their own entrypoints and
repository-specific policy, but consume the toolkit through a flake input.

This guide assumes the consuming repo:

- exposes toolkit commands from its default `nix develop` shell
- sets `TOOLKIT_ROOT` to the toolkit flake input path in `shellHook`
- keeps a tiny repo-local `scripts/toolkit-shell.sh` bootstrap for `just`, CI,
  and hooks

## Ownership Split

- toolkit repo:
  generic checks, reusable lints, fixture harnesses, config parsing, proc
  macros / effect support, and tooling shells
- consuming repo:
  local `justfile`, CI wiring, hooks, and a repo-owned `policy/` directory

## Recommended Layout

```text
repo/
  flake.nix
  flake.lock
  justfile
  .githooks/
  .github/
  policy/
    README.md
    toolkit.toml
    checks/
    lints/
    fixtures/
    docs/
```

The consuming repository keeps ownership of:

- `justfile`
- CI workflow commands
- pre-commit hooks
- the repo `flake.nix` and `flake.lock`
- small local bootstrap scripts such as `scripts/toolkit-shell.sh`
- `policy/`

Those entrypoints call into the toolkit commands exposed from the repo's own
default dev shell and pass the local config, usually
`--config policy/toolkit.toml`.

## Developer Workflow

1. Add the toolkit as a flake input and pin it in `flake.lock`.
2. Keep repo entrypoints local.
3. Add `policy/toolkit.toml`.
4. Add repo-local `policy/checks/`, `policy/lints/`, `policy/fixtures/`, and
   policy docs only as needed.
5. Export `TOOLKIT_ROOT` to the toolkit input path from the repo shell.
6. Add the toolkit command packages to the repo shell.
7. Add a tiny `scripts/toolkit-shell.sh` bootstrap.
8. Run toolkit commands from `just`, CI, and hooks through that bootstrap.
9. Add domain-specific rules only under `policy/`.

## Toolkit Command Surface

Inside the toolkit Nix shell, the reusable command surface is:

- `toolkit-xtask`
  Runs the generic `xtask` runner from the pinned toolkit checkout.
- `toolkit-fmt`
  Runs nightly `cargo fmt`, using either `--config <rustfmt.toml>` or the
  toolkit repo's own `rustfmt.toml`.
- `toolkit-install-dylint`
  Installs `cargo-dylint` and `dylint-link`, then links the pinned nightly
  toolchain name used by toolkit lint runs.
- `toolkit-dylint`
  Runs either a toolkit-owned lint (`--toolkit-lint <name>`) or a
  consumer-owned lint (`--lint-path <path>`) with the toolkit nightly setup.

Toolkit commands assume the consuming repo has added those packages to its own
shell and exported `TOOLKIT_ROOT` to the toolkit input path.

## Recommended Flake Wiring

The consuming repo should usually wire the toolkit in its own `flake.nix`:

```nix
inputs = {
  toolkit = {
    url = "github:hxrts/rust-toolkit";
    inputs.nixpkgs.follows = "nixpkgs";
    inputs.rust-overlay.follows = "rust-overlay";
    inputs.flake-utils.follows = "flake-utils";
  };
};
```

and then add the toolkit packages plus:

```nix
shellHook = ''
  export TOOLKIT_ROOT="${toolkit}"
'';
```

That keeps toolkit versioning repo-owned through the repo's flake lock while
making the toolkit command surface directly usable from `just`, CI, and hooks.

A minimal shell package set usually includes:

```nix
toolkit.packages.${system}.toolkit-xtask
toolkit.packages.${system}.toolkit-fmt
toolkit.packages.${system}.toolkit-install-dylint
toolkit.packages.${system}.toolkit-dylint
toolkit.packages.${system}.toolkit-dylint-link
```

## Copyable Bootstrap Script

Most consuming repos should keep one tiny wrapper at
`scripts/toolkit-shell.sh`. A canonical version lives in
[`docs/toolkit-shell.sh`](./toolkit-shell.sh) and can be copied directly.

Recommended content:

```bash
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
```

This wrapper does two things:

- if the caller is already inside the consuming repo's flake shell, it runs the
  toolkit command directly
- otherwise, it enters the pinned toolkit flake directly from `flake.lock`

That second path avoids snapshotting the whole consuming repo as a local flake
input, which is usually much faster on dirty worktrees.

After copying it into `scripts/toolkit-shell.sh`, make it executable:

```bash
chmod +x scripts/toolkit-shell.sh
```

## Rule Placement

- if a rule is generic and only the scope is repo-specific, configure it in
  `policy/toolkit.toml`
- if a rule depends on repo-specific architecture concepts, keep it under
  `policy/`
- if a repo-local rule later proves reusable, move it into the toolkit and
  delete the local copy

## Practical Command Shape

The consuming repo should usually call toolkit commands through that local
bootstrap:

```bash
./scripts/toolkit-shell.sh toolkit-xtask check <name> --repo-root . --config policy/toolkit.toml
./scripts/toolkit-shell.sh toolkit-install-dylint
./scripts/toolkit-shell.sh toolkit-dylint --repo-root . --toolkit-lint trait_purity --all -- --all-targets
./scripts/toolkit-shell.sh toolkit-dylint --repo-root . --lint-path ./policy/lints/model_policy --all -- --all-targets
./scripts/toolkit-shell.sh toolkit-fmt --all -- --check
```

Repo-specific policy commands remain repo-owned. For example, a consuming repo
can still run its own policy runner directly:

```bash
cargo run --manifest-path policy/xtask/Cargo.toml -- check <name>
```

## Minimal Adoption Checklist

1. Add `toolkit` as a flake input.
2. Expose toolkit packages from the repo's default shell.
3. Export `TOOLKIT_ROOT="${toolkit}"` from `shellHook`.
4. Copy [`docs/toolkit-shell.sh`](./toolkit-shell.sh) to
   `scripts/toolkit-shell.sh`.
5. `chmod +x scripts/toolkit-shell.sh`.
6. Add `policy/toolkit.toml`.
7. Point `just`, CI, and hooks at `./scripts/toolkit-shell.sh toolkit-...`.

## Repo-Local Dylint Requirements

If the consuming repo adds its own `policy/lints/*` crates, those lint crates
still need the normal Dylint linker setup. The recommended shape is:

```text
policy/
  lints/
    .cargo/
      config.toml
```

with:

```toml
[target.'cfg(not(target_os = "windows"))']
linker = "toolkit-dylint-link"
```

That linker wrapper is provided by the toolkit Nix shell and is required so
`cargo dylint` produces the `@toolchain` library filename that Dylint expects.
