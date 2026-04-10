use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_files, line_number_at, normalize_rel_path},
};

// long-block-exception: the scan keeps field and function surface checks
// together
pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.public_type_width else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let field_re =
        Regex::new(r"(?m)^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?P<ty>[^,]+),")?;
    let fn_re = Regex::new(
        r"(?s)pub(?:\([^)]*\))?\s+fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\((?P<params>[^)]*)\)\s*(?:->\s*(?P<ret>[^;{]+))?",
    )?;
    let mut findings = FlatFindingSet::default();
    for path in collect_rust_files(repo_root, &check.include_paths)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for captures in field_re.captures_iter(&source) {
            let Some(ty) = captures.name("ty") else {
                continue;
            };
            for banned in &check.banned_types {
                if has_type_token(ty.as_str(), banned) {
                    findings.entries.insert(format!(
                        "{rel}:{}: public field uses banned public type `{banned}`",
                        line_number_at(&source, ty.start())
                    ));
                }
            }
        }
        for captures in fn_re.captures_iter(&source) {
            let Some(name) = captures.name("name") else {
                continue;
            };
            let params = captures
                .name("params")
                .map(|item| item.as_str())
                .unwrap_or_default();
            for banned in &check.banned_types {
                if has_type_token(params, banned) {
                    findings.entries.insert(format!(
                        "{rel}:{}: public function `{}` exposes banned parameter type `{banned}`",
                        line_number_at(&source, name.start()),
                        name.as_str()
                    ));
                }
                if captures
                    .name("ret")
                    .map(|ret| has_type_token(ret.as_str(), banned))
                    .unwrap_or(false)
                {
                    findings.entries.insert(format!(
                        "{rel}:{}: public function `{}` exposes banned return type `{banned}`",
                        line_number_at(&source, name.start()),
                        name.as_str()
                    ));
                }
            }
        }
    }
    Ok(findings)
}

fn has_type_token(haystack: &str, needle: &str) -> bool {
    haystack
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .any(|token| token == needle)
}
