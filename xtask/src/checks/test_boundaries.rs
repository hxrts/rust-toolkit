use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_policy_files, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.test_boundaries else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let standalone_tests = Regex::new(r"/src/.+/tests\.rs$")?;
    let import_from_tests = Regex::new(
        r#"(#\[\s*path\s*=\s*".*tests/)|(include_(str|bytes)?!\s*\(\s*".*tests/)"#,
    )?;

    let mut findings = FlatFindingSet::default();
    for path in
        collect_rust_policy_files(repo_root, &check.scan_roots, &check.exclude_path_parts)?
    {
        let rel = normalize_rel_path(repo_root, &path);
        if check
            .exclude_prefixes
            .iter()
            .any(|prefix| rel.starts_with(prefix))
        {
            continue;
        }

        if standalone_tests.is_match(&rel) {
            findings.entries.insert(format!(
                "{rel}:1: standalone unit-test source files under src/ are forbidden; \
                 colocate unit tests in the owning file"
            ));
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for (idx, line) in contents.lines().enumerate() {
            if import_from_tests.is_match(line) {
                findings.entries.insert(format!(
                    "{rel}:{}: source-tree unit tests must not import helpers out of tests/",
                    idx + 1
                ));
            }
        }
    }
    Ok(findings)
}
