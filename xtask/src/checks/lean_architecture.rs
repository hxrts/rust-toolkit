use std::{fs, path::Path};

use anyhow::{Context, Result};
use regex::Regex;

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_lean_files, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.lean_architecture else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let placeholder_re = Regex::new(r"\bProp\s*:=\s*True\b")?;
    let root_import_re = Regex::new(
        r"^import .*\b(MutualTest|LocalTypeDBExamples|Examples|Tests)\b",
    )?;
    let legacy_projection_re = Regex::new(
        r"^import\s+Choreography\.Projection\.(Trans|Projectb|ProjectProps|Embed|EmbedProps|Erasure|Regression)\b",
    )?;
    let theorempack_re = Regex::new(r"^import\s+Runtime\.Proofs\.TheoremPack$")?;

    let mut findings = FlatFindingSet::default();
    for path in collect_lean_files(repo_root, &check.include_paths, &check.exclude_path_parts)? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        for (index, line) in source.lines().enumerate() {
            let line_no = index + 1;
            if placeholder_re.is_match(line) {
                findings.entries.insert(format!(
                    "{rel}:{line_no}: placeholder contract `Prop := True` is forbidden in production Lean modules"
                ));
            }
            if check.root_facade_files.iter().any(|file| file == &rel)
                && root_import_re.is_match(line)
            {
                findings.entries.insert(format!(
                    "{rel}:{line_no}: root facade imports debug/example/test modules"
                ));
            }
            if legacy_projection_re.is_match(line)
                && !check
                    .legacy_projection_exempt_path_parts
                    .iter()
                    .any(|part| !part.is_empty() && rel.contains(part))
            {
                findings.entries.insert(format!(
                    "{rel}:{line_no}: deprecated legacy projection import remains in a production module"
                ));
            }
            if theorempack_re.is_match(line)
                && !check
                    .theorempack_exempt_path_parts
                    .iter()
                    .any(|part| !part.is_empty() && rel.contains(part))
            {
                findings.entries.insert(format!(
                    "{rel}:{line_no}: direct Runtime.Proofs.TheoremPack import is forbidden outside migration/example shims"
                ));
            }
        }
    }

    Ok(findings)
}
