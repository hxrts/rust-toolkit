use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::{DocsIndexConfig, ToolkitConfig},
    report::FlatFindingSet,
    util::normalize_rel_path,
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.docs_index else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let mut findings = FlatFindingSet::default();
    let docs_root = repo_root.join(&check.docs_root);
    let index_path = repo_root.join(&check.index_file);
    if !docs_root.is_dir() {
        findings.entries.insert(format!(
            "missing docs root: {}",
            normalize_rel_path(repo_root, &docs_root)
        ));
        return Ok(findings);
    }
    if !index_path.is_file() {
        findings.entries.insert(format!(
            "missing docs index file: {}",
            normalize_rel_path(repo_root, &index_path)
        ));
        return Ok(findings);
    }

    let index_entries =
        parse_index_entries(repo_root, &index_path, check, &mut findings)?;
    let expected_files = collect_expected_files(&docs_root, &check.exclude_files)?;
    for file_name in &expected_files {
        if !index_entries.contains_key(file_name) {
            findings.entries.insert(format!(
                "{}: missing index entry for {file_name}",
                normalize_rel_path(repo_root, &index_path)
            ));
        }
    }

    for (file_name, entry) in &index_entries {
        let target = docs_root.join(file_name);
        if !target.is_file() {
            findings.entries.insert(format!(
                "{}:{}: index entry points to non-existent file: {file_name}",
                normalize_rel_path(repo_root, &index_path),
                entry.line_no
            ));
            continue;
        }
        if check
            .exclude_files
            .iter()
            .any(|excluded| excluded == file_name)
        {
            findings.entries.insert(format!(
                "{}:{}: index must not include excluded file: {file_name}",
                normalize_rel_path(repo_root, &index_path),
                entry.line_no
            ));
        }
        if let Some(actual_title) = extract_h1(&target)? {
            if actual_title != entry.link_text {
                findings.entries.insert(format!(
                    "{}:{}: link text '{}' does not match H1 title '{}' in {file_name}",
                    normalize_rel_path(repo_root, &index_path),
                    entry.line_no,
                    entry.link_text,
                    actual_title
                ));
            }
        }
    }

    Ok(findings)
}

#[derive(Debug, Clone)]
struct IndexEntry {
    link_text: String,
    line_no: usize,
}

fn parse_index_entries(
    repo_root: &Path,
    index_path: &Path,
    check: &DocsIndexConfig,
    findings: &mut FlatFindingSet,
) -> Result<BTreeMap<String, IndexEntry>> {
    let contents = fs::read_to_string(index_path)
        .with_context(|| format!("reading {}", index_path.display()))?;
    let row_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").expect("valid row regex");
    let mut entries = BTreeMap::new();
    let mut in_table = false;
    let mut heading_found = false;

    for (idx, line) in contents.lines().enumerate() {
        let line_no = idx + 1;
        if line == check.heading {
            heading_found = true;
            in_table = true;
            continue;
        }
        if in_table && line.starts_with("## ") && line != check.heading {
            break;
        }
        if !in_table || !line.starts_with('|') {
            continue;
        }
        if line.starts_with("| Document |") || is_separator_row(line) {
            continue;
        }
        let Some(captures) = row_re.captures(line) else {
            continue;
        };
        let Some(link_text) = captures.get(1).map(|m| m.as_str().to_string()) else {
            continue;
        };
        let Some(target) = captures.get(2).map(|m| m.as_str()) else {
            continue;
        };
        let file_name = normalize_target_file_name(target);
        entries.insert(file_name, IndexEntry { link_text, line_no });
    }

    let rel_index = normalize_rel_path(repo_root, index_path);
    if !heading_found {
        findings
            .entries
            .insert(format!("{rel_index}: missing heading '{}'", check.heading));
    }
    if heading_found && entries.is_empty() {
        findings.entries.insert(format!(
            "{rel_index}: no index entries found under '{}'",
            check.heading
        ));
    }
    Ok(entries)
}

fn is_separator_row(line: &str) -> bool {
    line.chars()
        .all(|ch| ch == '|' || ch == '-' || ch.is_whitespace())
}

fn normalize_target_file_name(target: &str) -> String {
    let without_anchor = target.split('#').next().unwrap_or(target);
    let without_query = without_anchor.split('?').next().unwrap_or(without_anchor);
    Path::new(without_query)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(without_query)
        .to_string()
}

fn collect_expected_files(
    docs_root: &Path,
    exclude_files: &[String],
) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(docs_root)
        .with_context(|| format!("reading {}", docs_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("md")
        {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if exclude_files.iter().any(|excluded| excluded == file_name) {
            continue;
        }
        files.push(file_name.to_string());
    }
    files.sort();
    Ok(files)
}

fn extract_h1(path: &PathBuf) -> Result<Option<String>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    Ok(contents
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(str::to_string)))
}
