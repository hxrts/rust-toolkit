# Toolkit

Portable enforcement tooling.

Rust policy machinery:

- reusable `dylint` crates
- reusable proc macros and effect-support traits
- generic Rust `xtask`-style checks
- fixture-based validation
- config loading and source discovery
- formatter, clippy, and dylint shell support

Lean policy machinery:

- generic Lean source-style checks over `.lean` files
- shared config parsing for Lean thresholds, imports, and exemptions

Consuming repositories keep domain-specific policy in a repo-owned `policy/`
directory, add the toolkit as a flake input, and call the exported toolkit
command surface from their default dev shell. See `docs/consuming_repos.md`
and `config/README.md`.
