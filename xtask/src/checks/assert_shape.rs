use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_files, line_number_at, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.assert_shape else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let assert_re = Regex::new(r"(?:debug_)?assert!\s*\([^;\n]*(?:&&|\|\|)[^;\n]*\)")?;
    let mut findings = FlatFindingSet::default();
    for path in collect_rust_files(repo_root, &check.include_paths)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for matched in assert_re.find_iter(&source) {
            findings.entries.insert(format!(
                "{rel}:{}: split compound assert conditions into separate assertions",
                line_number_at(&source, matched.start())
            ));
        }
    }
    Ok(findings)
}
