mod types;
pub use self::types::*;

pub mod anchor;
pub mod div;

/// Creates a new [`Div`] element.
pub fn div() -> self::div::Div<()> {
    self::div::Div::default()
}

/// Creates a new [`Anchor`] element.
pub fn anchor() -> self::anchor::Anchor<()> {
    self::anchor::Anchor::default()
}
