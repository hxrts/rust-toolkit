use std::collections::BTreeSet;

use anyhow::{bail, Context, Result};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{
        all_identifiers, collect_markdown_files, just_recipes, normalize_rel_path,
        parse_workspace_sources, workspace_crate_names,
    },
};

const SKIP_IDENTIFIERS: &[&str] = &[
    "String",
    "Vec",
    "Option",
    "Result",
    "Box",
    "Arc",
    "Rc",
    "Mutex",
    "HashMap",
    "HashSet",
    "BTreeMap",
    "BTreeSet",
    "PathBuf",
    "Path",
    "Ok",
    "Err",
    "Some",
    "None",
    "Self",
    "Sized",
    "Send",
    "Sync",
    "Clone",
    "Copy",
    "Debug",
    "Display",
    "Default",
    "Drop",
    "Eq",
    "Ord",
    "Hash",
    "Iterator",
    "Future",
    "Pin",
    "From",
    "Into",
    "AsRef",
    "Deref",
    "PartialEq",
    "PartialOrd",
    "Serialize",
    "Deserialize",
    "Error",
    "Read",
    "Write",
    "PhantomData",
    "Infallible",
    "README",
    "SUMMARY",
    "TODO",
    "FIXME",
    "NOTE",
    "WARNING",
    "IMPORTANT",
    "API",
    "CLI",
    "CI",
    "CD",
    "PR",
    "OS",
    "IO",
    "UUID",
    "HTTP",
    "HTTPS",
    "URL",
    "JSON",
    "CBOR",
    "TOML",
    "YAML",
    "WASM",
    "BFT",
    "CRDT",
    "BLE",
    "GPS",
    "GATT",
    "QUIC",
    "MTU",
    "Alice",
    "Bob",
    "Client",
    "Server",
    "Worker",
    "Coordinator",
    "Done",
    "Active",
    "Closed",
    "Faulted",
    "Admitted",
    "Blocked",
    "Failure",
    "Full",
    "Ack",
    "Commit",
    "Abort",
    "Cancel",
    "Retry",
    "Ping",
    "Pong",
];

const EXTERNAL_PREFIXES: &[&str] = &[
    "std",
    "core",
    "alloc",
    "serde",
    "serde_json",
    "tokio",
    "futures",
    "uuid",
    "blake3",
    "thiserror",
    "tracing",
    "proc_macro2",
    "telltale",
];

pub fn run(
    repo_root: &std::path::Path,
    config: &ToolkitConfig,
) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.docs_semantic_drift else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
        return Ok(FlatFindingSet::default());
    }

    let parsed = parse_workspace_sources(repo_root, &config.workspace.crate_roots)?;
    let mut identifiers = all_identifiers(&parsed);
    identifiers.extend(SKIP_IDENTIFIERS.iter().map(|item| item.to_string()));
    let mut crate_tokens = workspace_crate_names(
        repo_root,
        &repo_root.join(&check.manifest_path),
        &config.workspace.crate_roots,
    )?;
    crate_tokens.extend(check.planned_crates.iter().cloned());
    crate_tokens.extend(
        check
            .planned_crates
            .iter()
            .map(|item| item.replace('-', "_")),
    );
    let just_recipes = just_recipes(repo_root)?;
    let exempt_files: BTreeSet<&str> =
        check.file_exemptions.iter().map(String::as_str).collect();
    let env = SnippetEnv {
        identifiers: &identifiers,
        crate_tokens: &crate_tokens,
        just_recipes: &just_recipes,
    };
    let mut findings = FlatFindingSet::default();

    for file in collect_markdown_files(repo_root, &check.docs_roots)? {
        let rel_file = normalize_rel_path(repo_root, &file);
        if exempt_files.contains(rel_file.as_str()) {
            continue;
        }
        let contents = std::fs::read_to_string(&file)
            .with_context(|| format!("reading {}", file.display()))?;
        let mut in_code_block = false;
        let parser = Parser::new_ext(&contents, Options::empty());
        for event in parser {
            match event {
                | Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_)))
                | Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                    in_code_block = true
                },
                | Event::End(TagEnd::CodeBlock) => in_code_block = false,
                | Event::Code(snippet) if !in_code_block => {
                    check_snippet(repo_root, &rel_file, &snippet, &env, &mut findings);
                },
                | _ => {},
            }
        }
    }

    Ok(findings)
}

struct SnippetEnv<'a> {
    identifiers: &'a BTreeSet<String>,
    crate_tokens: &'a BTreeSet<String>,
    just_recipes: &'a BTreeSet<String>,
}

// long-block-exception: ordered snippet classification is clearer as one
// decision chain
#[allow(clippy::too_many_lines)]
fn check_snippet(
    repo_root: &std::path::Path,
    file: &str,
    snippet: &str,
    env: &SnippetEnv<'_>,
    findings: &mut FlatFindingSet,
) {
    let snippet = snippet.trim();
    if snippet.is_empty() {
        return;
    }
    if should_skip_snippet(snippet) {
        return;
    }
    if snippet.contains('-') && !snippet.contains('/') && !snippet.contains("::") {
        return;
    }
    if let Some(recipe) = snippet.strip_prefix("just ") {
        let recipe = recipe.split_whitespace().next().unwrap_or_default();
        if !recipe.is_empty() && !env.just_recipes.contains(recipe) {
            findings
                .entries
                .insert(format!("{file}: unknown just recipe `{snippet}`"));
        }
        return;
    }
    if looks_like_path(snippet) {
        if !repo_root.join(snippet).exists() {
            findings
                .entries
                .insert(format!("{file}: unresolved path `{snippet}`"));
        }
        return;
    }
    if env.crate_tokens.contains(snippet) {
        return;
    }
    if snippet.contains("::") {
        let segments: Vec<&str> = snippet.split("::").collect();
        let prefix = segments.first().copied().unwrap_or_default();
        if EXTERNAL_PREFIXES.contains(&prefix) {
            return;
        }
        let known_segment = segments
            .iter()
            .filter_map(|segment| root_identifier(segment))
            .any(|segment| {
                env.identifiers.contains(segment)
                    || env.crate_tokens.contains(segment)
                    || SKIP_IDENTIFIERS.contains(&segment)
            });
        if !known_segment {
            findings
                .entries
                .insert(format!("{file}: unresolved qualified symbol `{snippet}`"));
        }
        return;
    }
    if looks_like_simple_symbol(snippet) {
        let Some(root_ident) = root_identifier(snippet) else {
            return;
        };
        if root_ident
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
            && !env.identifiers.contains(root_ident)
            && !env.crate_tokens.contains(root_ident)
            && !SKIP_IDENTIFIERS.contains(&root_ident)
        {
            findings
                .entries
                .insert(format!("{file}: unresolved symbol `{snippet}`"));
        }
    }
}

fn looks_like_path(snippet: &str) -> bool {
    matches!(snippet, "AGENTS.md" | "CLAUDE.md" | "Cargo.toml" | "justfile")
        || ["docs/", "crates/", "scripts/", "lints/", "nix/", ".github/", "toolkit/"]
            .iter()
            .any(|prefix| snippet.starts_with(prefix))
        || snippet.ends_with(".md")
        || snippet.ends_with(".rs")
        || snippet.ends_with(".sh")
        || snippet.ends_with(".toml")
        || snippet.ends_with(".lean")
        || snippet.ends_with(".qnt")
        || snippet.ends_with(".json")
}

fn should_skip_snippet(snippet: &str) -> bool {
    snippet.contains('*')
        || snippet.contains('\\')
        || snippet.contains('{')
        || snippet.contains('}')
        || snippet.contains('(')
        || snippet.contains(')')
        || snippet.contains('[')
        || snippet.contains(']')
        || snippet.contains('=')
        || snippet.contains('+')
        || snippet.contains(' ')
}

fn looks_like_simple_symbol(snippet: &str) -> bool {
    !snippet.contains("::")
        && snippet
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn root_identifier(snippet: &str) -> Option<&str> {
    let mut start = None;
    for (idx, ch) in snippet.char_indices() {
        if start.is_none() {
            if ch.is_ascii_alphabetic() || ch == '_' {
                start = Some(idx);
            }
            continue;
        }
        if !(ch.is_ascii_alphanumeric() || ch == '_') {
            let start = start?;
            return Some(&snippet[start..idx]);
        }
    }
    start.map(|start| &snippet[start..])
}

pub fn run_or_fail(repo_root: &std::path::Path, config: &ToolkitConfig) -> Result<()> {
    let findings = run(repo_root, config)?;
    if findings.is_empty() {
        println!("docs-semantic-drift: no stale backtick references found");
        return Ok(());
    }
    for entry in &findings.entries {
        eprintln!("{entry}");
    }
    bail!("docs-semantic-drift failed")
}

#[cfg(test)]
mod tests {
    use super::{looks_like_path, looks_like_simple_symbol, should_skip_snippet};

    #[test]
    fn semantic_drift_skips_math_and_wildcards() {
        assert!(should_skip_snippet("C_{t+1} = C_t"));
        assert!(should_skip_snippet("docs/1xx_*.md"));
        assert!(!should_skip_snippet("AuthorityId"));
    }

    #[test]
    fn semantic_drift_recognizes_common_paths() {
        assert!(looks_like_path("AGENTS.md"));
        assert!(looks_like_path("docs/000_project_overview.md"));
        assert!(!looks_like_path("AuthorityId"));
    }

    #[test]
    fn semantic_drift_only_treats_identifier_like_snippets_as_symbols() {
        assert!(looks_like_simple_symbol("AuthorityId"));
        assert!(!looks_like_simple_symbol("Specified only"));
        assert!(!looks_like_simple_symbol("DefaultBudget(n)"));
    }
}
