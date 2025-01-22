//! Allows parsing a color from a string.

use {
    crate::utility::is_string_literal,
    proc_macro2::{Span, TokenStream, TokenTree},
    quote::quote_spanned,
};

/// Creates a new color.
fn quote_rgb(r: u8, g: u8, b: u8, span: Span) -> TokenStream {
    quote_spanned! { span =>
        ::kui::peniko::Color::from_rgb8(#r, #g, #b)
    }
}

/// Creates a new color.
fn quote_rgba(r: u8, g: u8, b: u8, a: u8, span: Span) -> TokenStream {
    quote_spanned! { span =>
        ::kui::peniko::Color::from_rgba8(#r, #g, #b, #a)
    }
}

/// Quotes the transparent color.
fn quote_transparent(span: Span) -> TokenStream {
    quote_spanned! { span =>
        ::kui::peniko::Color::TRANSPARENT
    }
}

/// Parses the provided string literal as a color.
pub fn parse_color_literal(tokens: TokenStream) -> TokenStream {
    let mut iter = tokens.into_iter();

    let tt = match iter.next() {
        Some(TokenTree::Literal(lit)) => lit,
        Some(tt) => {
            tt.span()
                .unwrap()
                .error(format!("Expected a string literal, got `{tt}`"))
                .emit();
            return quote_transparent(tt.span());
        }
        None => {
            Span::call_site()
                .unwrap()
                .error("Expected a string literal")
                .emit();
            return quote_transparent(Span::call_site());
        }
    };

    let lit = tt.to_string();
    let Some(lit) = is_string_literal(&lit) else {
        tt.span()
            .unwrap()
            .error("Colors must be string literals")
            .emit();
        return quote_transparent(tt.span());
    };

    if !lit.starts_with("#") {
        tt.span()
            .unwrap()
            .error("Expected a color literal")
            .help("Color literals start with '#'")
            .emit();
        return quote_transparent(tt.span());
    }

    let lit = &lit[1..];

    fn hex_digit(a: u8, span: Span) -> u8 {
        match a {
            b'0'..=b'9' => a - b'0',
            b'a'..=b'f' => a - b'a' + 10,
            b'A'..=b'F' => a - b'A' + 10,
            _ => {
                span.unwrap().error("Expected a hexadecimal digit").emit();
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
            quote_rgb(r, g, b, tt.span())
        }
        [r1, g1, b1] => {
            let r = hex_digit(r1, spanat(0));
            let g = hex_digit(g1, spanat(1));
            let b = hex_digit(b1, spanat(2));
            quote_rgb(r | (r << 4), g | (g << 4), b | (b << 4), tt.span())
        }
        [r1, r2, g1, g2, b1, b2, a1, a2] => {
            let r = parse_hex_digit_2(r1, r2, spanat(0), spanat(1));
            let g = parse_hex_digit_2(g1, g2, spanat(2), spanat(3));
            let b = parse_hex_digit_2(b1, b2, spanat(4), spanat(5));
            let a = parse_hex_digit_2(a1, a2, spanat(6), spanat(7));
            quote_rgba(r, g, b, a, tt.span())
        }
        [r1, g1, b1, a1] => {
            let r = hex_digit(r1, spanat(0));
            let g = hex_digit(g1, spanat(1));
            let b = hex_digit(b1, spanat(2));
            let a = hex_digit(a1, spanat(3));
            quote_rgba(
                r | (r << 4),
                g | (g << 4),
                b | (b << 4),
                a | (a << 4),
                tt.span(),
            )
        }
        _ => {
            tt.span()
                .unwrap()
                .error("Expected a color literal")
                .help("Color literals are either 3 or 6 hexadecimal digits")
                .emit();
            quote_transparent(tt.span())
        }
    }
}
