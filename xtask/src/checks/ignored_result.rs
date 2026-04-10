use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_files, line_number_at, normalize_rel_path, preceding_lines},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.ignored_result else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let ignore_re = Regex::new(r"(?m)let\s+_\s*=\s*[^;]+;")?;
    let mut findings = FlatFindingSet::default();
    for path in collect_rust_files(repo_root, &check.include_paths)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for matched in ignore_re.find_iter(&source) {
            let line = source[matched.start()..].lines().next().unwrap_or_default();
            let context_ok = line_has_marker(line, &check.allowed_comment_markers)
                || preceding_lines(&source, matched.start(), 2).iter().any(
                    |candidate| {
                        line_has_marker(candidate, &check.allowed_comment_markers)
                    },
                );
            if context_ok {
                continue;
            }
            findings.entries.insert(format!(
                "{rel}:{}: ignored result-like value without justification comment",
                line_number_at(&source, matched.start())
            ));
        }
    }
    Ok(findings)
}

fn line_has_marker(line: &str, markers: &[String]) -> bool {
    markers.iter().any(|marker| line.contains(marker))
}
