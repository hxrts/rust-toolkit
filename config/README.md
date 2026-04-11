# Toolkit Config

Toolkit checks are configured by the consuming repository.

The intended local config file is `policy/toolkit.toml`.

At minimum, the config should define:

- workspace crate roots
- include and exclude scope
- per-check enablement
- per-check exemptions
- any repository-specific planned crate names or path roots needed by generic checks

Generic source-policy checks can also carry repo-owned thresholds. For example,
the Lean source-style check is configured in the consuming repo with keys such
as:

- `checks.lean_style.include_paths`
- `checks.lean_style.max_file_lines`
- `checks.lean_style.max_decl_lines_target`
- `checks.lean_style.max_decl_lines_hard_limit`
- `checks.lean_style.require_problem_statement`
- `checks.lean_style.enforce_top_of_file_structure`
- `checks.lean_style.require_section_headers`
- `checks.lean_style.require_explanatory_comment_for_long_blocks`
- `checks.lean_style.require_public_theorem_lemma_docstrings`
- `checks.lean_style.banned_imports`
- `checks.lean_style.banned_import_exemptions`
- `checks.lean_style.require_todo_for_sorry`
- `checks.lean_style.todo_comment_markers`
- `checks.lean_style.file_exemptions`
- `checks.lean_style.declaration_exemptions`
