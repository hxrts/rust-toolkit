use std::{fs, path::{Path, PathBuf}};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_files, line_number_at, mask_rust_comments_and_literals, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.rust_architecture else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let all_rust_files = collect_rust_files(repo_root, &check.rust_roots)?;
    let fixed_re = Regex::new(r"\bfixed::")?;
    let config_float_re = Regex::new(
        r"(?m)pub\s+[A-Za-z_][A-Za-z0-9_]*\s*:\s*(?:Option<\s*)?f(?:32|64)\b",
    )?;
    let fixed_decode_re =
        Regex::new(r"visit_f(?:32|64)\s*\(|impl\s+From<f(?:32|64)>\s+for\s+FixedQ32")?;
    let float_type_re = Regex::new(r"\b(?:f32|f64)\b")?;
    let nondet_re = Regex::new(
        r"SystemTime::now\(|Instant::now\(|UNIX_EPOCH|rand::thread_rng\(|thread_rng\(|rand::random\(|getrandom\(|from_entropy\(|OsRng\b|Utc::now\(|Local::now\(",
    )?;
    let side_effect_re = Regex::new(
        r"std::fs::|std::net::|std::env::var\(|std::process::Command|tokio::fs::|tokio::net::|tokio::process::Command",
    )?;
    let thread_re = Regex::new(
        r"std::thread::spawn\(|std::thread::sleep\(|tokio::spawn\(|tokio::time::sleep\(",
    )?;

    let mut findings = FlatFindingSet::default();
    for path in &all_rust_files {
        let rel = normalize_rel_path(repo_root, path);
        let source = fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let masked = mask_rust_comments_and_literals(&source);

        if !check.fixed_wrapper_paths.iter().any(|allowed| allowed == &rel) {
            for matched in fixed_re.find_iter(&masked) {
                findings.entries.insert(format!(
                    "{rel}:{}: raw `fixed::` usage is forbidden outside configured wrapper paths",
                    line_number_at(&source, matched.start())
                ));
            }
        }

        if rel.contains("config") || rel.contains("invariants") {
            for matched in config_float_re.find_iter(&masked) {
                findings.entries.insert(format!(
                    "{rel}:{}: float-typed public config/schema field is forbidden",
                    line_number_at(&source, matched.start())
                ));
            }
        }

        if check.fixed_wrapper_paths.iter().any(|allowed| allowed == &rel) {
            for matched in fixed_decode_re.find_iter(&masked) {
                findings.entries.insert(format!(
                    "{rel}:{}: FixedQ32 must not accept float-token decoding or float conversion shims",
                    line_number_at(&source, matched.start())
                ));
            }
        }
    }

    scan_scoped_patterns(
        repo_root,
        &check.determinism_runtime_paths,
        &float_type_re,
        "floating-point types are forbidden in deterministic runtime paths",
        &mut findings,
    )?;
    scan_scoped_patterns(
        repo_root,
        &check.determinism_test_paths,
        &float_type_re,
        "floating-point types are forbidden in deterministic conformance paths",
        &mut findings,
    )?;
    scan_scoped_patterns(
        repo_root,
        &check.determinism_runtime_paths,
        &nondet_re,
        "direct host nondeterminism is forbidden in deterministic runtime paths",
        &mut findings,
    )?;
    scan_scoped_patterns(
        repo_root,
        &check.determinism_test_paths,
        &nondet_re,
        "direct host nondeterminism is forbidden in deterministic conformance paths",
        &mut findings,
    )?;
    scan_scoped_patterns(
        repo_root,
        &check.kernel_paths,
        &side_effect_re,
        "direct host side effects are forbidden in kernel paths",
        &mut findings,
    )?;
    scan_scoped_patterns(
        repo_root,
        &check.determinism_runtime_paths,
        &thread_re,
        "direct thread scheduling and timer calls are forbidden in deterministic runtime paths",
        &mut findings,
    )?;
    scan_scoped_patterns(
        repo_root,
        &check.determinism_test_paths,
        &thread_re,
        "direct thread scheduling and timer calls are forbidden in deterministic conformance paths",
        &mut findings,
    )?;

    Ok(findings)
}

fn scan_scoped_patterns(
    repo_root: &Path,
    rel_paths: &[String],
    pattern: &Regex,
    message: &str,
    findings: &mut FlatFindingSet,
) -> Result<()> {
    for path in collect_scope_files(repo_root, rel_paths) {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let masked = mask_rust_comments_and_literals(&source);
        for matched in pattern.find_iter(&masked) {
            findings.entries.insert(format!(
                "{rel}:{}: {message}",
                line_number_at(&source, matched.start())
            ));
        }
    }
    Ok(())
}

fn collect_scope_files(repo_root: &Path, rel_paths: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for rel in rel_paths {
        let path = repo_root.join(rel);
        if path.is_file() {
            if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
            continue;
        }
        if !path.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&path)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let entry_path = entry.path();
            if entry_path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            files.push(entry_path.to_path_buf());
        }
    }
    files.sort();
    files.dedup();
    files
}
