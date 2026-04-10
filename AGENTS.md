# AGENTS.md

This repository contains reusable Rust policy tooling. It is not the place for
repository-specific architecture checks.

## Mission

The toolkit owns portable enforcement machinery:

- generic `xtask` checks
- reusable `dylint` crates
- reusable proc macros and effect-support traits
- fixture-backed validation
- shared config parsing and source discovery
- formatter, clippy, and dylint shell support

A consuming repository owns:

- `policy/toolkit.toml`
- domain-specific checks
- domain-specific lint crates
- repo-specific exemptions
- local `just`, CI, and hook entrypoints
- the one local wrapper that resolves the pinned toolkit checkout and enters
  the toolkit Nix shell

If a proposed rule needs to name repo-specific concepts, crate topology, or
domain language, it does not belong here.

## Working Rules

- Keep this repo path-independent. Do not assume it lives under
  `repo/toolkit/`; resolve paths from `CARGO_MANIFEST_DIR`, explicit arguments,
  or environment variables.
- Prefer config-driven generic behavior over hardcoded repository names,
  exemptions, or crate paths.
- Add new generic checks under `xtask/src/checks/`.
- Add reusable compiler-backed lints under `lints/`.
- Add reusable trait-surface proc macros under `macros/` and shared expansion
  logic under `trait_contracts/`.
- Keep effect support traits and hidden marker types in `effects/`.
- Add fixture coverage under `fixtures/` and `xtask/tests/` when behavior is
  subtle or easy to regress.
- Keep docs in `docs/` and keep the root `README.md` short.
- Do not add consuming-repo policy examples that imply this repo owns a local
  `policy/` directory.

## Commands

Run commands from the toolkit repo root.

```bash
nix develop path:./nix --command toolkit-install-dylint
nix develop path:./nix --command toolkit-xtask show-config --repo-root <repo> --config <repo>/policy/toolkit.toml
nix develop path:./nix --command toolkit-xtask check <name> --repo-root <repo> --config <repo>/policy/toolkit.toml
nix develop path:./nix --command toolkit-xtask parity <name> --repo-root <repo> --config <repo>/policy/toolkit.toml
nix develop path:./nix --command toolkit-fmt --config ./rustfmt.toml --all -- --check
nix develop path:./nix --command toolkit-dylint --repo-root <repo> --toolkit-lint trait_purity --all -- --all-targets
```

## Adding New Enforcement

Use this decision rule:

1. If the rule is generic and only its scope varies by repo, add config support.
2. If the rule semantics are reusable across repos, implement it here.
3. If the rule depends on one repo's architecture language, leave it in that
   repo's `policy/`.

When adding a generic rule:

1. Add the implementation.
2. Wire it into the `xtask` command surface.
3. Add or update fixture coverage.
4. Document any config keys or ownership constraints in `docs/`.

## Editing Constraints

- Default to ASCII.
- Use succinct comments only when the code is not self-explanatory.
- Do not silently bake Jacquard-specific assumptions back into shared code.
- Preserve existing user changes in this repo unless explicitly asked to revert
  them.
