use {
    crate::{
        len::parse_length_literal,
        utility::{STANDARD_SUFFIXES, is_decimal_number_literal, is_string_literal},
    },
    proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree},
};

fn quote_color_path(span: Span) -> TokenStream {
    [
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("kui", span)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("peniko", span)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("Color", span)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
    ]
    .into_iter()
    .collect()
}

/// Creates a new color.
fn quote_color_rgb(r: u8, g: u8, b: u8, span: Span) -> TokenStream {
    quote_color_path(span)
        .into_iter()
        .chain([
            TokenTree::Ident(Ident::new("from_rgb8", span)),
            TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                [
                    TokenTree::Literal(Literal::u8_suffixed(r)),
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                    TokenTree::Literal(Literal::u8_suffixed(g)),
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                    TokenTree::Literal(Literal::u8_suffixed(b)),
                ]
                .into_iter()
                .collect(),
            )),
        ])
        .collect()
}

/// Creates a new color.
fn quote_color_rgba(r: u8, g: u8, b: u8, a: u8, span: Span) -> TokenStream {
    quote_color_path(span)
        .into_iter()
        .chain([
            TokenTree::Ident(Ident::new("from_rgb8", span)),
            TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                [
                    TokenTree::Literal(Literal::u8_suffixed(r)),
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                    TokenTree::Literal(Literal::u8_suffixed(g)),
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                    TokenTree::Literal(Literal::u8_suffixed(b)),
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                    TokenTree::Literal(Literal::u8_suffixed(a)),
                ]
                .into_iter()
                .collect(),
            )),
        ])
        .collect()
}

/// Quotes the transparent color.
fn quote_color_constant(name: &str, span: Span) -> TokenStream {
    quote_color_path(span)
        .into_iter()
        .chain(Some(TokenTree::Ident(Ident::new(name, span))))
        .collect()
}

/// Parses the provided string literal as a color.
fn parse_color_literal(tokens: TokenStream) -> TokenStream {
    let mut iter = tokens.into_iter();

    let tt = match iter.next() {
        Some(TokenTree::Literal(lit)) => lit,
        Some(tt) => {
            tt.span()
                .error(format!("Expected a string literal, got `{tt}`"))
                .emit();
            return quote_color_constant("TRANSPARENT", tt.span());
        }
        None => {
            Span::call_site().error("Expected a string literal").emit();
            return quote_color_constant("TRANSPARENT", Span::call_site());
        }
    };

    let lit = tt.to_string();
    let Some(lit) = is_string_literal(&lit) else {
        tt.span().error("Expected a string literal").emit();
        return quote_color_constant("TRANSPARENT", tt.span());
    };

    if !lit.starts_with("#") {
        tt.span()
            .error("Expected a color literal")
            .help("Color literals start with '#'")
            .emit();
        return quote_color_constant("TRANSPARENT", tt.span());
    }

    let lit = &lit[1..];

    fn hex_digit(a: u8, span: Span) -> u8 {
        match a {
            b'0'..=b'9' => a - b'0',
            b'a'..=b'f' => a - b'a' + 10,
            b'A'..=b'F' => a - b'A' + 10,
            _ => {
                span.error("Expected a hexadecimal digit").emit();
                0
            }
        }
    }

    fn parse_hex_digit_2(a: u8, b: u8, a_span: Span, b_span: Span) -> u8 {
        (hex_digit(a, a_span) << 4) | hex_digit(b, b_span)
    }

    let spanat = |s: usize| tt.subspan(2 + s..2 + s + 1).unwrap();

    match *lit.as_bytes() {
        [r1, r2, g1, g2, b1, b2] => {
            let r = parse_hex_digit_2(r1, r2, spanat(0), spanat(1));
            let g = parse_hex_digit_2(g1, g2, spanat(2), spanat(3));
            let b = parse_hex_digit_2(b1, b2, spanat(4), spanat(5));
            quote_color_rgb(r, g, b, tt.span())
        }
        [r1, g1, b1] => {
            let r = hex_digit(r1, spanat(0));
            let g = hex_digit(g1, spanat(1));
            let b = hex_digit(b1, spanat(2));
            quote_color_rgb(r | (r << 4), g | (g << 4), b | (b << 4), tt.span())
        }
        [r1, r2, g1, g2, b1, b2, a1, a2] => {
            let r = parse_hex_digit_2(r1, r2, spanat(0), spanat(1));
            let g = parse_hex_digit_2(g1, g2, spanat(2), spanat(3));
            let b = parse_hex_digit_2(b1, b2, spanat(4), spanat(5));
            let a = parse_hex_digit_2(a1, a2, spanat(6), spanat(7));
            quote_color_rgba(r, g, b, a, tt.span())
        }
        [r1, g1, b1, a1] => {
            let r = hex_digit(r1, spanat(0));
            let g = hex_digit(g1, spanat(1));
            let b = hex_digit(b1, spanat(2));
            let a = hex_digit(a1, spanat(3));
            quote_color_rgba(
                r | (r << 4),
                g | (g << 4),
                b | (b << 4),
                a | (a << 4),
                tt.span(),
            )
        }
        _ => {
            tt.span()
                .error("Expected a color literal")
                .help("Color literals are either 3 or 6 hexadecimal digits")
                .emit();
            quote_color_constant("TRANSPARENT", tt.span())
        }
    }
}

/// Returns whether the provided token tree is the field separator character.
fn is_field_separator(tt: &TokenTree) -> bool {
    match tt {
        TokenTree::Punct(punct) => punct.as_char() == ',',
        _ => false,
    }
}

/// The predicted kind of a field value.
enum FieldValueKind {
    /// The field value seems to be a length literal.
    Length,
    /// The field value seems to be a color literal.
    Color,
    /// The field value is unknown.
    Unknown,
}

impl FieldValueKind {
    /// Attempts to predict the type of the provided tokens.
    pub fn predicts(tokens: TokenStream) -> Self {
        let mut iter = tokens.into_iter();

        match iter.next() {
            Some(TokenTree::Literal(lit)) => {
                let lit = lit.to_string();
                if let Some((_, suffix)) = is_decimal_number_literal(&lit) {
                    if STANDARD_SUFFIXES.contains(&suffix) {
                        return Self::Unknown;
                    }
                    if iter.next().is_some() {
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

/// Represents an element field declaration.
struct ElementField {
    /// The identifier of the field.
    ident: Ident,
    /// The value of the field.
    value: TokenStream,
}

impl ElementField {
    /// Turns the field into a token stream.
    pub fn to_tokens(&self) -> TokenStream {
        // If the value looks like a `Length`, use that. Otherwise just forward it.
        let val = match FieldValueKind::predicts(self.value.clone()) {
            FieldValueKind::Length => parse_length_literal(self.value.clone()),
            FieldValueKind::Color => parse_color_literal(self.value.clone()),
            FieldValueKind::Unknown => self.value.clone(),
        };

        [
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(self.ident.clone()),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, val)),
        ]
        .into_iter()
        .collect()
    }
}

/// Represents an element.
struct Element {
    /// The identifier of the element.
    ident: Ident,
    /// The fields of the element.
    fields: Vec<ElementField>,
    /// The children of the element.
    children: Vec<Element>,
}

impl Element {
    /// Parses the provided token stream into an element.
    pub fn parse_content(ident: Ident, content: TokenStream) -> Self {
        let mut content = content.into_iter();

        let mut fields = Vec::new();
        let mut children = Vec::new();

        let mut next = content.next();
        loop {
            let ident = match next {
                Some(TokenTree::Ident(ident)) => ident,
                Some(tt) => {
                    tt.span()
                        .error(format!("Expected an identifier, got `{tt}`"))
                        .emit();
                    Ident::new("dummy_identifier", tt.span())
                }
                None => break,
            };

            match content.next() {
                Some(TokenTree::Group(group)) => {
                    if group.delimiter() == Delimiter::Brace {
                        children.push(Self::parse_content(ident, group.stream()));
                    } else {
                        group.span().error("Expected a `:` or `{`").emit();
                    }
                    next = content.next();
                }
                Some(TokenTree::Punct(punct)) => match punct.as_char() {
                    ':' => {
                        fields.push(ElementField {
                            ident,
                            value: (&mut content)
                                .take_while(|tt| !is_field_separator(tt))
                                .collect(),
                        });
                        next = content.next();
                    }
                    ',' => {
                        fields.push(ElementField {
                            ident,
                            value: TokenStream::new(),
                        });
                        next = content.next();
                    }
                    _ => {
                        punct
                            .span()
                            .error(format!("Expected a `:` or `{{`, got `{punct}`"))
                            .emit();
                        next = Some(punct.into());
                    }
                },
                Some(tt) => {
                    tt.span()
                        .error(format!(
                            "Expected a group or a field definition, got `{tt}`"
                        ))
                        .emit();

                    // Assume field for better auto-completion.
                    println!("Produced dummy field: {ident}");
                    fields.push(ElementField {
                        ident,
                        value: TokenStream::new(),
                    });

                    next = Some(tt)
                }
                None => {
                    Span::call_site()
                        .error("Expected an element or a field deinition")
                        .emit();

                    // Assume field for better auto-completion.
                    println!("Produced dummy field: {ident}");
                    fields.push(ElementField {
                        ident,
                        value: TokenStream::new(),
                    });

                    next = None;
                }
            }
        }

        Self {
            ident,
            fields,
            children,
        }
    }

    /// Parses the provided token stream into an element recursively.
    pub fn parse(tokens: TokenStream) -> Self {
        let mut tokens = tokens.into_iter();

        let ident = match tokens.next() {
            Some(TokenTree::Ident(ident)) => ident,
            Some(tt) => {
                tt.span()
                    .error(format!("Expected an identifier, got `{tt}`"))
                    .emit();
                Ident::new("dummy_identifier", tt.span())
            }
            None => {
                Span::call_site().error("Expected an identifier").emit();
                Ident::new("dummy_identifier", Span::call_site())
            }
        };

        let group = match tokens.next() {
            Some(TokenTree::Group(group)) => group,
            Some(tt) => {
                tt.span()
                    .error(format!("Expected a group, got `{tt}`"))
                    .emit();
                Group::new(Delimiter::Brace, TokenStream::new())
            }
            None => {
                Span::call_site().error("Expected a group").emit();
                Group::new(Delimiter::Brace, TokenStream::new())
            }
        };

        if group.delimiter() != Delimiter::Brace {
            group
                .span()
                .error("Expected a group with brackets '{ ... }'")
                .emit();
        }

        Self::parse_content(ident, group.stream())
    }

    /// Turns the element into a token stream as a child.
    pub fn to_tokens_as_child(&self) -> TokenStream {
        [
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("child", self.ident.span())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, self.to_tokens())),
        ]
        .into_iter()
        .collect()
    }

    /// Turns the element into a token stream.
    pub fn to_tokens(&self) -> TokenStream {
        [
            TokenTree::Ident(self.ident.clone()),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
        ]
        .into_iter()
        .chain(self.fields.iter().flat_map(ElementField::to_tokens))
        .chain(self.children.iter().flat_map(Self::to_tokens_as_child))
        .collect()
    }
}

/// Parses an element tree.
pub fn parse_element_tree(tokens: TokenStream) -> TokenStream {
    Element::parse(tokens).to_tokens()
}
