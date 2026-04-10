use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{config::ToolkitConfig, report::FlatFindingSet, util::normalize_rel_path};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.crate_root_policy else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let mut findings = FlatFindingSet::default();
    for crate_root in &config.workspace.crate_roots {
        let root = repo_root.join(crate_root);
        if !root.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&root)
            .with_context(|| format!("reading crate root {}", root.display()))?
        {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            for candidate in ["src/lib.rs", "src/main.rs"] {
                let path = entry.path().join(candidate);
                if !path.is_file() {
                    continue;
                }
                let rel = normalize_rel_path(repo_root, &path);
                let source = fs::read_to_string(&path)
                    .with_context(|| format!("reading {}", path.display()))?;
                for required in &check.required_attributes {
                    if !source.contains(required) {
                        findings.entries.insert(format!(
                            "{rel}:1: missing crate-root policy attribute `{required}`"
                        ));
                    }
                }
            }
        }
    }
    Ok(findings)
}
