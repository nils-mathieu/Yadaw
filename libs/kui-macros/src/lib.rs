#![feature(proc_macro_diagnostic, proc_macro_span)]

use proc_macro::TokenStream;

mod len;

/// Creates a [`kui::elements::Length`] from the given value.
#[proc_macro]
pub fn len(tokens: TokenStream) -> TokenStream {
    self::len::parse_length_literal(tokens)
}
