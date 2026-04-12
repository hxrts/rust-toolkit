use std::path::Path;

use anyhow::Result;

use crate::{
    checks::contract_support::run_scoped_pattern_contracts, config::ToolkitConfig,
    report::FlatFindingSet,
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.protocol_machine_placeholders else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }
    run_scoped_pattern_contracts(repo_root, check)
}
