use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{
    config::{FileLiteralContract, ToolkitConfig},
    report::FlatFindingSet,
    util::normalize_rel_path,
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.formal_claim_scope else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let mut findings = FlatFindingSet::default();
    for file in &check.files {
        scan_file(repo_root, file, &mut findings)?;
    }
    Ok(findings)
}

pub(crate) fn scan_file(
    repo_root: &Path,
    contract: &FileLiteralContract,
    findings: &mut FlatFindingSet,
) -> Result<()> {
    let path = repo_root.join(&contract.path);
    let rel = normalize_rel_path(repo_root, &path);
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    for literal in &contract.required_literals {
        if !contents.contains(literal) {
            findings
                .entries
                .insert(format!("{rel}: missing required text `{literal}`"));
        }
    }
    for literal in &contract.forbidden_literals {
        if contents.contains(literal) {
            findings
                .entries
                .insert(format!("{rel}: forbidden text present `{literal}`"));
        }
    }
    Ok(())
}
