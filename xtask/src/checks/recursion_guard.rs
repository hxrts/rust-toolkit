use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_files, line_number_at, matching_brace, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.recursion_guard else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let fn_re = Regex::new(
        r"(?s)fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*\)\s*(?:->\s*[^;{]+)?\{",
    )?;
    let mut findings = FlatFindingSet::default();
    for path in collect_rust_files(repo_root, &check.include_paths)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for captures in fn_re.captures_iter(&source) {
            let Some(matched) = captures.get(0) else {
                continue;
            };
            let Some(name) = captures.name("name") else {
                continue;
            };
            let open_idx = matched.end().saturating_sub(1);
            let Some(close_idx) = matching_brace(&source, open_idx) else {
                continue;
            };
            let body = &source[open_idx + 1..close_idx];
            if body.contains(&check.allow_comment_marker) {
                continue;
            }
            let needle = format!("{}(", name.as_str());
            if body.contains(&needle) {
                findings.entries.insert(format!(
                    "{rel}:{}: direct recursion in `{}` requires an explicit exception marker",
                    line_number_at(&source, name.start()),
                    name.as_str()
                ));
            }
        }
    }
    Ok(findings)
}
