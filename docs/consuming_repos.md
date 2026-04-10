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
- any repo-local wrapper scripts
- `policy/`

Those entrypoints call into the pinned toolkit checkout and pass the local
config, usually `--config policy/toolkit.toml`.

## Developer Workflow

1. Pin an explicit toolkit revision.
2. Keep repo entrypoints local.
3. Add `policy/toolkit.toml`.
4. Run toolkit checks through local wrappers from `just`, CI, and hooks.
5. Add domain-specific rules only under `policy/`.

## Rule Placement

- if a rule is generic and only the scope is repo-specific, configure it in
  `policy/toolkit.toml`
- if a rule depends on repo-specific architecture concepts, keep it under
  `policy/`
- if a repo-local rule later proves reusable, move it into the toolkit and
  delete the local copy

## Practical Command Shape

The toolkit should be called through repo-local wrappers, but the underlying
shape usually looks like:

```bash
cargo run --manifest-path <toolkit-root>/xtask/Cargo.toml -- \
  check <name> --repo-root . --config policy/toolkit.toml
```

Portable nightly lint flows should also be wrapped locally so the consuming
repo can decide how to resolve and pin the toolkit checkout.
