/// A list of standard suffixes for numeric literals.
pub const STANDARD_SUFFIXES: [&str; 14] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64",
];

/// Returns whether the provided string represents a numeric literal of any kind.
pub fn is_decimal_number_literal(lit: &str) -> Option<(&str, &str)> {
    // We don't support E notation for now.

    let number_len = lit
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(lit.len());

    let (num, suffix) = lit.split_at(number_len);

    if !num.is_empty() && !suffix.contains(|c: char| !c.is_ascii_lowercase()) {
        Some((num, suffix))
    } else {
        None
    }
}

/// Returns whether the provided string represents a string literal.
pub fn is_string_literal(lit: &str) -> Option<&str> {
    if lit.starts_with('"') && lit.ends_with('"') {
        Some(&lit[1..lit.len() - 1])
    } else {
        None
    }
}
