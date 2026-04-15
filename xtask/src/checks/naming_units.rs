use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_files, line_number_at, normalize_rel_path},
};

const TIME_NAMES: &[&str] =
    &["timeout", "latency", "backoff", "interval", "duration", "ttl"];
const TIME_SUFFIXES: &[&str] = &[
    "_ns", "_us", "_ms", "_s", "_ticks", "_slots", "_epochs", "_rounds", "_steps",
    "_cycles",
];

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.naming_units else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let binding_re = Regex::new(
        r"(?m)\b(?:let|const|static)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)",
    )?;
    let field_re =
        Regex::new(r"(?m)^\s*(?:pub\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*[^,]+,")?;
    let fn_re = Regex::new(r"(?s)fn\s+[A-Za-z_][A-Za-z0-9_]*\s*\((?P<params>[^)]*)\)")?;
    let mut findings = FlatFindingSet::default();
    for path in collect_rust_files(repo_root, &check.include_paths)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for matched in binding_re.captures_iter(&source) {
            let Some(name) = matched.get(1) else {
                continue;
            };
            add_name_finding(&mut findings, &rel, &source, name.start(), name.as_str());
        }
        for matched in field_re.captures_iter(&source) {
            let Some(name) = matched.get(1) else {
                continue;
            };
            add_name_finding(&mut findings, &rel, &source, name.start(), name.as_str());
        }
        for matched in fn_re.captures_iter(&source) {
            let Some(params) = matched.name("params") else {
                continue;
            };
            for param in params.as_str().split(',') {
                let Some((name, _)) = param.split_once(':') else {
                    continue;
                };
                let name = name.trim().trim_start_matches("mut ").trim();
                if name == "self"
                    || name == "&self"
                    || name == "&mut self"
                    || name.is_empty()
                {
                    continue;
                }
                if let Some(start) = source.find(name) {
                    add_name_finding(&mut findings, &rel, &source, start, name);
                }
            }
        }
    }
    Ok(findings)
}

fn add_name_finding(
    findings: &mut FlatFindingSet,
    rel: &str,
    source: &str,
    start: usize,
    name: &str,
) {
    let lower = name.to_ascii_lowercase();
    if TIME_NAMES.iter().any(|needle| lower.contains(needle)) && !has_time_units(&lower)
    {
        findings.entries.insert(format!(
            "{rel}:{}: `{name}` describes a time quantity but does not include units in the name",
            line_number_at(source, start)
        ));
    }
    if lower == "size" || lower.ends_with("_size") {
        findings.entries.insert(format!(
            "{rel}:{}: `{name}` should use a concrete unit such as `_bytes` instead of bare `size`",
            line_number_at(source, start)
        ));
    }
}

fn has_time_units(name: &str) -> bool {
    TIME_SUFFIXES.iter().any(|suffix| name.ends_with(suffix))
        || TIME_SUFFIXES.iter().any(|suffix| {
            name.ends_with(&format!("{suffix}_max"))
                || name.ends_with(&format!("{suffix}_min"))
        })
}
