# Rust Toolkit

Portable Rust enforcement tooling.

Policy machinery:

- reusable `dylint` crates
- reusable proc macros and effect-support traits
- generic `xtask`-style checks
- fixture-based validation
- config loading and source discovery
- formatter, clippy, and dylint shell support

Consuming repositories keep domain-specific policy in a repo-owned `policy/`
directory, resolve a pinned toolkit checkout locally, and call the toolkit
command surface from its Nix shell. See `docs/consuming_repos.md`.
