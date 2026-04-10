use std::{path::Path, process::Command};

use anyhow::{bail, Context, Result};

use crate::report::{FindingSet, FlatFindingSet};

pub fn run_proc_macro_scope(repo_root: &Path) -> Result<FindingSet> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            "legacy/crates-xtask/Cargo.toml",
            "--",
            "check",
            "proc-macro-scope",
        ])
        .current_dir(repo_root)
        .output()
        .context("running archived legacy xtask check proc-macro-scope")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    let findings = parse_findings(&combined);
    if output.status.success() != findings.is_empty() {
        bail!(
            "legacy proc-macro-scope status/output mismatch: success={} findings_empty={}",
            output.status.success(),
            findings.is_empty()
        );
    }
    Ok(findings)
}

fn parse_findings(output: &str) -> FindingSet {
    let mut findings = FindingSet::default();
    enum Section {
        None,
        Stale,
        Missing,
    }
    let mut section = Section::None;
    for line in output.lines() {
        let trimmed = line.trim_end();
        match trimmed {
            | "stale proc-macro exemptions:" => {
                section = Section::Stale;
                continue;
            },
            | "missing proc-macro file coverage:" => {
                section = Section::Missing;
                continue;
            },
            | _ => {},
        }
        let Some(entry) = trimmed.strip_prefix("  ") else {
            continue;
        };
        match section {
            | Section::Stale => {
                findings.stale.insert(entry.to_string());
            },
            | Section::Missing => {
                findings.missing.insert(entry.to_string());
            },
            | Section::None => {},
        }
    }
    findings
}

pub fn run_flat_check(repo_root: &Path, check_name: &str) -> Result<FlatFindingSet> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            "legacy/crates-xtask/Cargo.toml",
            "--",
            "check",
            check_name,
        ])
        .current_dir(repo_root)
        .output()
        .with_context(|| format!("running archived legacy xtask check {check_name}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    let findings = parse_flat_findings(&combined);
    if output.status.success() != findings.is_empty() {
        bail!(
            "legacy {check_name} status/output mismatch: success={} findings_empty={}",
            output.status.success(),
            findings.is_empty()
        );
    }
    Ok(findings)
}

fn parse_flat_findings(output: &str) -> FlatFindingSet {
    let mut findings = FlatFindingSet::default();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.ends_with(": OK")
            || trimmed == "test-boundaries: violation(s)"
            || trimmed.starts_with("Finished `")
            || trimmed.starts_with("Running `")
        {
            continue;
        }
        if let Some(entry) = trimmed.strip_prefix("  ") {
            findings.entries.insert(entry.to_string());
            continue;
        }
        if trimmed.starts_with("missing docs link: ")
            || trimmed.starts_with("link to private scratch directory: ")
            || trimmed.starts_with("absolute path in link: ")
            || trimmed.contains(": unknown just recipe `")
            || trimmed.contains(": unresolved path `")
            || trimmed.contains(": unresolved qualified symbol `")
            || trimmed.contains(": unresolved symbol `")
            || trimmed.contains(": trait ")
        {
            findings.entries.insert(trimmed.to_string());
        }
    }
    findings
}
