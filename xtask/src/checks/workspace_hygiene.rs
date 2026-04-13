use std::path::Path;

use anyhow::Result;

use crate::{config::ToolkitConfig, report::FlatFindingSet, util::normalize_rel_path};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.workspace_hygiene else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let mut findings = FlatFindingSet::default();
    for rel in &check.include_paths {
        let root = repo_root.join(rel);
        if !root.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            let rel_path = normalize_rel_path(repo_root, path);
            if should_exclude(&rel_path, &check.exclude_path_parts) {
                continue;
            }
            if entry.file_type().is_file()
                && path.file_name().and_then(|name| name.to_str()) == Some("mod.rs")
                && is_lonely_mod_rs(path)
            {
                findings.entries.insert(format!(
                    "{rel_path}: lonely mod.rs file should be collapsed into a sibling Rust file"
                ));
            }
            if entry.file_type().is_dir()
                && is_empty_directory(path, &check.exclude_path_parts)
            {
                findings.entries.insert(format!(
                    "{rel_path}: empty directory should be removed or populated"
                ));
            }
        }
    }
    Ok(findings)
}

fn should_exclude(rel_path: &str, exclude_path_parts: &[String]) -> bool {
    rel_path.contains("/target/")
        || rel_path.contains("/.git/")
        || exclude_path_parts
            .iter()
            .any(|part| !part.is_empty() && rel_path.contains(part))
}

fn is_lonely_mod_rs(path: &Path) -> bool {
    let Some(dir) = path.parent() else {
        return false;
    };
    let mut sibling_rs = 0usize;
    let mut subdirs = 0usize;
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in read_dir.filter_map(std::result::Result::ok) {
        let entry_path = entry.path();
        if entry_path == path {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.is_dir() {
            subdirs += 1;
            continue;
        }
        if metadata.is_file()
            && entry_path.extension().and_then(|ext| ext.to_str()) == Some("rs")
        {
            sibling_rs += 1;
        }
    }
    sibling_rs == 0 && subdirs == 0
}

fn is_empty_directory(path: &Path, exclude_path_parts: &[String]) -> bool {
    let Ok(read_dir) = std::fs::read_dir(path) else {
        return false;
    };
    for entry in read_dir.filter_map(std::result::Result::ok) {
        let rel = entry.path().to_string_lossy().replace('\\', "/");
        if should_exclude(&rel, exclude_path_parts) {
            continue;
        }
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{is_empty_directory, is_lonely_mod_rs};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("toolkit-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn workspace_hygiene_detects_lonely_mod_rs() {
        let root = temp_dir("lonely-mod");
        let dir = root.join("crate/src/only");
        fs::create_dir_all(&dir).expect("create nested dir");
        let mod_rs = dir.join("mod.rs");
        fs::write(&mod_rs, "pub fn helper() {}\n").expect("write mod");
        assert!(is_lonely_mod_rs(&mod_rs));
    }

    #[test]
    fn workspace_hygiene_detects_empty_directory() {
        let root = temp_dir("empty-dir");
        let dir = root.join("crate/src/empty");
        fs::create_dir_all(&dir).expect("create nested dir");
        assert!(is_empty_directory(&dir, &[]));
    }
}
