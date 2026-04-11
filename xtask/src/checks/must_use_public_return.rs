use std::{fs, path::Path};

use anyhow::{Context, Result};
use syn::{
    visit::Visit, ImplItem, ImplItemFn, ItemFn, ItemImpl, ReturnType, Type, Visibility,
};

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_policy_files, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.must_use_public_return else {
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
        let file = syn::parse_file(&source)
            .with_context(|| format!("parsing {}", path.display()))?;
        let mut visitor = MustUseVisitor {
            rel_path: &rel,
            allowed_return_type_prefixes: &check.allowed_return_type_prefixes,
            findings: &mut findings,
        };
        visitor.visit_file(&file);
    }
    Ok(findings)
}

struct MustUseVisitor<'a> {
    rel_path: &'a str,
    allowed_return_type_prefixes: &'a [String],
    findings: &'a mut FlatFindingSet,
}

impl MustUseVisitor<'_> {
    fn inspect_fn(
        &mut self,
        attrs: &[syn::Attribute],
        vis: &Visibility,
        sig: &syn::Signature,
    ) {
        if !matches!(vis, Visibility::Public(_))
            || attrs_contain_must_use(attrs)
            || attrs_contain_proc_macro_entry(attrs)
            || sig.receiver().is_some()
        {
            return;
        }

        let ReturnType::Type(_, ty) = &sig.output else {
            return;
        };
        if return_type_allowed(ty, self.allowed_return_type_prefixes) {
            return;
        }

        let line = sig.ident.span().start().line;
        self.findings.entries.insert(format!(
            "{}:{line}: public function `{}` returns a meaningful value without #[must_use]",
            self.rel_path, sig.ident
        ));
    }
}

impl<'ast> Visit<'ast> for MustUseVisitor<'_> {
    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        self.inspect_fn(&item_fn.attrs, &item_fn.vis, &item_fn.sig);
    }

    fn visit_item_impl(&mut self, item_impl: &'ast ItemImpl) {
        for item in &item_impl.items {
            if let ImplItem::Fn(method) = item {
                self.visit_impl_item_fn(method);
            }
        }
    }

    fn visit_impl_item_fn(&mut self, method: &'ast ImplItemFn) {
        self.inspect_fn(&method.attrs, &method.vis, &method.sig);
    }
}

fn attrs_contain_must_use(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("must_use"))
}

fn attrs_contain_proc_macro_entry(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        matches!(
            attr.path().segments.last().map(|segment| segment.ident.to_string()),
            Some(name)
                if matches!(
                    name.as_str(),
                    "proc_macro" | "proc_macro_attribute" | "proc_macro_derive"
                )
        )
    })
}

fn return_type_allowed(ty: &Type, allowed_prefixes: &[String]) -> bool {
    let Some(root_ident) = root_type_ident(ty) else {
        return false;
    };
    allowed_prefixes
        .iter()
        .any(|allowed| allowed == &root_ident)
}

fn root_type_ident(ty: &Type) -> Option<String> {
    match ty {
        | Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        | _ => None,
    }
}
