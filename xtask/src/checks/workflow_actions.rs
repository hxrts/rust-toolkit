use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use regex::Regex;
use walkdir::WalkDir;

use crate::{
    config::{ToolkitConfig, WorkflowActionsConfig},
    report::FlatFindingSet,
    util::normalize_rel_path,
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.workflow_actions else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let uses_re = Regex::new(
        r#"^\s*(?:-\s*)?uses:\s*["']?([^[:space:]#"']+)["']?(?:\s*#\s*(.*))?$"#,
    )?;
    let mut unresolved: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut findings = FlatFindingSet::default();

    for workflow in collect_workflow_files(repo_root, &check.workflow_roots) {
        scan_workflow(
            repo_root,
            &workflow,
            check,
            &uses_re,
            &mut unresolved,
            &mut findings,
        )?;
    }

    for (spec, mut locations) in unresolved {
        locations.sort();
        findings.entries.insert(format!(
            "{}: unresolved GitHub Action reference {spec}",
            locations.join(", ")
        ));
    }

    Ok(findings)
}

fn collect_workflow_files(root: &Path, workflow_roots: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for rel_root in workflow_roots {
        let dir = root.join(rel_root);
        if !dir.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&dir).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let Some(ext) = entry.path().extension().and_then(|ext| ext.to_str()) else {
                continue;
            };
            if ext != "yml" && ext != "yaml" {
                continue;
            }
            files.push(entry.into_path());
        }
    }
    files.sort();
    files.dedup();
    files
}

fn scan_workflow(
    repo_root: &Path,
    workflow: &Path,
    check: &WorkflowActionsConfig,
    uses_re: &Regex,
    unresolved: &mut BTreeMap<String, Vec<String>>,
    findings: &mut FlatFindingSet,
) -> Result<()> {
    let rel = normalize_rel_path(repo_root, workflow);
    let contents = std::fs::read_to_string(workflow)
        .with_context(|| format!("reading {}", workflow.display()))?;
    for (index, line) in contents.lines().enumerate() {
        let line_no = index + 1;
        let Some(captures) = uses_re.captures(line) else {
            continue;
        };
        let Some(spec_match) = captures.get(1) else {
            continue;
        };
        let spec = spec_match.as_str().trim_matches('"').trim_matches('\'');
        if spec.starts_with("./") || spec.starts_with("docker://") {
            continue;
        }
        if !spec.contains('@') {
            findings.entries.insert(format!(
                "{rel}:{line_no}: malformed action reference without @ref: {spec}"
            ));
            continue;
        }

        let comment = captures.get(2).map(|item| item.as_str()).unwrap_or_default();
        if has_pin_comment(comment, &check.pin_comment_markers) {
            continue;
        }

        let repo = spec.split('@').next().unwrap_or_default();
        let git_ref = spec.rsplit('@').next().unwrap_or_default();
        if !remote_ref_exists(repo, git_ref)? {
            unresolved
                .entry(spec.to_string())
                .or_default()
                .push(format!("{rel}:{line_no}"));
        }
    }
    Ok(())
}

fn has_pin_comment(comment: &str, markers: &[String]) -> bool {
    let lowered = comment.to_ascii_lowercase();
    markers.iter().any(|marker| {
        let marker = marker.trim().to_ascii_lowercase();
        !marker.is_empty() && lowered.contains(&marker)
    })
}

fn remote_ref_exists(repo: &str, git_ref: &str) -> Result<bool> {
    let output = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args([
            "-c",
            "credential.helper=",
            "-c",
            "core.askPass=",
            "-c",
            "credential.interactive=never",
            "ls-remote",
            "--exit-code",
            &format!("https://github.com/{repo}.git"),
            &format!("refs/tags/{git_ref}"),
            &format!("refs/heads/{git_ref}"),
        ])
        .output()
        .with_context(|| format!("running git ls-remote for {repo}@{git_ref}"))?;
    Ok(output.status.success())
}
