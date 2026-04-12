use std::path::Path;

use anyhow::Result;

use crate::{
    checks::formal_claim_scope::scan_file, config::ToolkitConfig,
    report::FlatFindingSet,
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.parity_ledger else {
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
