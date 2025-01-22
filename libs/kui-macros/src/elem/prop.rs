use {
    super::color::parse_color_literal,
    crate::{
        len::parse_length_literal,
        utility::{STANDARD_SUFFIXES, is_decimal_number_literal, is_string_literal},
    },
    proc_macro2::{Ident, Spacing, TokenStream, TokenTree, token_stream::IntoIter},
    quote::quote,
};

/// A prop declaration within an element's body.
pub struct PropDecl {
    /// The identifier of the property.
    pub ident: Ident,
    /// The value that will be assigned.
    pub values: Vec<TokenStream>,
}

impl PropDecl {
    /// Parses the provided token stream into a [`PropDecl`].
    ///
    /// Returns `None` if given an empty token stream.
    pub fn parse(iter: &mut IntoIter) -> Option<Self> {
        let ident = match iter.next() {
            Some(TokenTree::Ident(ident)) => ident,
            Some(tt) => {
                tt.span()
                    .unwrap()
                    .error(format!("Expected a prop name, got {tt}"))
                    .emit();
                Ident::new("_dummy", tt.span())
            }
            None => return None,
        };

        match iter.next() {
            Some(TokenTree::Punct(punct)) => match punct.as_char() {
                ':' if punct.spacing() == Spacing::Alone => (),
                ';' if punct.spacing() == Spacing::Alone => {
                    return Some(Self {
                        ident,
                        values: Vec::new(),
                    });
                }
                _ => {
                    punct
                        .span()
                        .unwrap()
                        .error(format!("Expected a colon, got {punct}"))
                        .emit();
                    return Some(Self {
                        ident,
                        values: Vec::new(),
                    });
                }
            },
            Some(tt) => {
                tt.span()
                    .unwrap()
                    .error(format!("Expected a colon, got {tt}"))
                    .emit();
                return Some(Self {
                    ident,
                    values: Vec::new(),
                });
            }
            None => {
                return Some(Self {
                    ident,
                    values: Vec::new(),
                });
            }
        }

        fn is_char(tt: &TokenTree, c: char) -> bool {
            matches!(tt, TokenTree::Punct(punct) if punct.spacing() == Spacing::Alone && punct.as_char() == c)
        }

        let mut prop_content = iter
            .take_while(|tt| !is_char(tt, ';'))
            .collect::<Vec<TokenTree>>()
            .into_iter();

        let mut values = Vec::new();

        while prop_content.len() > 0 {
            let prop_value: TokenStream = (&mut prop_content)
                .take_while(|tt| !is_char(tt, ','))
                .collect();
            values.push(prop_value);
        }

        Some(Self { ident, values })
    }

    /// Turns the field into a token stream.
    pub fn to_builder_method(&self) -> TokenStream {
        let ident = &self.ident;

        let values = self
            .values
            .iter()
            .map(|value| match PropValueHint::predict(value.clone()) {
                PropValueHint::Length => parse_length_literal(value.clone()),
                PropValueHint::Color => parse_color_literal(value.clone()),
                PropValueHint::Unknown => value.clone(),
            });

        quote! { .#ident ( #(#values),* ) }
    }
}

/// The predicted kind of a field value.
enum PropValueHint {
    /// The field value seems to be a length literal.
    Length,
    /// The field value seems to be a color literal.
    Color,
    /// The field value is unknown.
    Unknown,
}

impl PropValueHint {
    /// Attempts to predict the type of the provided tokens.
    pub fn predict(tokens: TokenStream) -> Self {
        let mut iter = tokens.into_iter();

        match iter.next() {
            Some(TokenTree::Literal(lit)) => {
                let lit = lit.to_string();
                if let Some((_, suffix)) = is_decimal_number_literal(&lit) {
                    if suffix.is_empty()
                        || STANDARD_SUFFIXES.contains(&suffix)
                        || iter.next().is_some()
                    {
                        return Self::Unknown;
                    }

                    return Self::Length;
                }

                if let Some(lit) = is_string_literal(&lit) {
                    if lit.starts_with("#") {
                        if iter.next().is_some() {
                            return Self::Unknown;
                        }

                        return Self::Color;
                    }
                }

                Self::Unknown
            }
            _ => Self::Unknown,
        }
    }
}
