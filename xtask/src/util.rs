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

pub fn collect_rust_policy_files(
    root: &Path,
    include_roots: &[String],
    exclude_path_parts: &[String],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path in collect_rust_files(root, include_roots)? {
        let rel_path = normalize_rel_path(root, &path);
        if rust_policy_path_excluded(&rel_path, exclude_path_parts) {
            continue;
        }
        files.push(path);
    }
    Ok(files)
}

pub fn normalize_rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn rust_policy_path_excluded(rel_path: &str, exclude_path_parts: &[String]) -> bool {
    rel_path.contains("/tests/")
        || rel_path.contains("/benches/")
        || rel_path.contains("/examples/")
        || rel_path.ends_with("/build.rs")
        || rel_path.contains("/target/")
        || exclude_path_parts
            .iter()
            .any(|part| !part.is_empty() && rel_path.contains(part))
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

pub fn mask_rust_comments_and_literals(source: &str) -> String {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum State {
        Code,
        LineComment,
        BlockComment(usize),
        String,
        Char,
        RawString(usize),
    }

    fn push_masked(out: &mut String, byte: u8) {
        if byte == b'\n' {
            out.push('\n');
        } else {
            out.push(' ');
        }
    }

    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut idx = 0usize;
    let mut state = State::Code;
    while idx < bytes.len() {
        match state {
            State::Code => {
                if bytes[idx] == b'/' && bytes.get(idx + 1) == Some(&b'/') {
                    push_masked(&mut out, bytes[idx]);
                    push_masked(&mut out, bytes[idx + 1]);
                    idx += 2;
                    state = State::LineComment;
                    continue;
                }
                if bytes[idx] == b'/' && bytes.get(idx + 1) == Some(&b'*') {
                    push_masked(&mut out, bytes[idx]);
                    push_masked(&mut out, bytes[idx + 1]);
                    idx += 2;
                    state = State::BlockComment(1);
                    continue;
                }
                if bytes[idx] == b'r' {
                    if let Some((hashes, consumed)) = raw_string_prefix(&bytes[idx..]) {
                        for byte in &bytes[idx..idx + consumed] {
                            push_masked(&mut out, *byte);
                        }
                        idx += consumed;
                        state = State::RawString(hashes);
                        continue;
                    }
                }
                if bytes[idx] == b'b' && bytes.get(idx + 1) == Some(&b'r') {
                    if let Some((hashes, consumed)) = raw_string_prefix(&bytes[idx + 1..]) {
                        push_masked(&mut out, bytes[idx]);
                        for byte in &bytes[idx + 1..idx + 1 + consumed] {
                            push_masked(&mut out, *byte);
                        }
                        idx += consumed + 1;
                        state = State::RawString(hashes);
                        continue;
                    }
                }
                if bytes[idx] == b'"' || (bytes[idx] == b'b' && bytes.get(idx + 1) == Some(&b'"')) {
                    push_masked(&mut out, bytes[idx]);
                    idx += 1;
                    if bytes.get(idx - 1) == Some(&b'b') {
                        push_masked(&mut out, bytes[idx]);
                        idx += 1;
                    }
                    state = State::String;
                    continue;
                }
                if bytes[idx] == b'\'' {
                    push_masked(&mut out, bytes[idx]);
                    idx += 1;
                    state = State::Char;
                    continue;
                }
                out.push(bytes[idx] as char);
                idx += 1;
            }
            State::LineComment => {
                push_masked(&mut out, bytes[idx]);
                if bytes[idx] == b'\n' {
                    state = State::Code;
                }
                idx += 1;
            }
            State::BlockComment(depth) => {
                if bytes[idx] == b'/' && bytes.get(idx + 1) == Some(&b'*') {
                    push_masked(&mut out, bytes[idx]);
                    push_masked(&mut out, bytes[idx + 1]);
                    idx += 2;
                    state = State::BlockComment(depth + 1);
                    continue;
                }
                if bytes[idx] == b'*' && bytes.get(idx + 1) == Some(&b'/') {
                    push_masked(&mut out, bytes[idx]);
                    push_masked(&mut out, bytes[idx + 1]);
                    idx += 2;
                    if depth == 1 {
                        state = State::Code;
                    } else {
                        state = State::BlockComment(depth - 1);
                    }
                    continue;
                }
                push_masked(&mut out, bytes[idx]);
                idx += 1;
            }
            State::String => {
                push_masked(&mut out, bytes[idx]);
                if bytes[idx] == b'\\' {
                    idx += 1;
                    if let Some(byte) = bytes.get(idx) {
                        push_masked(&mut out, *byte);
                        idx += 1;
                    }
                    continue;
                }
                if bytes[idx] == b'"' {
                    state = State::Code;
                }
                idx += 1;
            }
            State::Char => {
                push_masked(&mut out, bytes[idx]);
                if bytes[idx] == b'\\' {
                    idx += 1;
                    if let Some(byte) = bytes.get(idx) {
                        push_masked(&mut out, *byte);
                        idx += 1;
                    }
                    continue;
                }
                if bytes[idx] == b'\'' {
                    state = State::Code;
                }
                idx += 1;
            }
            State::RawString(hashes) => {
                push_masked(&mut out, bytes[idx]);
                if bytes[idx] == b'"' && raw_string_terminator_matches(bytes, idx + 1, hashes) {
                    idx += 1;
                    for _ in 0..hashes {
                        if let Some(byte) = bytes.get(idx) {
                            push_masked(&mut out, *byte);
                            idx += 1;
                        }
                    }
                    state = State::Code;
                    continue;
                }
                idx += 1;
            }
        }
    }
    out
}

fn raw_string_prefix(bytes: &[u8]) -> Option<(usize, usize)> {
    if bytes.first().copied() != Some(b'r') {
        return None;
    }
    let mut idx = 1usize;
    while bytes.get(idx) == Some(&b'#') {
        idx += 1;
    }
    (bytes.get(idx) == Some(&b'"')).then_some((idx - 1, idx + 1))
}

fn raw_string_terminator_matches(bytes: &[u8], start: usize, hashes: usize) -> bool {
    bytes
        .get(start..start + hashes)
        .map(|slice| slice.iter().all(|byte| *byte == b'#'))
        .unwrap_or(false)
}
