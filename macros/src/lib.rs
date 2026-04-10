//! Portable proc macros for trait-surface and effect-boundary contracts.

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_quote, Ident, Path};

fn support_path() -> Path {
    parse_quote!(::rust_toolkit_effects::__private)
}

fn marker_name() -> Ident {
    Ident::new("__toolkit_handler_marker", Span::call_site())
}

#[proc_macro_attribute]
pub fn purity(attr: TokenStream, item: TokenStream) -> TokenStream {
    rust_toolkit_trait_contracts::expand_purity(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn effect_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    rust_toolkit_trait_contracts::expand_effect_trait(
        attr.into(),
        item.into(),
        support_path(),
        marker_name(),
    )
    .into()
}

#[proc_macro_attribute]
pub fn effect_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    rust_toolkit_trait_contracts::expand_effect_handler(
        attr.into(),
        item.into(),
        support_path(),
        marker_name(),
    )
    .into()
}
