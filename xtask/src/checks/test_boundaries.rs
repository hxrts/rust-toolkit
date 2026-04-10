use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;
use walkdir::WalkDir;

use crate::{
    config::{TestBoundariesConfig, ToolkitConfig},
    report::FlatFindingSet,
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
    for scan_root in &check.scan_roots {
        let full = repo_root.join(scan_root);
        if !full.exists() {
            continue;
        }
        for entry in WalkDir::new(&full).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.extension().and_then(|ext| ext.to_str()) != Some("rs")
            {
                continue;
            }
            let rel = path
                .strip_prefix(repo_root)
                .with_context(|| {
                    format!("computing relative path for {}", path.display())
                })?
                .to_string_lossy()
                .replace('\\', "/");
            if is_excluded(&rel, check) {
                continue;
            }

            if standalone_tests.is_match(&rel) {
                findings.entries.insert(format!(
                    "{rel}:1: standalone unit-test source files under src/ are forbidden; colocate unit tests in the owning file"
                ));
            }

            let contents = fs::read_to_string(path)
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
    }
    Ok(findings)
}

fn is_excluded(rel: &str, check: &TestBoundariesConfig) -> bool {
    check
        .exclude_prefixes
        .iter()
        .any(|prefix| rel.starts_with(prefix))
        || check
            .exclude_path_parts
            .iter()
            .any(|needle| rel.contains(needle))
}
