use std::path::Path;

use anyhow::Result;
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_policy_files, scan_with_marker},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.allow_attribute_guard else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }
    // Matches #[allow(...)] and #[expect(...)] (stabilised in Rust 1.81).
    let attr_re = Regex::new(r"#\[\s*(?:allow|expect)\s*\(")?;
    let files =
        collect_rust_policy_files(repo_root, &check.include_paths, &check.exclude_path_parts)?;
    let marker = &check.allow_comment_marker;
    scan_with_marker(files, repo_root, &attr_re, marker, 3, |rel, line_no, _| {
        format!(
            "{rel}:{line_no}: `#[allow(...)]` requires a preceding `{marker}` rationale comment"
        )
    })
}
