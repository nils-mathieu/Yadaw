use {
    crate::utility::is_decimal_number_literal,
    proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree},
};

/// A possible suffix for a length literal.
#[derive(Debug)]
pub enum LengthSuffix {
    /// The length is specified in term of unscaled pixels.
    ///
    /// `upx`
    UnscaledPixels,
    /// The length is specified in term of scaled pixels.
    ///
    /// `px`
    Pixels,

    /// The length is specified in term of the parent's width.
    ///
    /// `w%`
    ParentWidth,
    /// The length is specified in term of the parent's height.
    ///
    /// `h%`
    ParentHeight,
}

impl LengthSuffix {
    /// Parses a length suffix from a string.
    pub fn parse(s: &str, span: Span) -> Result<Self, ()> {
        match s {
            "upx" => Ok(Self::UnscaledPixels),
            "px" => Ok(Self::Pixels),
            "w%" => Ok(Self::ParentWidth),
            "h%" => Ok(Self::ParentHeight),
            "%" => {
                span.warning(
                    "`%` unit is an alias for `w%`, it won't automatically handle length direction",
                )
                .emit();
                Ok(Self::ParentWidth)
            }
            _ => {
                span.error(format!("Length unit not recognized: `{s}`"))
                    .help("Available units are `upx`, `px`, `w%`, `h%`")
                    .emit();
                Err(())
            }
        }
    }

    /// The identifier of the variant.
    pub fn identifier(&self) -> &'static str {
        match self {
            Self::UnscaledPixels => "UnscaledPixels",
            Self::Pixels => "Pixels",
            Self::ParentWidth => "ParentWidth",
            Self::ParentHeight => "ParentHeight",
        }
    }

    /// Converts the length suffix into the appropriate literal.
    pub fn literal(&self, val: f64) -> Literal {
        match self {
            Self::UnscaledPixels => Literal::f64_suffixed(val),
            Self::Pixels => Literal::f64_suffixed(val),
            Self::ParentWidth => Literal::f64_suffixed(val / 100.0),
            Self::ParentHeight => Literal::f64_suffixed(val / 100.0),
        }
    }
}

/// Parses the provided string into a `f64`.
fn parse_f64(s: &str, span: Span) -> Result<f64, ()> {
    match s.parse() {
        Ok(ok) => Ok(ok),
        Err(_) => {
            span.error(format!("Failed to parse `{s}` into `f64`"))
                .emit();
            Err(())
        }
    }
}

/// A parsed length literal.
#[derive(Debug)]
pub enum Length {
    /// The `0` literal.
    Zero,
    /// A literal value with a suffix.
    Literal {
        /// The value.
        value: f64,
        /// The suffix associated with the value.
        suffix: LengthSuffix,
    },
}

impl Length {
    /// Parses the provided literal into a length literal.
    pub fn parse_literal(lit: &Literal) -> Result<Self, ()> {
        let s = lit.to_string();
        let (number_str, suffix_str) = match is_decimal_number_literal(&s) {
            Some((number, suffix)) => (number, suffix),
            None => {
                lit.span()
                    .error("Failed to parse the literal as a number")
                    .emit();
                return Err(());
            }
        };

        let value_span = lit.subspan(0..number_str.len()).unwrap();
        let value = parse_f64(number_str, value_span)?;

        if suffix_str.is_empty() {
            if value == 0.0 {
                Ok(Self::Zero)
            } else {
                value_span
                    .warning("Length literal without a suffix is treated as `px`")
                    .help("Available length units are `upx`, `px`, `w%`, `h%`")
                    .emit();
                Ok(Self::Literal {
                    value,
                    suffix: LengthSuffix::Pixels,
                })
            }
        } else {
            let suffix_span = lit.subspan(number_str.len()..).unwrap();
            let suffix = LengthSuffix::parse(suffix_str, suffix_span)?;
            Ok(Self::Literal { value, suffix })
        }
    }

    /// Parses a length literal from the provided token stream.
    pub fn parse(stream: TokenStream) -> Result<Self, ()> {
        let mut tokens = stream.into_iter();

        match tokens.next() {
            Some(TokenTree::Literal(lit)) => Self::parse_literal(&lit),
            Some(tt) => {
                tt.span()
                    .error(format!("Expected a length literal, got `{tt}`"))
                    .emit();
                Err(())
            }
            None => {
                Span::call_site()
                    .error("Expected a length literal")
                    .help("If you wish to use a length of `0px`, simply use `0`")
                    .emit();
                Err(())
            }
        }
    }

    pub fn to_tokens(&self) -> TokenStream {
        let span = Span::call_site();

        let length_root = [
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("kui", span)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("elements", span)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Length", span)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        ];

        match self {
            Self::Zero => length_root
                .into_iter()
                .chain(Some(TokenTree::Ident(Ident::new("ZERO", span))))
                .collect(),
            Self::Literal { value, suffix } => length_root
                .into_iter()
                .chain([
                    TokenTree::Ident(Ident::new(suffix.identifier(), span)),
                    TokenTree::Group(Group::new(
                        Delimiter::Parenthesis,
                        Some(TokenTree::Literal(suffix.literal(*value)))
                            .into_iter()
                            .collect(),
                    )),
                ])
                .collect(),
        }
    }
}

/// Parses the provided token stream into a length literal.
pub fn parse_length_literal(tokens: TokenStream) -> TokenStream {
    Length::parse(tokens).unwrap_or(Length::Zero).to_tokens()
}
