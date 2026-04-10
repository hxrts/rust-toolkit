use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;
use walkdir::WalkDir;

use crate::{
    config::{ResultMustUseConfig, ToolkitConfig},
    report::FlatFindingSet,
    util::matching_brace,
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.result_must_use else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let mut findings = FlatFindingSet::default();
    for include_path in &check.include_paths {
        let full = repo_root.join(include_path);
        if !full.exists() {
            continue;
        }
        scan_tree(repo_root, &full, check, &mut findings)?;
    }

    Ok(findings)
}

fn scan_tree(
    repo_root: &Path,
    root: &Path,
    _check: &ResultMustUseConfig,
    findings: &mut FlatFindingSet,
) -> Result<()> {
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs")
        {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(repo_root)
            .with_context(|| {
                format!("computing relative path for {}", entry.path().display())
            })?
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(entry.path())
            .with_context(|| format!("reading {}", entry.path().display()))?;
        for (trait_name, method_name) in find_missing_must_use(&source)? {
            findings.entries.insert(format!(
                "{rel}:1: trait {trait_name} method {method_name} returns Result without #[must_use]"
            ));
        }
    }
    Ok(())
}

fn find_missing_must_use(source: &str) -> Result<Vec<(String, String)>> {
    let trait_re = Regex::new(r"(?s)\btrait\s+([A-Za-z_][A-Za-z0-9_]*)[^{]*\{")?;
    let method_re = Regex::new(
        r#"(?s)(?P<attrs>(?:\s*#\[[^\]]+\]\s*)*)fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\([^;{}]*\)\s*->\s*(?P<ret>[^;{}]+)[;{]"#,
    )?;

    let mut out = Vec::new();
    let mut search_offset = 0usize;
    while let Some(captures) = trait_re.captures(&source[search_offset..]) {
        let Some(matched) = captures.get(0) else {
            break;
        };
        let trait_name = captures
            .get(1)
            .map(|item| item.as_str().to_owned())
            .unwrap_or_default();
        let trait_start = search_offset + matched.start();
        let body_open = search_offset + matched.end() - 1;
        let Some(body_close) = matching_brace(source, body_open) else {
            break;
        };
        let body = &source[body_open + 1..body_close];
        for method in method_re.captures_iter(body) {
            let attrs = method
                .name("attrs")
                .map(|item| item.as_str())
                .unwrap_or_default();
            let return_type = method
                .name("ret")
                .map(|item| item.as_str())
                .unwrap_or_default();
            if attrs.contains("#[must_use") || !contains_result(return_type) {
                continue;
            }
            let Some(method_name) = method.name("name") else {
                continue;
            };
            out.push((trait_name.clone(), method_name.as_str().to_owned()));
        }
        search_offset = body_close.max(trait_start + 1);
    }
    Ok(out)
}

fn contains_result(return_type: &str) -> bool {
    return_type
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .any(|token| token == "Result")
}
