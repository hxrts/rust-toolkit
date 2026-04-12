#!/usr/bin/env bash

# Publishable crates in release order (leaves first, dependents last).
#
# Dependency order:
#   trait-contracts, effects (no toolkit deps)
#   macros -> trait-contracts
RELEASE_PACKAGES=(
  "rust-toolkit-trait-contracts"
  "rust-toolkit-effects"
  "rust-toolkit-macros"
)

manifest_path() {
  local crate="$1"
  case "${crate}" in
    rust-toolkit-trait-contracts) echo "trait_contracts/Cargo.toml" ;;
    rust-toolkit-effects)         echo "effects/Cargo.toml" ;;
    rust-toolkit-macros)          echo "macros/Cargo.toml" ;;
    *)
      echo "unknown package: ${crate}" >&2
      return 1
      ;;
  esac
}
