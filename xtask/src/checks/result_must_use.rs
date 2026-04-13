use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_files, matching_brace, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.result_must_use else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let mut findings = FlatFindingSet::default();
    for path in collect_rust_files(repo_root, &check.include_paths)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for (trait_name, method_name) in find_missing_must_use(&source)? {
            findings.entries.insert(format!(
                "{rel}:1: trait {trait_name} method {method_name} returns Result without #[must_use]"
            ));
        }
    }
    Ok(findings)
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
            let method_start =
                method.get(0).map(|item| item.start()).unwrap_or_default();
            let window_start = method_start.saturating_sub(4096);
            let prefix_window = &body[window_start..method_start];
            let must_use_macro = prefix_window.contains("must_use_evidence!");
            let return_type = method
                .name("ret")
                .map(|item| item.as_str())
                .unwrap_or_default();
            if attrs.contains("#[must_use")
                || must_use_macro
                || !contains_result(return_type)
            {
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
