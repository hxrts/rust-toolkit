use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{config::ToolkitConfig, report::FlatFindingSet};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.text_formatting else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let mut findings = FlatFindingSet::default();
    for rel in &check.include_paths {
        let path = repo_root.join(rel);
        if path.is_file() {
            scan_file(repo_root, &path, &check.exclude_path_parts, &mut findings)?;
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
            scan_file(
                repo_root,
                entry.path(),
                &check.exclude_path_parts,
                &mut findings,
            )?;
        }
    }
    Ok(findings)
}

fn scan_file(
    repo_root: &Path,
    path: &Path,
    exclude_path_parts: &[String],
    findings: &mut FlatFindingSet,
) -> Result<()> {
    let rel = normalize_rel_path(repo_root, path);
    if exclude_path_parts
        .iter()
        .any(|part| !part.is_empty() && rel.contains(part))
    {
        return Ok(());
    }
    let contents = match fs::read_to_string(path) {
        | Ok(contents) => contents,
        | Err(err) if err.kind() == std::io::ErrorKind::InvalidData => return Ok(()),
        | Err(err) => {
            return Err(err).with_context(|| format!("reading {}", path.display()))
        },
    };
    for (line_no, line) in contents.lines().enumerate() {
        if contains_forbidden_emoji(line) {
            findings
                .entries
                .insert(format!("{rel}:{}: forbidden emoji detected", line_no + 1));
        }
    }
    Ok(())
}

fn contains_forbidden_emoji(line: &str) -> bool {
    line.contains('✅')
        || line.contains('❌')
        || line.contains("⚠️")
        || line
            .chars()
            .any(|ch| matches!(ch as u32, 0x1F300..=0x1FAFF))
}

fn normalize_rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::contains_forbidden_emoji;

    #[test]
    fn text_formatting_flags_emoji_but_not_symbols() {
        assert!(contains_forbidden_emoji("bad ✅"));
        assert!(contains_forbidden_emoji("bad 😀"));
        assert!(!contains_forbidden_emoji("good ✓"));
        assert!(!contains_forbidden_emoji("good ⚠"));
    }
}
