use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use pulldown_cmark::{Event, Options, Parser, Tag};
use walkdir::WalkDir;

use crate::{
    config::{DocsLinkCheckConfig, ToolkitConfig},
    report::FlatFindingSet,
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.docs_link_check else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let mut findings = FlatFindingSet::default();
    for docs_root in &check.docs_roots {
        let docs_root_abs = repo_root.join(docs_root);
        if !docs_root_abs.is_dir() {
            continue;
        }
        for file in collect_markdown_files(&docs_root_abs) {
            scan_file(repo_root, &docs_root_abs, &file, check, &mut findings)?;
        }
    }
    Ok(findings)
}

fn collect_markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|ext| ext.to_str()) == Some("md")
        {
            files.push(entry.into_path());
        }
    }
    files.sort();
    files
}

fn scan_file(
    repo_root: &Path,
    docs_root: &Path,
    file: &Path,
    check: &DocsLinkCheckConfig,
    findings: &mut FlatFindingSet,
) -> Result<()> {
    let rel_file = normalize_rel_path(repo_root, file);
    let contents = std::fs::read_to_string(file)
        .with_context(|| format!("reading {}", file.display()))?;
    for event in Parser::new_ext(&contents, Options::empty()) {
        let Event::Start(Tag::Link { dest_url, .. }) = event else {
            continue;
        };
        let target = dest_url.to_string();
        if should_skip_target(&target) {
            continue;
        }
        if target.contains(&check.scratch_dir_prefix) {
            findings.entries.insert(format!(
                "link to private scratch directory: {rel_file} -> {target}"
            ));
        }
        if rel_file.starts_with("docs/")
            && target.starts_with('/')
            && matches!(
                target.as_str(),
                s if s.starts_with("/Users/")
                    || s.starts_with("/home/")
                    || s.starts_with("/tmp/")
                    || s.starts_with("/var/")
                    || s.starts_with("/opt/")
                    || s.starts_with("/root/")
            )
        {
            findings
                .entries
                .insert(format!("absolute path in link: {rel_file} -> {target}"));
        }

        let path_part = target.split('#').next().unwrap_or_default();
        if path_part.is_empty() {
            continue;
        }
        let Some(resolved) = resolve_target(repo_root, file, path_part) else {
            continue;
        };
        if !resolved.starts_with(docs_root) {
            continue;
        }
        if !resolved.is_file() {
            findings.entries.insert(format!(
                "missing docs link: {rel_file} -> {}",
                normalize_rel_path(repo_root, &resolved)
            ));
        }
    }
    Ok(())
}

fn should_skip_target(target: &str) -> bool {
    target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with('#')
}

fn resolve_target(root: &Path, source_file: &Path, target: &str) -> Option<PathBuf> {
    if target.starts_with('/') {
        return Some(PathBuf::from(target));
    }
    let target_path = Path::new(target);
    if target.starts_with("docs/") {
        return Some(root.join(target_path));
    }
    Some(
        source_file
            .parent()
            .unwrap_or(root)
            .join(target_path)
            .components()
            .collect(),
    )
}

fn normalize_rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
