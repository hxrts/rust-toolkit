use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::{LeanEscapeHatchesConfig, ToolkitConfig},
    report::FlatFindingSet,
    util::{collect_lean_files, normalize_rel_path},
};

struct PatternSpec {
    kind: &'static str,
    description: &'static str,
    regex: Regex,
}

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.lean_escape_hatches else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let line_patterns = compile_line_patterns()?;
    let shell_patterns = compile_shell_patterns()?;
    let exemptions = exemption_map(check);
    let mut by_kind: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for path in collect_lean_files(repo_root, &check.include_paths, &check.exclude_path_parts)? {
        scan_file(
            repo_root,
            &path,
            check,
            &exemptions,
            &line_patterns,
            &shell_patterns,
            &mut by_kind,
        )?;
    }

    let mut findings = FlatFindingSet::default();
    for (kind, entries) in by_kind {
        let threshold = check.kind_thresholds.get(&kind).copied().unwrap_or(0);
        if entries.len() <= threshold {
            continue;
        }
        findings.entries.insert(format!(
            "lean escape hatch `{kind}` count {} exceeds threshold {threshold}",
            entries.len()
        ));
        for entry in entries {
            findings.entries.insert(entry);
        }
    }

    Ok(findings)
}

fn scan_file(
    repo_root: &Path,
    path: &Path,
    check: &LeanEscapeHatchesConfig,
    exemptions: &BTreeMap<String, BTreeSet<String>>,
    line_patterns: &[PatternSpec],
    shell_patterns: &[(Regex, &'static str)],
    by_kind: &mut BTreeMap<String, Vec<String>>,
) -> Result<()> {
    let rel = normalize_rel_path(repo_root, path);
    let source = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let exempt_kinds = exemptions.get(&rel);
    let mut seen_shell_lines = BTreeSet::new();

    for (index, line) in source.lines().enumerate() {
        let line_no = index + 1;
        for spec in line_patterns {
            if !spec.regex.is_match(line) || is_exempt(exempt_kinds, spec.kind) {
                continue;
            }
            by_kind
                .entry(spec.kind.to_string())
                .or_default()
                .push(format!(
                    "{rel}:{line_no}: lean escape hatch `{}`: {}",
                    spec.kind, spec.description
                ));
        }
    }

    for (regex, variant) in shell_patterns {
        for capture in regex.find_iter(&source) {
            if is_exempt(exempt_kinds, "empty_theorem_shell") {
                continue;
            }
            let line_no = source[..capture.start()].bytes().filter(|byte| *byte == b'\n').count()
                + 1;
            if !seen_shell_lines.insert(line_no) {
                continue;
            }
            by_kind
                .entry("empty_theorem_shell".to_string())
                .or_default()
                .push(format!(
                    "{rel}:{line_no}: lean escape hatch `empty_theorem_shell`: empty proposition shell ({variant})"
                ));
        }
    }

    let _ = check;
    Ok(())
}

fn compile_line_patterns() -> Result<Vec<PatternSpec>> {
    Ok(vec![
        PatternSpec {
            kind: "sorry",
            description: "admits a goal without proof",
            regex: Regex::new(r"\bsorry\b")?,
        },
        PatternSpec {
            kind: "sorry_ax",
            description: "uses the sorry axiom",
            regex: Regex::new(r"sorryAx")?,
        },
        PatternSpec {
            kind: "axiom",
            description: "introduces an unproven assumption",
            regex: Regex::new(r"^[[:space:]]*axiom\b")?,
        },
        PatternSpec {
            kind: "private_axiom",
            description: "introduces a private unproven assumption",
            regex: Regex::new(r"^[[:space:]]*private[[:space:]]+axiom\b")?,
        },
        PatternSpec {
            kind: "lc_proof",
            description: "uses low-level proof bypass",
            regex: Regex::new(r"lcProof")?,
        },
        PatternSpec {
            kind: "decreasing_by_sorry",
            description: "uses an unproved termination argument",
            regex: Regex::new(r"decreasing_by[[:space:]]+sorry")?,
        },
        PatternSpec {
            kind: "unsafe",
            description: "disables Lean safety checks",
            regex: Regex::new(r"\bunsafe\b")?,
        },
        PatternSpec {
            kind: "partial_def",
            description: "disables termination checking",
            regex: Regex::new(r"^[[:space:]]*partial[[:space:]]+def\b")?,
        },
        PatternSpec {
            kind: "csimp",
            description: "uses the `@[csimp]` escape hatch",
            regex: Regex::new(r"@\[(?:.*\s)?csimp(?:\s.*)?\]")?,
        },
        PatternSpec {
            kind: "unsafe_cast",
            description: "uses `unsafeCast`",
            regex: Regex::new(r"unsafeCast")?,
        },
        PatternSpec {
            kind: "panic",
            description: "uses `panic!`",
            regex: Regex::new(r"\bpanic!\b")?,
        },
        PatternSpec {
            kind: "unreachable",
            description: "uses `unreachable!`",
            regex: Regex::new(r"\bunreachable!\b")?,
        },
        PatternSpec {
            kind: "native_decide",
            description: "uses native code for decidability",
            regex: Regex::new(r"native_decide")?,
        },
        PatternSpec {
            kind: "implemented_by",
            description: "uses `implemented_by` indirection",
            regex: Regex::new(r"implemented_by")?,
        },
        PatternSpec {
            kind: "extern",
            description: "uses foreign-function linkage",
            regex: Regex::new(r"\bextern\b")?,
        },
        PatternSpec {
            kind: "reduce_bool",
            description: "uses native boolean reduction",
            regex: Regex::new(r"Lean\.reduceBool")?,
        },
        PatternSpec {
            kind: "reduce_nat",
            description: "uses native natural-number reduction",
            regex: Regex::new(r"Lean\.reduceNat")?,
        },
        PatternSpec {
            kind: "opaque",
            description: "hides an implementation behind `opaque`",
            regex: Regex::new(r"\bopaque\b")?,
        },
        PatternSpec {
            kind: "noncomputable",
            description: "marks a declaration or section as noncomputable",
            regex: Regex::new(r"\bnoncomputable\b")?,
        },
    ])
}

fn compile_shell_patterns() -> Result<Vec<(Regex, &'static str)>> {
    Ok(vec![
        (
            Regex::new(
                r"(?ms)^\s*(?:def|abbrev)\s+[A-Za-z0-9_'.]+\s*:[\s\S]{0,240}?\bProp\b[\s\S]{0,120}?\:=\s*True\b",
            )?,
            "Prop := True",
        ),
        (
            Regex::new(
                r"(?ms)^\s*(?:def|abbrev|theorem|lemma)\s+[A-Za-z0-9_'.]+\s*:[\s\S]{0,240}?\bProp\b[\s\S]{0,160}?\:=\s*by\s*(?:trivial|exact\s+True\.intro)\b",
            )?,
            "Prop := by trivial",
        ),
    ])
}

fn exemption_map(
    check: &LeanEscapeHatchesConfig,
) -> BTreeMap<String, BTreeSet<String>> {
    check
        .file_exemptions
        .iter()
        .map(|exemption| {
            let _ = &exemption.reason;
            (
                exemption.path.clone(),
                exemption.kinds.iter().cloned().collect::<BTreeSet<_>>(),
            )
        })
        .collect()
}

fn is_exempt(exempt_kinds: Option<&BTreeSet<String>>, kind: &str) -> bool {
    let Some(exempt_kinds) = exempt_kinds else {
        return false;
    };
    exempt_kinds.is_empty() || exempt_kinds.contains(kind)
}
