# Toolkit

Portable Rust enforcement tooling intended to live in its own repository and be
imported by consuming repositories.

The toolkit owns reusable policy machinery:

- generic `xtask`-style checks
- reusable `dylint` crates
- formatter and lint execution helpers
- hermetic Nix tooling
- fixture-based validation
- shared config loading and source-discovery support

The toolkit does not own repository-specific policy. Each consuming repository
keeps its domain rules, exemptions, and local scope configuration in a
repo-owned `policy/` directory.

## Toolkit Layout

```text
toolkit/
  README.md
  rustfmt.toml
  xtask/
  lints/
  fixtures/
  docs/
  nix/
  config/
  templates/
```

## Directory Roles

- `toolkit/xtask/`
  Generic runner and reusable source-scan checks.
- `toolkit/lints/`
  Portable `dylint` crates and shared dylint support files.
- `toolkit/fixtures/`
  Reusable fixture inputs and expected outputs for toolkit validation.
- `toolkit/docs/`
  Toolkit-level design docs, rule-family docs, config docs, and extraction notes.
- `toolkit/nix/`
  Hermetic formatter, lint, and tooling shell support.
- `toolkit/config/`
  Shared config schema notes, defaults, and adapter examples.
- `toolkit/templates/`
  Starter files for repositories adopting the toolkit.

## Consuming Repository Layout

Each repository that adopts the toolkit should keep a local policy layer beside
its normal source tree:

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

The repository keeps ownership of developer entrypoints such as:

- `justfile`
- CI workflow commands
- pre-commit hooks
- repository-local `policy/`

Those entrypoints call into the imported toolkit with the repository's local
policy config, typically `--config policy/toolkit.toml`.

## Developer Workflow

For a repository adopting the toolkit, the intended workflow is:

1. Pin an explicit toolkit version.
2. Keep local entrypoints in the repository.
3. Add `policy/toolkit.toml` describing crate roots, scope, enablement, and exceptions.
4. Run toolkit checks through local wrappers such as `just`, CI, and hooks.
5. Add domain-specific checks and lints only under `policy/`.

The rule for new enforcement should be:

- if the rule is generic and only its scope is repository-specific, keep the
  rule in the toolkit and express the scope in `policy/toolkit.toml`
- if the rule semantics depend on repository architecture or domain language,
  keep the rule in `policy/`
- if a local rule later proves reusable, promote it into the toolkit and remove
  the local copy

## Portable vs Repository-Specific

The split is:

- toolkit core:
  generic checks, generic lint crates, formatter and lint shells, config
  loading, and fixture harnesses
- repository policy:
  crate roots, include and exclude scope, exemptions, and domain-specific rules

If a rule can be described without naming repository-specific concepts, it
belongs in the toolkit.

If a rule depends on repository architecture concepts, crate names, ownership
language, or domain invariants, it belongs in the repository's `policy/`
directory.
