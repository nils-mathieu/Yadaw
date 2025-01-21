#![feature(proc_macro_diagnostic, proc_macro_span)]

use proc_macro::TokenStream;

mod elem;
mod len;
mod utility;

/// Creates a [`kui::elements::Length`] from the given value.
#[proc_macro]
pub fn len(tokens: TokenStream) -> TokenStream {
    self::len::parse_length_literal(tokens)
}

/// Creates a tree of elements.
#[proc_macro]
pub fn elem(tokens: TokenStream) -> TokenStream {
    self::elem::parse_element_tree(tokens)
}
