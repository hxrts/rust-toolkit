# Consuming The Toolkit

Repositories that adopt the toolkit should keep their own entrypoints and
repository-specific policy.

## Expected Split

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

## Usage Rule

- if a rule is generic and only the scope is repo-specific, configure it in
  `policy/toolkit.toml`
- if a rule depends on repo-specific architecture concepts, keep it under
  `policy/`
