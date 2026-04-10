# Repository Layout

This repository should stay focused on portable enforcement machinery.

## Top-Level Layout

```text
toolkit/
  README.md
  AGENTS.md
  rustfmt.toml
  clippy.toml
  effects/
  trait_contracts/
  macros/
  xtask/
  lints/
  fixtures/
  docs/
  nix/
  config/
```

## Directory Roles

- `effects/`
  Generic support traits and hidden marker types for effect vocabularies and
  handlers.
- `trait_contracts/`
  Shared expansion logic for portable trait-surface proc macros.
- `macros/`
  Public proc-macro crate for portable purity and effect-boundary annotations.
- `xtask/`
  Generic runner, config loading, source discovery, and reusable checks.
- `lints/`
  Portable `dylint` crates and shared dylint support files.
- `fixtures/`
  Reusable fixture repositories and expected results for validation.
- `docs/`
  Consumer workflow, ownership rules, and contributor guidance.
- `nix/`
  Hermetic formatter, clippy, and dylint shell support.
- `config/`
  Notes about shared config schema, defaults, and migration shape.

## Ownership Rules

This repo owns:

- generic check semantics
- generic lint semantics
- shared tooling shells
- shared fixture harnesses

This repo does not own:

- Jacquard-specific policy
- another repo's exemptions
- another repo's architecture boundaries
- hardcoded crate names or repo paths that only make sense for one consumer

## Design Constraints

- Prefer config-driven scope over hardcoded exceptions.
- Keep the toolkit path-independent.
- Do not assume the checkout lives at `repo/toolkit`.
- Keep `README.md` short and move substantial guidance into `docs/`.
- Add tests or fixtures when introducing non-trivial enforcement behavior.
