use std::{fs, path::Path};

use anyhow::{Context, Result};
use syn::{
    visit::Visit, Attribute, ImplItem, ItemEnum, ItemFn, ItemImpl, ItemStruct,
    ItemTrait, TraitItem, Visibility,
};

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_policy_files, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.doc_coverage else {
        return Ok(FlatFindingSet::default());
    };
    if !check.enabled {
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
        let mut visitor = DocVisitor { rel: &rel, findings: &mut findings };
        visitor.visit_file(&file);
    }
    Ok(findings)
}

fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn has_doc(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("doc"))
}

struct DocVisitor<'a> {
    rel: &'a str,
    findings: &'a mut FlatFindingSet,
}

impl DocVisitor<'_> {
    fn flag(&mut self, kind: &str, name: &str, line: usize) {
        self.findings.entries.insert(format!(
            "{}:{line}: public {kind} `{name}` is missing a doc comment",
            self.rel
        ));
    }
}

impl<'ast> Visit<'ast> for DocVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if is_public(&node.vis) && !has_doc(&node.attrs) {
            let line = node.sig.ident.span().start().line;
            self.flag("fn", &node.sig.ident.to_string(), line);
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if is_public(&node.vis) && !has_doc(&node.attrs) {
            let line = node.ident.span().start().line;
            self.flag("struct", &node.ident.to_string(), line);
        }
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        if is_public(&node.vis) && !has_doc(&node.attrs) {
            let line = node.ident.span().start().line;
            self.flag("enum", &node.ident.to_string(), line);
        }
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        for item in &node.items {
            if let ImplItem::Fn(method) = item {
                if is_public(&method.vis) && !has_doc(&method.attrs) {
                    let line = method.sig.ident.span().start().line;
                    self.flag("fn", &method.sig.ident.to_string(), line);
                }
            }
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        if is_public(&node.vis) && !has_doc(&node.attrs) {
            let line = node.ident.span().start().line;
            self.flag("trait", &node.ident.to_string(), line);
        }
        for item in &node.items {
            if let TraitItem::Fn(method) = item {
                if !has_doc(&method.attrs) {
                    let line = method.sig.ident.span().start().line;
                    self.flag("fn", &method.sig.ident.to_string(), line);
                }
            }
        }
        syn::visit::visit_item_trait(self, node);
    }
}
