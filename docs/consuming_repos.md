# Consuming The Toolkit

Repositories that adopt the toolkit should keep their own entrypoints and
repository-specific policy.

## Ownership Split

- toolkit repo:
  generic checks, reusable lints, fixture harnesses, config parsing, and
  tooling shells
- consuming repo:
  local `justfile`, CI wiring, hooks, and a repo-owned `policy/` directory

## Recommended Layout

```text
repo/
  justfile
  .githooks/
  .github/
  policy/
    README.md
    toolkit.toml
    checks/
    lints/
    fixtures/
    exemptions/
    docs/
```

The consuming repository keeps ownership of:

- `justfile`
- CI workflow commands
- pre-commit hooks
- any repo-local toolkit resolver such as `scripts/toolkit-shell.sh`
- `policy/`

Those entrypoints call into the pinned toolkit checkout and pass the local
config, usually `--config policy/toolkit.toml`.

## Developer Workflow

1. Pin an explicit toolkit revision.
2. Keep repo entrypoints local.
3. Add `policy/toolkit.toml`.
4. Resolve the pinned toolkit checkout through one local shell entrypoint.
5. Run toolkit commands from `just`, CI, and hooks through that shell.
6. Add domain-specific rules only under `policy/`.

## Rule Placement

- if a rule is generic and only the scope is repo-specific, configure it in
  `policy/toolkit.toml`
- if a rule depends on repo-specific architecture concepts, keep it under
  `policy/`
- if a repo-local rule later proves reusable, move it into the toolkit and
  delete the local copy

## Practical Command Shape

The consuming repo should usually keep only one local wrapper that resolves the
pinned toolkit checkout and enters `nix develop path:<toolkit-root>/nix`.

Inside that shell, the toolkit exposes reusable commands directly:

```bash
./scripts/toolkit-shell.sh toolkit-xtask check <name> --repo-root . --config policy/toolkit.toml
./scripts/toolkit-shell.sh toolkit-install-dylint
./scripts/toolkit-shell.sh toolkit-dylint --repo-root . --toolkit-lint trait_purity --all -- --all-targets
./scripts/toolkit-shell.sh toolkit-dylint --repo-root . --lint-path ./policy/lints/model_policy --all -- --all-targets
./scripts/toolkit-shell.sh toolkit-fmt --config ./rustfmt.toml --all -- --check
```

Repo-specific policy commands remain repo-owned. For example, a consuming repo
can still run its own policy runner directly:

```bash
cargo run --manifest-path policy/xtask/Cargo.toml -- check <name>
```
