use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{config::ToolkitConfig, report::FlatFindingSet, util::normalize_rel_path};

const DEP_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.dependency_policy else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let mut findings = FlatFindingSet::default();
    for rel_root in &check.manifest_roots {
        let root = repo_root.join(rel_root);
        if !root.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if !entry.file_type().is_file()
                || entry.path().file_name().and_then(|name| name.to_str())
                    != Some("Cargo.toml")
            {
                continue;
            }
            let rel = normalize_rel_path(repo_root, entry.path());
            let contents = fs::read_to_string(entry.path())
                .with_context(|| format!("reading {}", entry.path().display()))?;
            let value: toml::Value = contents
                .parse()
                .with_context(|| format!("parsing {}", entry.path().display()))?;
            let Some(table) = value.as_table() else {
                continue;
            };
            for dep_table in DEP_TABLES {
                let Some(deps) = table.get(*dep_table).and_then(toml::Value::as_table)
                else {
                    continue;
                };
                for banned in &check.banned_dependencies {
                    if deps.contains_key(banned) {
                        findings.entries.insert(format!(
                            "{rel}: dependency `{banned}` is banned by toolkit dependency policy"
                        ));
                    }
                }
                for dep in &check.require_default_features_false {
                    let Some(value) = deps.get(dep) else {
                        continue;
                    };
                    if !has_default_features_false(value) {
                        findings.entries.insert(format!(
                            "{rel}: dependency `{dep}` must set `default-features = false`"
                        ));
                    }
                }
            }
        }
    }
    Ok(findings)
}

fn has_default_features_false(value: &toml::Value) -> bool {
    match value {
        | toml::Value::String(_) => false,
        | toml::Value::Table(table) => {
            table.get("default-features").and_then(toml::Value::as_bool) == Some(false)
        },
        | _ => false,
    }
}
