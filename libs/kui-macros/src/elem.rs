use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

/// Returns whether the provided token tree is the field separator character.
fn is_field_separator(tt: &TokenTree) -> bool {
    match tt {
        TokenTree::Punct(punct) => punct.as_char() == ',',
        _ => false,
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
        [
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(self.ident.clone()),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, self.value.clone())),
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
