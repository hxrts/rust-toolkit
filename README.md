# Toolkit

Portable Rust enforcement tooling for consuming repositories.

This repository owns reusable policy machinery:

- generic `xtask`-style checks
- reusable `dylint` crates
- reusable proc macros and effect-support traits
- fixture-based validation
- config loading and source discovery
- formatter, clippy, and dylint shell support

Consuming repositories keep domain-specific policy in a repo-owned `policy/`
directory and call this toolkit through local wrappers.

Documentation:

- [Consumer workflow](docs/consuming_repos.md)
- [Repository layout and ownership](docs/repo_layout.md)
- [Agent instructions](AGENTS.md)
