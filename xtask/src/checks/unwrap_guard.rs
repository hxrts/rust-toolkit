use std::path::Path;

use anyhow::Result;
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_policy_files, scan_with_marker},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.unwrap_guard else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }
    let banned_re =
        Regex::new(r"\.(unwrap|expect)\s*\(|(?:^|[^a-zA-Z_])panic\s*!\s*\(")?;
    let files = collect_rust_policy_files(
        repo_root,
        &check.include_paths,
        &check.exclude_path_parts,
    )?;
    let marker = &check.allow_comment_marker;
    scan_with_marker(
        files,
        repo_root,
        &banned_re,
        marker,
        3,
        |rel, line_no, m| {
            let call = m
                .trim_start_matches('.')
                .split('(')
                .next()
                .unwrap_or(m)
                .trim();
            format!("{rel}:{line_no}: `{call}` requires a preceding `{marker}` rationale comment")
        },
    )
}
