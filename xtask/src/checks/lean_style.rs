use std::{
    collections::BTreeSet,
    fs,
    path::Path,
};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::{LeanStyleConfig, ToolkitConfig},
    report::FlatFindingSet,
    util::{collect_lean_files, normalize_rel_path},
};

#[derive(Debug, Clone)]
struct DeclarationSpan {
    name: String,
    kind: String,
    start_line: usize,
    end_line: usize,
}

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.lean_style else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let file_exemptions = exempt_file_paths(check);
    let decl_exemptions = exempt_declarations(check);
    let mut findings = FlatFindingSet::default();

    for path in collect_lean_files(repo_root, &check.include_paths, &check.exclude_path_parts)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let lines: Vec<&str> = source.lines().collect();
        let line_count = source.lines().count();
        let file_exempt = file_exemptions.contains(rel.as_str());

        if !file_exempt && line_count > check.max_file_lines {
            findings.entries.insert(format!(
                "{rel}: file has {line_count} lines (limit {})",
                check.max_file_lines
            ));
        }

        if line_count >= check.non_trivial_file_lines {
            if !file_exempt
                && check.require_problem_statement
                && !has_problem_statement(&lines)
            {
                findings.entries.insert(format!(
                    "{rel}: non-trivial Lean file is missing a prose problem statement block near the top"
                ));
            }
            if !file_exempt
                && check.require_section_headers
                && line_count >= check.section_header_min_lines
                && !has_section_headers(&lines)
            {
                findings.entries.insert(format!(
                    "{rel}: non-trivial Lean file is missing `/-! ## ... -/` section headers"
                ));
            }
        }

        for declaration in declaration_spans(&source)? {
            if file_exempt
                || decl_exemptions.contains(&(rel.clone(), declaration.name.clone()))
            {
                continue;
            }

            let decl_lines = declaration.end_line + 1 - declaration.start_line;
            let over_target = decl_lines > check.max_decl_lines_target;
            let over_hard = decl_lines > check.max_decl_lines_hard_limit;
            if !over_target && !over_hard {
                continue;
            }

            let has_marker = has_over_limit_marker(
                &lines,
                &declaration,
                &check.over_limit_comment_markers,
            );
            let comment_required = check.require_over_limit_comment;

            if over_hard && !(comment_required && has_marker) {
                findings.entries.insert(format!(
                    "{rel}: {} `{}` spans {} lines (hard limit {}) without justification marker {:?}",
                    declaration.kind,
                    declaration.name,
                    decl_lines,
                    check.max_decl_lines_hard_limit,
                    check.over_limit_comment_markers
                ));
                continue;
            }

            if check.enforce_target_decl_lines && over_target && !(comment_required && has_marker)
            {
                findings.entries.insert(format!(
                    "{rel}: {} `{}` spans {} lines (target limit {}) without justification marker {:?}",
                    declaration.kind,
                    declaration.name,
                    decl_lines,
                    check.max_decl_lines_target,
                    check.over_limit_comment_markers
                ));
            }
        }
    }

    Ok(findings)
}

fn exempt_file_paths(check: &LeanStyleConfig) -> BTreeSet<&str> {
    check
        .file_exemptions
        .iter()
        .map(|exemption| {
            let _ = &exemption.reason;
            exemption.path.as_str()
        })
        .collect()
}

fn exempt_declarations(check: &LeanStyleConfig) -> BTreeSet<(String, String)> {
    check
        .declaration_exemptions
        .iter()
        .map(|exemption| {
            let _ = &exemption.reason;
            (exemption.path.clone(), exemption.name.clone())
        })
        .collect()
}

fn has_problem_statement(lines: &[&str]) -> bool {
    let mut index = 0usize;
    while let Some(line) = lines.get(index) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("import ") {
            index += 1;
            continue;
        }
        break;
    }

    while let Some(line) = lines.get(index) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            index += 1;
            continue;
        }
        if trimmed.starts_with("/-!") {
            index = advance_block_comment(lines, index);
            continue;
        }
        return trimmed.starts_with("/-") && !trimmed.starts_with("/-!");
    }

    false
}

fn has_section_headers(lines: &[&str]) -> bool {
    lines.iter().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("/-! ## ") || trimmed.starts_with("/-! ### ")
    })
}

fn advance_block_comment(lines: &[&str], start: usize) -> usize {
    let mut index = start;
    while let Some(line) = lines.get(index) {
        if line.contains("-/") {
            return index + 1;
        }
        index += 1;
    }
    lines.len()
}

fn declaration_spans(source: &str) -> Result<Vec<DeclarationSpan>> {
    let mut spans = Vec::new();
    let mut starts = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let start_re = Regex::new(
        r"^(?:(?:private|protected|noncomputable|unsafe|partial|scoped|local|opaque|mutual)\s+)*(theorem|lemma|def|abbrev|axiom|instance|inductive|structure|class|example)\b",
    )?;
    let boundary_re = Regex::new(
        r"^(?:theorem|lemma|def|abbrev|axiom|instance|inductive|structure|class|example|namespace|section|end)\b",
    )?;

    for (line_no, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let leading_ws = line.len() - trimmed.len();
        if leading_ws == 0 && start_re.is_match(trimmed) {
            starts.push((line_no + 1, trimmed.to_string()));
        }
    }

    for (idx, (start_line, header)) in starts.iter().enumerate() {
        let mut end_line = lines.len();
        for line_idx in *start_line..lines.len() {
            let trimmed = lines[line_idx].trim_start();
            let leading_ws = lines[line_idx].len() - trimmed.len();
            if leading_ws == 0
                && (trimmed.starts_with("/-! ## ")
                    || trimmed.starts_with("/-! ### ")
                    || boundary_re.is_match(trimmed))
            {
                end_line = line_idx;
                break;
            }
        }
        if let Some((next_start_line, _)) = starts.get(idx + 1) {
            if *next_start_line - 1 < end_line {
                end_line = *next_start_line - 1;
            }
        }
        while end_line > *start_line && lines[end_line - 1].trim().is_empty() {
            end_line -= 1;
        }
        spans.push(DeclarationSpan {
            name: declaration_name(header, *start_line),
            kind: declaration_kind(header),
            start_line: *start_line,
            end_line,
        });
    }

    Ok(spans)
}

fn declaration_name(header: &str, start_line: usize) -> String {
    let tokens: Vec<&str> = header.split_whitespace().collect();
    let Some(kind_pos) = tokens.iter().position(|token| {
        matches!(
            *token,
            "theorem"
                | "lemma"
                | "def"
                | "abbrev"
                | "axiom"
                | "instance"
                | "inductive"
                | "structure"
                | "class"
                | "example"
        )
    }) else {
        return format!("declaration@{start_line}");
    };
    let kind = tokens[kind_pos];
    let Some(name_token) = tokens.get(kind_pos + 1) else {
        return format!("{kind}@{start_line}");
    };
    if *name_token == ":" || *name_token == "where" {
        return format!("{kind}@{start_line}");
    }
    sanitize_name(name_token, kind, start_line)
}

fn declaration_kind(header: &str) -> String {
    header
        .split_whitespace()
        .find(|token| {
            matches!(
                *token,
                "theorem"
                    | "lemma"
                    | "def"
                    | "abbrev"
                    | "axiom"
                    | "instance"
                    | "inductive"
                    | "structure"
                    | "class"
                    | "example"
            )
        })
        .unwrap_or("declaration")
        .to_string()
}

fn sanitize_name(token: &str, kind: &str, start_line: usize) -> String {
    let trimmed = token
        .trim_matches(|ch: char| matches!(ch, '(' | ')' | ':' | '{' | '[' | '=' | ','));
    if trimmed.is_empty() {
        format!("{kind}@{start_line}")
    } else {
        trimmed.to_string()
    }
}

fn has_over_limit_marker(
    lines: &[&str],
    declaration: &DeclarationSpan,
    markers: &[String],
) -> bool {
    if markers.is_empty() {
        return false;
    }
    let start = declaration.start_line.saturating_sub(4);
    let end = declaration.end_line.min(lines.len());
    let lookback = lines[start..end].join("\n");
    markers.iter().any(|marker| lookback.contains(marker))
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::run;
    use crate::config::load;

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let dir = env::temp_dir().join(format!(
            "jacquard-toolkit-lean-style-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn lean_style_reports_missing_problem_statement() {
        let root = temp_dir("missing-problem");
        fs::create_dir_all(root.join("verification/Field")).expect("field dir");
        fs::write(
            root.join("toolkit.toml"),
            r#"
[workspace]
crate_roots = ["crates"]

[checks.lean_style]
enabled = true
include_paths = ["verification/Field"]
exclude_path_parts = []
non_trivial_file_lines = 5
section_header_min_lines = 5
max_file_lines = 500
max_decl_lines_target = 30
max_decl_lines_hard_limit = 50
enforce_target_decl_lines = false
require_problem_statement = true
require_section_headers = true
require_over_limit_comment = true
over_limit_comment_markers = ["long-block-exception:"]
"#,
        )
        .expect("config");
        fs::write(
            root.join("verification/Field/Test.lean"),
            "import Field.Model.API\n\nset_option autoImplicit false\n\n/-! ## Section -/\n\ndef foo : Nat :=\n  1\n",
        )
        .expect("lean file");

        let config = load(&root.join("toolkit.toml")).expect("load config");
        let findings = run(&root, &config).expect("run check");
        assert!(
            findings
                .entries
                .iter()
                .any(|entry| entry.contains("missing a prose problem statement")),
            "expected problem statement finding, got {findings:?}"
        );
    }

    #[test]
    fn lean_style_allows_exempt_long_file_and_marked_long_block() {
        let root = temp_dir("exemptions");
        fs::create_dir_all(root.join("verification/Field")).expect("field dir");
        fs::write(
            root.join("toolkit.toml"),
            r#"
[workspace]
crate_roots = ["crates"]

[checks.lean_style]
enabled = true
include_paths = ["verification/Field"]
exclude_path_parts = []
non_trivial_file_lines = 5
section_header_min_lines = 5
max_file_lines = 5
max_decl_lines_target = 3
max_decl_lines_hard_limit = 5
enforce_target_decl_lines = false
require_problem_statement = true
require_section_headers = true
require_over_limit_comment = true
over_limit_comment_markers = ["long-block-exception:"]

[[checks.lean_style.file_exemptions]]
path = "verification/Field/Test.lean"
reason = "fixture over file limit"
"#,
        )
        .expect("config");
        fs::write(
            root.join("verification/Field/Test.lean"),
            r#"import Field.Model.API

/-
Fixture problem statement.
-/

/-! ## Section -/

def short : Nat :=
  1

/-- long block fixture -/
def longBlock : Nat := by
  -- long-block-exception: fixture keeps one synthetic block for coverage
  have h0 : Nat := 0
  have h1 : Nat := 1
  have h2 : Nat := 2
  have h3 : Nat := 3
  exact h0 + h1 + h2 + h3
"#,
        )
        .expect("lean file");

        let config = load(&root.join("toolkit.toml")).expect("load config");
        let findings = run(&root, &config).expect("run check");
        assert!(findings.entries.is_empty(), "unexpected findings: {findings:?}");
    }
}
