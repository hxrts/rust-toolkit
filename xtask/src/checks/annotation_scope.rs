use std::{fs, path::Path};

use anyhow::{Context, Result};
use syn::{Attribute, Item};

use crate::{
    config::{AnnotationScopeRule, ToolkitConfig},
    report::FlatFindingSet,
    util::{collect_rust_policy_files, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.annotation_scope else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled || check.rules.is_empty() {
        return Ok(FlatFindingSet::default());
    }
    let mut findings = FlatFindingSet::default();
    for path in collect_rust_policy_files(
        repo_root,
        &check.include_paths,
        &check.exclude_path_parts,
    )? {
        let rel = normalize_rel_path(repo_root, &path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let file = match syn::parse_file(&source) {
            | Ok(f) => f,
            | Err(_) => continue,
        };
        check_items(&file.items, &rel, &check.rules, &mut findings);
    }
    Ok(findings)
}

fn check_items(
    items: &[Item],
    rel: &str,
    rules: &[AnnotationScopeRule],
    findings: &mut FlatFindingSet,
) {
    for item in items {
        let Some((attrs, kind, name, line)) = item_info(item) else {
            continue;
        };
        for rule in rules {
            let annotation = parse_annotation(&rule.annotation);
            if !attrs.iter().any(|a| attr_matches(a, annotation)) {
                continue;
            }
            if !rule.allowed_paths.is_empty()
                && !rule.allowed_paths.iter().any(|p| rel.starts_with(p))
            {
                findings.entries.insert(format!(
                    "{rel}:{line}: `{annotation}` on `{name}` is outside \
                     allowed paths for this annotation"
                ));
            }
            if rule.forbidden_paths.iter().any(|p| rel.starts_with(p)) {
                findings.entries.insert(format!(
                    "{rel}:{line}: `{annotation}` on `{name}` is in a \
                     forbidden path for this annotation"
                ));
            }
            if !rule.allowed_item_kinds.is_empty()
                && !rule.allowed_item_kinds.iter().any(|k| k == kind)
            {
                findings.entries.insert(format!(
                    "{rel}:{line}: `{annotation}` is not allowed on {kind} \
                     items (allowed: {})",
                    rule.allowed_item_kinds.join(", ")
                ));
            }
        }
        // Recurse into modules and impl blocks.
        if let Item::Mod(m) = item {
            if let Some((_, ref items)) = m.content {
                check_items(items, rel, rules, findings);
            }
        }
        if let Item::Impl(imp) = item {
            for impl_item in &imp.items {
                let Some((attrs, kind, _name, line)) = impl_item_info(impl_item) else {
                    continue;
                };
                for rule in rules {
                    let annotation = parse_annotation(&rule.annotation);
                    if !attrs.iter().any(|a| attr_matches(a, annotation)) {
                        continue;
                    }
                    if !rule.allowed_item_kinds.is_empty()
                        && !rule.allowed_item_kinds.iter().any(|k| k == kind)
                    {
                        findings.entries.insert(format!(
                            "{rel}:{line}: `{annotation}` is not allowed on \
                             {kind} items (allowed: {})",
                            rule.allowed_item_kinds.join(", ")
                        ));
                    }
                }
            }
        }
    }
}

fn item_info(item: &Item) -> Option<(&[Attribute], &str, String, usize)> {
    match item {
        | Item::Trait(t) => Some((
            &t.attrs,
            "trait",
            t.ident.to_string(),
            t.ident.span().start().line,
        )),
        | Item::Struct(s) => Some((
            &s.attrs,
            "struct",
            s.ident.to_string(),
            s.ident.span().start().line,
        )),
        | Item::Enum(e) => Some((
            &e.attrs,
            "enum",
            e.ident.to_string(),
            e.ident.span().start().line,
        )),
        | Item::Fn(f) => Some((
            &f.attrs,
            "fn",
            f.sig.ident.to_string(),
            f.sig.ident.span().start().line,
        )),
        | Item::Impl(i) => {
            let line = i.impl_token.span.start().line;
            let name = type_name(&i.self_ty);
            Some((&i.attrs, "impl", name, line))
        },
        | Item::Type(t) => Some((
            &t.attrs,
            "type",
            t.ident.to_string(),
            t.ident.span().start().line,
        )),
        | Item::Const(c) => Some((
            &c.attrs,
            "const",
            c.ident.to_string(),
            c.ident.span().start().line,
        )),
        | Item::Static(s) => Some((
            &s.attrs,
            "static",
            s.ident.to_string(),
            s.ident.span().start().line,
        )),
        | Item::Mod(m) => Some((
            &m.attrs,
            "mod",
            m.ident.to_string(),
            m.ident.span().start().line,
        )),
        | _ => None,
    }
}

fn impl_item_info(item: &syn::ImplItem) -> Option<(&[Attribute], &str, String, usize)> {
    match item {
        | syn::ImplItem::Fn(f) => Some((
            &f.attrs,
            "fn",
            f.sig.ident.to_string(),
            f.sig.ident.span().start().line,
        )),
        | syn::ImplItem::Const(c) => Some((
            &c.attrs,
            "const",
            c.ident.to_string(),
            c.ident.span().start().line,
        )),
        | syn::ImplItem::Type(t) => Some((
            &t.attrs,
            "type",
            t.ident.to_string(),
            t.ident.span().start().line,
        )),
        | _ => None,
    }
}

fn type_name(ty: &syn::Type) -> String {
    match ty {
        | syn::Type::Path(p) => p
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        | _ => "<anonymous>".to_string(),
    }
}

/// Strip `#[` prefix and `]` suffix from the raw annotation string.
fn parse_annotation(raw: &str) -> &str {
    let s = raw.trim();
    let s = s.strip_prefix("#[").unwrap_or(s);
    let s = s.strip_suffix(']').unwrap_or(s);
    s.split('(').next().unwrap_or(s).trim()
}

fn attr_matches(attr: &Attribute, name: &str) -> bool {
    let path = attr.path();
    if name.contains("::") {
        let segments: Vec<_> =
            path.segments.iter().map(|s| s.ident.to_string()).collect();
        segments.join("::") == name
    } else {
        path.is_ident(name)
    }
}
