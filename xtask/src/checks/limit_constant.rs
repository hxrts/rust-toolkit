use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_files, line_number_at, normalize_rel_path, preceding_lines},
};

const LIMIT_KEYWORDS: &[&str] = &[
    "max",
    "min",
    "limit",
    "capacity",
    "batch",
    "retry",
    "backoff",
    "timeout",
    "ttl",
    "inflight",
    "concurrency",
];

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.limit_constant else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let binding_re = Regex::new(
        r"(?m)^\s*let\s+(?:mut\s+)?(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*(?::[^=]+)?=\s*(?P<value>\d[\d_]*)\s*;",
    )?;
    let mut findings = FlatFindingSet::default();
    for path in collect_rust_files(repo_root, &check.include_paths)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for captures in binding_re.captures_iter(&source) {
            let Some(name) = captures.name("name") else {
                continue;
            };
            let lower = name.as_str().to_ascii_lowercase();
            if !LIMIT_KEYWORDS.iter().any(|needle| lower.contains(needle)) {
                continue;
            }
            let context_ok = preceding_lines(&source, name.start(), 2)
                .iter()
                .any(|line| line.contains(&check.allow_comment_marker));
            if context_ok {
                continue;
            }
            findings.entries.insert(format!(
                "{rel}:{}: hard limit `{}` should be a named constant, not a local numeric binding",
                line_number_at(&source, name.start()),
                name.as_str()
            ));
        }
    }
    Ok(findings)
}
