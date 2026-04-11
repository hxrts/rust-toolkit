use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result};
use syn::{
    spanned::Spanned,
    visit::Visit,
    Field, Fields, ImplItem, ImplItemFn, ItemEnum, ItemFn, ItemImpl, ItemStruct, ReturnType, Type,
    Visibility,
};

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_policy_files, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.public_type_width else {
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
        let mut visitor = PublicTypeWidthVisitor {
            rel_path: &rel,
            banned_types: check.banned_types.iter().cloned().collect(),
            findings: &mut findings,
        };
        visitor.visit_file(&file);
    }
    Ok(findings)
}

struct PublicTypeWidthVisitor<'a> {
    rel_path: &'a str,
    banned_types: BTreeSet<String>,
    findings: &'a mut FlatFindingSet,
}

impl PublicTypeWidthVisitor<'_> {
    fn inspect_public_fn(&mut self, vis: &Visibility, sig: &syn::Signature) {
        if !matches!(vis, Visibility::Public(_)) {
            return;
        }

        for input in &sig.inputs {
            let syn::FnArg::Typed(pat_type) = input else {
                continue;
            };
            for banned in banned_types_in_type(&pat_type.ty, &self.banned_types) {
                let line = pat_type.ty.span().start().line;
                self.findings.entries.insert(format!(
                    "{}:{line}: public function `{}` exposes banned parameter type `{banned}`",
                    self.rel_path, sig.ident
                ));
            }
        }

        let ReturnType::Type(_, ty) = &sig.output else {
            return;
        };
        for banned in banned_types_in_type(ty, &self.banned_types) {
            let line = ty.span().start().line;
            self.findings.entries.insert(format!(
                "{}:{line}: public function `{}` exposes banned return type `{banned}`",
                self.rel_path, sig.ident
            ));
        }
    }

    fn inspect_public_field(&mut self, field: &Field) {
        if !matches!(field.vis, Visibility::Public(_)) {
            return;
        }
        for banned in banned_types_in_type(&field.ty, &self.banned_types) {
            let line = field.ty.span().start().line;
            self.findings.entries.insert(format!(
                "{}:{line}: public field uses banned public type `{banned}`",
                self.rel_path
            ));
        }
    }

    fn inspect_public_enum_fields(&mut self, fields: &Fields) {
        for field in fields {
            for banned in banned_types_in_type(&field.ty, &self.banned_types) {
                let line = field.ty.span().start().line;
                self.findings.entries.insert(format!(
                    "{}:{line}: public enum field uses banned public type `{banned}`",
                    self.rel_path
                ));
            }
        }
    }
}

impl<'ast> Visit<'ast> for PublicTypeWidthVisitor<'_> {
    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        self.inspect_public_fn(&item_fn.vis, &item_fn.sig);
    }

    fn visit_item_impl(&mut self, item_impl: &'ast ItemImpl) {
        for item in &item_impl.items {
            if let ImplItem::Fn(method) = item {
                self.visit_impl_item_fn(method);
            }
        }
    }

    fn visit_impl_item_fn(&mut self, method: &'ast ImplItemFn) {
        self.inspect_public_fn(&method.vis, &method.sig);
    }

    fn visit_item_struct(&mut self, item_struct: &'ast ItemStruct) {
        if !matches!(item_struct.vis, Visibility::Public(_)) {
            return;
        }
        match &item_struct.fields {
            Fields::Named(fields) => {
                for field in &fields.named {
                    self.inspect_public_field(field);
                }
            }
            Fields::Unnamed(fields) => {
                for field in &fields.unnamed {
                    self.inspect_public_field(field);
                }
            }
            Fields::Unit => {}
        }
    }

    fn visit_item_enum(&mut self, item_enum: &'ast ItemEnum) {
        if !matches!(item_enum.vis, Visibility::Public(_)) {
            return;
        }
        for variant in &item_enum.variants {
            self.inspect_public_enum_fields(&variant.fields);
        }
    }
}

fn banned_types_in_type(ty: &Type, banned_types: &BTreeSet<String>) -> Vec<String> {
    let mut out = BTreeSet::new();
    collect_banned_types(ty, banned_types, &mut out);
    out.into_iter().collect()
}

fn collect_banned_types(
    ty: &Type,
    banned_types: &BTreeSet<String>,
    out: &mut BTreeSet<String>,
) {
    match ty {
        Type::Array(array) => collect_banned_types(&array.elem, banned_types, out),
        Type::BareFn(bare_fn) => {
            for input in &bare_fn.inputs {
                collect_banned_types(&input.ty, banned_types, out);
            }
            if let ReturnType::Type(_, ty) = &bare_fn.output {
                collect_banned_types(ty, banned_types, out);
            }
        }
        Type::Group(group) => collect_banned_types(&group.elem, banned_types, out),
        Type::ImplTrait(impl_trait) => {
            for bound in &impl_trait.bounds {
                if let syn::TypeParamBound::Trait(bound) = bound {
                    if let Some(segment) = bound.path.segments.last() {
                        let ident = segment.ident.to_string();
                        if banned_types.contains(&ident) {
                            out.insert(ident);
                        }
                    }
                }
            }
        }
        Type::Paren(paren) => collect_banned_types(&paren.elem, banned_types, out),
        Type::Path(type_path) => {
            for segment in &type_path.path.segments {
                let ident = segment.ident.to_string();
                if banned_types.contains(&ident) {
                    out.insert(ident.clone());
                }
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(ty) = arg {
                            collect_banned_types(ty, banned_types, out);
                        }
                    }
                }
            }
        }
        Type::Ptr(ptr) => collect_banned_types(&ptr.elem, banned_types, out),
        Type::Reference(reference) => collect_banned_types(&reference.elem, banned_types, out),
        Type::Slice(slice) => collect_banned_types(&slice.elem, banned_types, out),
        Type::Tuple(tuple) => {
            for elem in &tuple.elems {
                collect_banned_types(elem, banned_types, out);
            }
        }
        _ => {}
    }
}
