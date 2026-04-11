use std::{collections::BTreeMap, fs, path::Path, process::Command};

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::{config::ToolkitConfig, report::FlatFindingSet, util::normalize_rel_path};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.workspace_layering else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let manifest_path = repo_root.join(&check.manifest_path);
    let status = Command::new("cargo")
        .args([
            "metadata",
            "--manifest-path",
            &manifest_path.to_string_lossy(),
            "--no-deps",
            "--format-version",
            "1",
        ])
        .current_dir(repo_root)
        .output()
        .context("running cargo metadata for workspace layering")?;
    if !status.status.success() {
        bail!(
            "workspace layering: cargo metadata failed: {}",
            String::from_utf8_lossy(&status.stderr).trim()
        );
    }

    let metadata: Value = serde_json::from_slice(&status.stdout)
        .context("parsing cargo metadata json")?;
    let workspace_members = metadata
        .get("workspace_members")
        .and_then(Value::as_array)
        .context("workspace_members missing from cargo metadata")?;
    let workspace_member_ids: Vec<&str> = workspace_members
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let packages = metadata
        .get("packages")
        .and_then(Value::as_array)
        .context("packages missing from cargo metadata")?;

    let mut member_packages = Vec::new();
    for package in packages {
        let Some(id) = package.get("id").and_then(Value::as_str) else {
            continue;
        };
        if workspace_member_ids.iter().any(|member| *member == id) {
            member_packages.push(package);
        }
    }

    let mut findings = FlatFindingSet::default();
    let mut package_names = BTreeMap::new();
    for package in &member_packages {
        let Some(name) = package.get("name").and_then(Value::as_str) else {
            continue;
        };
        let rel_manifest = package
            .get("manifest_path")
            .and_then(Value::as_str)
            .and_then(|path| path.strip_prefix(&format!("{}/", repo_root.display())))
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| normalize_rel_path(repo_root, &manifest_path));
        package_names.insert(name.to_owned(), rel_manifest);
        if !check.crate_layers.contains_key(name) {
            findings.entries.insert(format!(
                "{rel_manifest}: workspace crate `{name}` is missing from `checks.workspace_layering.crate_layers`"
            ));
        }
    }

    for package in member_packages {
        let Some(name) = package.get("name").and_then(Value::as_str) else {
            continue;
        };
        let Some(&pkg_layer) = check.crate_layers.get(name) else {
            continue;
        };
        let rel_manifest = package_names
            .get(name)
            .cloned()
            .unwrap_or_else(|| normalize_rel_path(repo_root, &manifest_path));
        let dependencies = package
            .get("dependencies")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for dependency in dependencies {
            if dependency.get("source").is_some_and(|value| !value.is_null()) {
                continue;
            }
            if dependency
                .get("kind")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind != "normal")
            {
                continue;
            }
            let Some(dep_name) = dependency.get("name").and_then(Value::as_str) else {
                continue;
            };
            let Some(&dep_layer) = check.crate_layers.get(dep_name) else {
                findings.entries.insert(format!(
                    "{rel_manifest}: local dependency `{dep_name}` is missing from `checks.workspace_layering.crate_layers`"
                ));
                continue;
            };
            if dep_layer > pkg_layer {
                findings.entries.insert(format!(
                    "{rel_manifest}: workspace crate `{name}` (layer {pkg_layer}) depends on higher-layer crate `{dep_name}` (layer {dep_layer})"
                ));
            }
        }
    }

    Ok(findings)
}
