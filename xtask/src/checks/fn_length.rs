use std::{fs, path::Path};

use anyhow::{Context, Result};
use syn::{visit::Visit, ImplItem, ItemFn, ItemImpl};

use crate::{
    config::ToolkitConfig,
    report::FlatFindingSet,
    util::{collect_rust_policy_files, normalize_rel_path},
};

pub fn run(repo_root: &Path, config: &ToolkitConfig) -> Result<FlatFindingSet> {
    let Some(check) = &config.checks.fn_length else {
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
        let mut visitor = FnLengthVisitor {
            source: &source,
            rel: &rel,
            warn_lines: check.warn_lines,
            hard_lines: check.hard_lines,
            marker: &check.allow_comment_marker,
            findings: &mut findings,
        };
        visitor.visit_file(&file);
    }
    Ok(findings)
}

struct FnLengthVisitor<'a> {
    source: &'a str,
    rel: &'a str,
    warn_lines: usize,
    hard_lines: usize,
    marker: &'a str,
    findings: &'a mut FlatFindingSet,
}

impl FnLengthVisitor<'_> {
    fn check_fn(&mut self, name: &str, start_line: usize, end_line: usize) {
        let len = end_line.saturating_sub(start_line) + 1;
        if len <= self.warn_lines {
            return;
        }
        let has_exemption = crate::util::preceding_lines(
            self.source,
            line_start_offset(self.source, start_line),
            3,
        )
        .iter()
        .any(|line| line.contains(self.marker));
        if has_exemption {
            return;
        }
        if len > self.hard_lines {
            self.findings.entries.insert(format!(
                "{}:{start_line}: function `{name}` is {len} lines (hard limit: {})",
                self.rel, self.hard_lines
            ));
        } else {
            self.findings.entries.insert(format!(
                "{}:{start_line}: function `{name}` is {len} lines (warn threshold: {})",
                self.rel, self.warn_lines
            ));
        }
    }
}

impl<'ast> Visit<'ast> for FnLengthVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let start = node.sig.ident.span().start().line;
        let end = node.block.brace_token.span.close().start().line;
        self.check_fn(&node.sig.ident.to_string(), start, end);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        for item in &node.items {
            if let ImplItem::Fn(method) = item {
                let start = method.sig.ident.span().start().line;
                let end = method.block.brace_token.span.close().start().line;
                self.check_fn(&method.sig.ident.to_string(), start, end);
            }
        }
        syn::visit::visit_item_impl(self, node);
    }
}

/// Convert a 1-based line number to a byte offset at the start of that line.
fn line_start_offset(source: &str, line: usize) -> usize {
    let mut current_line = 1usize;
    for (idx, byte) in source.bytes().enumerate() {
        if current_line == line {
            return idx;
        }
        if byte == b'\n' {
            current_line += 1;
        }
    }
    source.len()
}
