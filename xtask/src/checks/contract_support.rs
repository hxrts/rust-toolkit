use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::{ScopedPatternContract, ScopedPatternContractsConfig},
    report::FlatFindingSet,
    util::normalize_rel_path,
};

pub fn run_scoped_pattern_contracts(
    repo_root: &Path,
    check: &ScopedPatternContractsConfig,
) -> Result<FlatFindingSet> {
    let mut findings = FlatFindingSet::default();
    for contract in &check.required_patterns {
        if !contract_has_match(repo_root, contract)? {
            findings.entries.insert(format!(
                "missing required pattern `{}` in [{}]",
                contract.pattern,
                contract.include_paths.join(", ")
            ));
        }
    }
    for contract in &check.forbidden_patterns {
        collect_forbidden_matches(repo_root, contract, &mut findings)?;
    }
    Ok(findings)
}

fn contract_has_match(
    repo_root: &Path,
    contract: &ScopedPatternContract,
) -> Result<bool> {
    let regex = Regex::new(&contract.pattern)
        .with_context(|| format!("invalid regex `{}`", contract.pattern))?;
    for file in iter_files(repo_root, &contract.include_paths)? {
        let contents = match fs::read_to_string(&file) {
            | Ok(contents) => contents,
            | Err(err) if err.kind() == std::io::ErrorKind::InvalidData => continue,
            | Err(err) => {
                return Err(err).with_context(|| format!("reading {}", file.display()))
            },
        };
        if regex.is_match(&contents) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn collect_forbidden_matches(
    repo_root: &Path,
    contract: &ScopedPatternContract,
    findings: &mut FlatFindingSet,
) -> Result<()> {
    let regex = Regex::new(&contract.pattern)
        .with_context(|| format!("invalid regex `{}`", contract.pattern))?;
    for file in iter_files(repo_root, &contract.include_paths)? {
        let rel = normalize_rel_path(repo_root, &file);
        let contents = match fs::read_to_string(&file) {
            | Ok(contents) => contents,
            | Err(err) if err.kind() == std::io::ErrorKind::InvalidData => continue,
            | Err(err) => {
                return Err(err).with_context(|| format!("reading {}", file.display()))
            },
        };
        for (line_idx, line) in contents.lines().enumerate() {
            if regex.is_match(line) {
                findings.entries.insert(format!(
                    "{rel}:{}: forbidden pattern `{}`",
                    line_idx + 1,
                    contract.pattern
                ));
            }
        }
    }
    Ok(())
}

fn iter_files(
    repo_root: &Path,
    include_paths: &[String],
) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for rel in include_paths {
        let path = repo_root.join(rel);
        if path.is_file() {
            files.push(path);
            continue;
        }
        if !path.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&path)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            files.push(entry.into_path());
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}
