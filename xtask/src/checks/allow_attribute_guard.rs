use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{
        collect_rust_policy_files, line_number_at, mask_rust_comments_and_literals,
        normalize_rel_path, preceding_lines,
    },
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
    let mut findings = FlatFindingSet::default();
    for path in collect_rust_policy_files(
        repo_root,
        &check.include_paths,
        &check.exclude_path_parts,
    )? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let masked = mask_rust_comments_and_literals(&source);
        for matched in attr_re.find_iter(&masked) {
            let offset = matched.start();
            let line_no = line_number_at(&source, offset);
            let has_exemption = preceding_lines(&source, offset, 3)
                .iter()
                .any(|line| line.contains(check.allow_comment_marker.as_str()));
            if !has_exemption {
                findings.entries.insert(format!(
                    "{rel}:{line_no}: `#[allow(...)]` requires a preceding `{}` \
                     rationale comment",
                    check.allow_comment_marker
                ));
            }
        }
    }
    Ok(findings)
}
