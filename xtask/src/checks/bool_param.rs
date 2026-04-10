use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_files, line_number_at, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.bool_param else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let fn_re = Regex::new(
        r"(?s)pub(?:\([^)]*\))?\s+fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\((?P<params>[^)]*)\)",
    )?;
    let bool_param_re = Regex::new(r"(^|,)\s*[A-Za-z_][A-Za-z0-9_]*\s*:\s*bool\b")?;
    let mut findings = FlatFindingSet::default();
    for path in collect_rust_files(repo_root, &check.include_paths)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for captures in fn_re.captures_iter(&source) {
            let Some(params) = captures.name("params") else {
                continue;
            };
            if !bool_param_re.is_match(params.as_str()) {
                continue;
            }
            let Some(name) = captures.name("name") else {
                continue;
            };
            findings.entries.insert(format!(
                "{rel}:{}: public function `{}` uses a bool parameter; prefer an enum or options struct",
                line_number_at(&source, name.start()),
                name.as_str()
            ));
        }
    }
    Ok(findings)
}
