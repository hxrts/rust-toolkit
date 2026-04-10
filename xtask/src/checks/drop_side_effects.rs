use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_files, line_number_at, matching_brace, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.drop_side_effects else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let impl_re = Regex::new(r"(?s)impl\s+Drop\s+for\s+[A-Za-z_][A-Za-z0-9_]*\s*\{")?;
    let drop_re = Regex::new(r"(?s)fn\s+drop\s*\([^)]*\)\s*\{")?;
    let mut findings = FlatFindingSet::default();
    for path in collect_rust_files(repo_root, &check.include_paths)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for impl_match in impl_re.find_iter(&source) {
            let open_idx = impl_match.end().saturating_sub(1);
            let Some(close_idx) = matching_brace(&source, open_idx) else {
                continue;
            };
            let impl_body = &source[open_idx + 1..close_idx];
            let Some(drop_match) = drop_re.find(impl_body) else {
                continue;
            };
            let drop_open = open_idx + 1 + drop_match.end() - 1;
            let Some(drop_close) = matching_brace(&source, drop_open) else {
                continue;
            };
            let drop_body = &source[drop_open + 1..drop_close];
            let meaningful = drop_body.lines().map(str::trim).any(|line| {
                !line.is_empty()
                    && !line.starts_with("//")
                    && !line.starts_with("/*")
                    && !line.contains(&check.allow_comment_marker)
            });
            if meaningful && !drop_body.contains(&check.allow_comment_marker) {
                findings.entries.insert(format!(
                    "{rel}:{}: nontrivial Drop implementation hides side effects; move work into an explicit method or add a narrow exception marker",
                    line_number_at(&source, drop_open)
                ));
            }
        }
    }
    Ok(findings)
}
