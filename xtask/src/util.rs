use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};

#[derive(Clone)]
pub struct ParsedSource {
    pub rel_path: String,
    pub source: String,
}

pub fn collect_rust_files(
    root: &Path,
    include_roots: &[String],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for rel in include_roots {
        let dir = root.join(rel);
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let rel_path = normalize_rel_path(root, path);
            if rel_path.contains("/target/") {
                continue;
            }
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

pub fn normalize_rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn collect_markdown_files(
    root: &Path,
    include_roots: &[String],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for rel in include_roots {
        let dir = root.join(rel);
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                if normalize_rel_path(root, path).starts_with("docs/book/") {
                    continue;
                }
                files.push(path.to_path_buf());
            }
        }
    }
    for rel in ["CLAUDE.md", "README.md"] {
        let path = root.join(rel);
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

pub fn collect_lean_files(
    root: &Path,
    include_roots: &[String],
    exclude_path_parts: &[String],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for rel in include_roots {
        let dir = root.join(rel);
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("lean") {
                continue;
            }
            let rel_path = normalize_rel_path(root, path);
            if rel_path.contains("/.lake/")
                || rel_path.contains("/build/")
                || rel_path.contains("/lake-packages/")
                || exclude_path_parts
                    .iter()
                    .any(|part| !part.is_empty() && rel_path.contains(part))
            {
                continue;
            }
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

pub fn just_recipes(root: &Path) -> Result<BTreeSet<String>> {
    let output = Command::new("just")
        .arg("--summary")
        .current_dir(root)
        .output()
        .context("running just --summary")?;
    if !output.status.success() {
        bail!("toolkit-xtask: just --summary failed");
    }
    let stdout = String::from_utf8(output.stdout).context("just summary utf8")?;
    Ok(stdout
        .split_whitespace()
        .map(std::string::ToString::to_string)
        .collect())
}

pub fn workspace_crate_names(
    root: &Path,
    manifest_path: &Path,
    crate_roots: &[String],
) -> Result<BTreeSet<String>> {
    let workspace_root = manifest_path.parent().unwrap_or(root);
    let mut crate_names = BTreeSet::new();
    collect_package_name(manifest_path, &mut crate_names)?;
    for crate_root in crate_roots {
        let dir = root.join(crate_root);
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.file_name().and_then(|name| name.to_str()) != Some("Cargo.toml")
            {
                continue;
            }
            if normalize_rel_path(workspace_root, path).contains("/target/") {
                continue;
            }
            collect_package_name(path, &mut crate_names)?;
        }
    }
    Ok(crate_names)
}

fn collect_package_name(path: &Path, out: &mut BTreeSet<String>) -> Result<()> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let value: toml::Value = contents
        .parse()
        .with_context(|| format!("parsing {}", path.display()))?;
    let Some(table) = value.as_table() else {
        return Ok(());
    };
    let Some(package) = table.get("package").and_then(toml::Value::as_table) else {
        return Ok(());
    };
    let Some(name) = package.get("name").and_then(toml::Value::as_str) else {
        return Ok(());
    };
    out.insert(name.to_owned());
    out.insert(name.replace('-', "_"));
    Ok(())
}

pub fn parse_workspace_sources(
    root: &Path,
    crate_roots: &[String],
) -> Result<Vec<ParsedSource>> {
    let mut parsed = Vec::new();
    for crate_root in crate_roots {
        let dir = root.join(crate_root);
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir) {
            let entry = entry?;
            let path = entry.path();
            if !entry.file_type().is_file() {
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let rel_path = normalize_rel_path(root, path);
            if rel_path.contains("/tests/")
                || rel_path.contains("/benches/")
                || rel_path.contains("/examples/")
                || rel_path.ends_with("/build.rs")
                || rel_path.contains("/target/")
            {
                continue;
            }
            let source = std::fs::read_to_string(path)
                .with_context(|| format!("reading {}", path.display()))?;
            parsed.push(ParsedSource { rel_path, source });
        }
    }
    parsed.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok(parsed)
}

pub fn all_identifiers(parsed: &[ParsedSource]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for source in parsed {
        for token in source
            .source
            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .filter(|token| !token.is_empty())
        {
            out.insert(token.to_string());
        }
    }
    out
}

pub fn line_number_at(source: &str, byte_index: usize) -> usize {
    source[..byte_index.min(source.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

pub fn matching_brace(source: &str, open_idx: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(open_idx).copied() != Some(b'{') {
        return None;
    }
    let mut depth = 0usize;
    for (idx, byte) in bytes.iter().enumerate().skip(open_idx) {
        match byte {
            | b'{' => depth += 1,
            | b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            },
            | _ => {},
        }
    }
    None
}

pub fn preceding_lines(source: &str, byte_index: usize, count: usize) -> Vec<&str> {
    let prefix = &source[..byte_index.min(source.len())];
    let mut lines: Vec<&str> = prefix.lines().collect();
    let keep_from = lines.len().saturating_sub(count);
    lines.drain(..keep_from);
    lines
}
