use std::path::PathBuf;

use rustc_hir::Item;
use rustc_span::{source_map::SourceMap, Span};

pub(crate) fn source_file_path(source_map: &SourceMap, item: &Item<'_>) -> PathBuf {
    PathBuf::from(format!(
        "{}",
        source_map
            .lookup_source_file(item.span.lo())
            .name
            .prefer_remapped_unconditionally()
    ))
}

pub(crate) fn source_file_contents(
    source_map: &SourceMap,
    item: &Item<'_>,
) -> Option<(PathBuf, String)> {
    let path = source_file_path(source_map, item);
    let contents = std::fs::read_to_string(&path).ok()?;
    Some((path, contents))
}

pub(crate) fn source_file_path_for_span(source_map: &SourceMap, span: Span) -> PathBuf {
    PathBuf::from(format!(
        "{}",
        source_map
            .lookup_source_file(span.lo())
            .name
            .prefer_remapped_unconditionally()
    ))
}
