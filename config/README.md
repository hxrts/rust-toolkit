# Toolkit Config

Toolkit checks are configured by the consuming repository.

The intended local config file is `policy/toolkit.toml`.

At minimum, the config should define:

- workspace crate roots
- include and exclude scope
- per-check enablement
- per-check exemptions
- any repository-specific planned crate names or path roots needed by generic checks
