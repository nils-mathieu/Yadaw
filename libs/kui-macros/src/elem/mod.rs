use {
    self::decl::Decl,
    proc_macro2::{Delimiter, TokenStream, TokenTree, token_stream::IntoIter},
    quote::quote,
};

mod color;
mod decl;
mod prop;

/// Represents an element.
struct Element {
    /// The identifier of the element.
    path: TokenStream,
    /// The declarations within the element's body.
    decls: Vec<Decl>,
}

impl Element {
    /// Parses the provided token stream into an element.
    ///
    /// Returns `None` if given an empty token stream.
    pub fn parse(tokens: &mut IntoIter) -> Option<Self> {
        let mut error_in_path = false;

        let mut path = TokenStream::new();
        let group = loop {
            match tokens.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    break group;
                }
                Some(TokenTree::Ident(i)) => path.extend(Some(TokenTree::Ident(i))),
                Some(TokenTree::Punct(p)) if p.as_char() == ':' => {
                    path.extend(Some(TokenTree::Punct(p)))
                }
                Some(tt) => {
                    if !error_in_path {
                        tt.span()
                            .unwrap()
                            .error(format!("Expected a path to an element, got {tt}"))
                            .emit();
                        error_in_path = true;
                    }
                    path.extend(Some(tt));
                }
                None => return None,
            }
        };

        let mut body = group.stream().into_iter();
        let decls = std::iter::from_fn(|| Decl::parse(&mut body)).collect();

        Some(Self { path, decls })
    }

    /// Turns the element into a token stream as a child.
    pub fn to_tokens_as_child(&self) -> TokenStream {
        let element = self.to_tokens();

        quote! {
            .child(
                #element
            )
        }
    }

    /// Turns the element into a token stream.
    pub fn to_tokens(&self) -> TokenStream {
        let path = &self.path;
        let decls = self.decls.iter().map(Decl::to_builder_method);

        quote! {
            ::kui::IntoElement::into_element(
                #path ()
                    #(#decls)*
            )
        }
    }
}

/// Parses an element tree.
pub fn parse_element_tree(tokens: TokenStream) -> TokenStream {
    match Element::parse(&mut tokens.into_iter()) {
        Some(e) => e.to_tokens(),
        None => quote! { () },
    }
}
