use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::{
    config::{ProcMacroScopeConfig, ToolkitConfig},
    report::FindingSet,
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FindingSet> {
    let Some(check) = &config.checks.proc_macro_scope else {
        return Ok(FindingSet::default());
    };
    if !check.enabled {
        return Ok(FindingSet::default());
    }

    let sources = collect_sources(repo_root, config, check)?;
    let source_set: BTreeSet<String> =
        sources.iter().map(|(rel, _)| rel.clone()).collect();
    let exclude_files: BTreeSet<String> = check.exclude_files.iter().cloned().collect();

    let stale = exclude_files
        .iter()
        .filter(|path| !source_set.contains(*path))
        .cloned()
        .collect();

    let missing = sources
        .into_iter()
        .filter(|(rel, source)| {
            !exclude_files.contains(rel) && !contains_marker(source, check)
        })
        .map(|(rel, _)| rel)
        .collect();

    Ok(FindingSet { stale, missing })
}

fn contains_marker(source: &str, check: &ProcMacroScopeConfig) -> bool {
    check
        .required_markers
        .iter()
        .any(|marker| source.contains(marker))
}

fn collect_sources(
    repo_root: &Path,
    config: &ToolkitConfig,
    check: &ProcMacroScopeConfig,
) -> Result<Vec<(String, String)>> {
    let include_paths: BTreeSet<&str> =
        check.include_paths.iter().map(String::as_str).collect();
    let include_crates: BTreeSet<&str> = config
        .workspace
        .include_crates
        .iter()
        .map(String::as_str)
        .collect();
    let exclude_crates: BTreeSet<&str> = config
        .workspace
        .exclude_crates
        .iter()
        .map(String::as_str)
        .collect();

    let mut sources = Vec::new();
    for root in &config.workspace.crate_roots {
        if !include_paths.contains(root.as_str()) {
            continue;
        }
        let root_dir = repo_root.join(root);
        if !root_dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&root_dir)
            .with_context(|| format!("reading crate root {}", root_dir.display()))?
        {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let crate_name = entry.file_name();
            let crate_name = crate_name.to_string_lossy();
            if !include_crates.is_empty()
                && !include_crates.contains(crate_name.as_ref())
            {
                continue;
            }
            if exclude_crates.contains(crate_name.as_ref()) {
                continue;
            }
            let src_dir = entry.path().join("src");
            if !src_dir.is_dir() {
                continue;
            }
            sources.extend(scan_src_dir(repo_root, &src_dir)?);
        }
    }
    sources.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(sources)
}

fn scan_src_dir(repo_root: &Path, src_dir: &Path) -> Result<Vec<(String, String)>> {
    let mut sources = Vec::new();
    for entry in WalkDir::new(src_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("rs")
        {
            continue;
        }
        let rel = relative_path(repo_root, path)?;
        let source = fs::read_to_string(path)
            .with_context(|| format!("reading source {}", path.display()))?;
        sources.push((rel, source));
    }
    Ok(sources)
}

fn relative_path(repo_root: &Path, path: &Path) -> Result<String> {
    let rel = path
        .strip_prefix(repo_root)
        .with_context(|| format!("computing relative path for {}", path.display()))?;
    Ok(path_to_string(rel))
}

fn path_to_string(path: &Path) -> String {
    let rel: PathBuf = path.components().collect();
    rel.to_string_lossy().replace('\\', "/")
}
