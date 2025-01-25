use {
    super::{Element, prop::PropDecl},
    proc_macro2::{Delimiter, TokenStream, TokenTree, token_stream::IntoIter},
};

/// A declaration in an element's body.
pub enum Decl {
    Prop(PropDecl),
    Child(Element),
}

impl Decl {
    /// Parses the provided token stream into a declaration.
    pub fn parse(tokens: &mut IntoIter) -> Option<Self> {
        match DeclKind::predict(tokens.clone()) {
            DeclKind::Prop => PropDecl::parse(tokens).map(Self::Prop),
            DeclKind::Child => Element::parse(tokens).map(Self::Child),
        }
    }

    /// Turns the declaration into a token stream as a builder method.
    pub fn to_builder_method(&self) -> TokenStream {
        match self {
            Self::Prop(prop) => prop.to_builder_method(),
            Self::Child(child) => child.to_tokens_as_child(),
        }
    }
}

impl From<PropDecl> for Decl {
    fn from(field: PropDecl) -> Self {
        Self::Prop(field)
    }
}

impl From<Element> for Decl {
    fn from(child: Element) -> Self {
        Self::Child(child)
    }
}

/// The kind of a declaration.
enum DeclKind {
    Prop,
    Child,
}

impl DeclKind {
    /// Looks ahead in the provided iterator and predicts whether the next declaration is a field or
    /// a child.
    pub fn predict(mut iter: IntoIter) -> Self {
        match iter.next() {
            Some(TokenTree::Ident(_)) => (),
            _ => return Self::Prop,
        };

        match iter.next() {
            Some(TokenTree::Punct(punct)) => match punct.as_char() {
                ':' => {
                    if punct.spacing() == proc_macro2::Spacing::Alone {
                        Self::Prop
                    } else {
                        Self::Child
                    }
                }
                _ => Self::Prop,
            },
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => Self::Child,
            _ => Self::Prop,
        }
    }
}
