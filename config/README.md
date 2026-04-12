# Toolkit Config

Toolkit checks are configured by the consuming repository.

The intended local config file is `policy/toolkit.toml`.

At minimum, the config should define:

- workspace crate roots
- include and exclude scope
- per-check enablement
- per-check exemptions
- any repository-specific planned crate names or path roots needed by generic checks

## Rust Config Surface

Generic Rust checks usually need:

- `checks.<rust_check>.include_paths`
- optional `checks.<rust_check>.exclude_path_parts`
- threshold or allowlist keys that are still repository-owned but generic in
  meaning

Examples:

- `checks.unsafe_boundary.allowed_path_parts`
- `checks.unsafe_boundary.required_comment_markers`
- `checks.must_use_public_return.allowed_return_type_prefixes`
- `checks.public_type_width.banned_types`

## Lean Config Surface

Generic Lean source-style checks carry repo-owned thresholds and exemptions. The
Lean source-style check is configured in the consuming repo with keys such as:

- `checks.lean_style.include_paths`
- `checks.lean_style.max_file_lines`
- `checks.lean_style.max_decl_lines_target`
- `checks.lean_style.max_decl_lines_hard_limit`
- `checks.lean_style.require_problem_statement`
- `checks.lean_style.enforce_top_of_file_structure`
- `checks.lean_style.require_section_headers`
- `checks.lean_style.require_explanatory_comment_for_long_blocks`
- `checks.lean_style.require_public_theorem_lemma_docstrings`
- `checks.lean_style.forbid_sorry`
- `checks.lean_style.banned_imports`
- `checks.lean_style.banned_import_exemptions`
- `checks.lean_style.require_todo_for_sorry`
- `checks.lean_style.todo_comment_markers`
- `checks.lean_style.file_exemptions`
- `checks.lean_style.declaration_exemptions`

Keep Rust and Lean policy keys visibly separated in consuming-repo examples.
Mixed repos can use both, but the toolkit config surface should still make it
clear which keys apply to Rust files and which apply to Lean files.

Additional generic text and workspace checks can use keys such as:

- `checks.text_formatting.include_paths`
- `checks.text_formatting.exclude_path_parts`
- `checks.workspace_hygiene.include_paths`
- `checks.workspace_hygiene.exclude_path_parts`
- `checks.docs_semantic_drift.file_exemptions`

## Bundles

Bundles enable a named group of checks with shared path configuration and
conventional defaults. An explicit `[checks.*]` section always takes precedence
over a bundle default.

### `rust-base`

Enables all generic Rust checks in one block:

```toml
[bundles.rust_base]
enabled = true
rust_roots = ["crates"]
docs_roots = ["docs"]        # optional; omit to skip docs checks
manifest_path = "Cargo.toml" # optional; defaults to "Cargo.toml"
workflow_roots = [".github/workflows"] # optional; defaults to [".github/workflows"]
```

Checks activated by `rust-base`:

- `proc-macro-scope` (required_markers defaults to [])
- `result-must-use`
- `test-boundaries`
- `docs-link-check` (only when docs_roots is non-empty)
- `docs-semantic-drift` (only when docs_roots is non-empty)
- `text-formatting` (only when docs_roots is non-empty)
- `workspace-hygiene`
- `crate-root-policy` (required_attributes defaults to [])
- `ignored-result`
- `unsafe-boundary`
- `bool-param`
- `must-use-public-return`
- `assert-shape`
- `drop-side-effects`
- `recursion-guard`
- `naming-units`
- `limit-constant`
- `public-type-width` (banned_types defaults to [])
- `dependency-policy`
- `workflow-actions`

Override any check by adding its `[checks.*]` section alongside the bundle.
