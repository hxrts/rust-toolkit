use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{config::ToolkitConfig, report::FlatFindingSet, util::normalize_rel_path};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.docs_prose_quality else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let mut findings = FlatFindingSet::default();
    for rel in &check.include_paths {
        let root = repo_root.join(rel);
        if !root.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let rel_path = normalize_rel_path(repo_root, path);
            if check
                .exclude_path_parts
                .iter()
                .any(|part| !part.is_empty() && rel_path.contains(part))
            {
                continue;
            }
            scan_file(&rel_path, path, check, &mut findings)?;
        }
    }
    Ok(findings)
}

fn scan_file(
    rel_path: &str,
    path: &Path,
    check: &crate::config::DocsProseQualityConfig,
    findings: &mut FlatFindingSet,
) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;

    let mut in_code = false;
    let mut pending_explainer = false;
    let mut prose_words = 0usize;
    let mut code_words = 0usize;

    for (index, line) in source.lines().enumerate() {
        let line_no = index + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_code {
                in_code = false;
                pending_explainer = check.require_explanatory_prose_after_code;
            } else {
                in_code = true;
            }
            continue;
        }

        if in_code {
            code_words += count_words(line);
            continue;
        }

        prose_words += count_words(line);

        if check.ban_em_dash && line.contains('—') {
            findings
                .entries
                .insert(format!("{rel_path}:{line_no}: em dash is not allowed"));
        }
        if check.ban_semicolon && line.contains(';') {
            findings
                .entries
                .insert(format!("{rel_path}:{line_no}: semicolon is not allowed"));
        }

        if pending_explainer {
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with('#') || trimmed.starts_with("```") {
                findings.entries.insert(format!(
                    "{rel_path}:{line_no}: code block must be followed by an explanatory paragraph"
                ));
            } else if is_markdown_list(trimmed) {
                findings.entries.insert(format!(
                    "{rel_path}:{line_no}: explanatory text after code block must be prose, not a list"
                ));
            }
            pending_explainer = false;
        }
    }

    if in_code {
        findings
            .entries
            .insert(format!("{rel_path}: unclosed fenced code block"));
    }
    if check.require_prose_exceeds_code && prose_words <= code_words {
        findings.entries.insert(format!(
            "{rel_path}: prose word count ({prose_words}) must exceed code word count ({code_words})"
        ));
    }

    Ok(())
}

fn count_words(line: &str) -> usize {
    line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|token| !token.is_empty())
        .count()
}

fn is_markdown_list(trimmed: &str) -> bool {
    trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || trimmed.chars().take_while(|ch| ch.is_ascii_digit()).count() > 0
            && (trimmed.contains(". ") || trimmed.contains(") "))
}
