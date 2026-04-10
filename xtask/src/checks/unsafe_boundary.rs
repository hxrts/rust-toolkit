use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_files, line_number_at, normalize_rel_path, preceding_lines},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.unsafe_boundary else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let unsafe_re = Regex::new(r"\bunsafe\b")?;
    let mut findings = FlatFindingSet::default();
    for path in collect_rust_files(repo_root, &check.include_paths)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for matched in unsafe_re.find_iter(&source) {
            let line = source[matched.start()..].lines().next().unwrap_or_default();
            if line.contains("unsafe_code") {
                continue;
            }
            let line_no = line_number_at(&source, matched.start());
            if !check.allowed_path_parts.is_empty()
                && !check
                    .allowed_path_parts
                    .iter()
                    .any(|part| rel.contains(part))
            {
                findings.entries.insert(format!(
                    "{rel}:{line_no}: unsafe must be isolated to configured boundary modules"
                ));
            }
            let has_safety_comment = preceding_lines(&source, matched.start(), 3)
                .iter()
                .any(|candidate| {
                    check
                        .required_comment_markers
                        .iter()
                        .any(|marker| candidate.contains(marker))
                });
            if !has_safety_comment {
                findings.entries.insert(format!(
                    "{rel}:{line_no}: unsafe requires a preceding `Safety:` rationale comment"
                ));
            }
        }
    }
    Ok(findings)
}
