mod types;
pub use self::types::*;

pub mod anchor;
pub mod div;
pub mod flex;
pub mod text;

/// Creates a new [`Div`] element.
pub fn div() -> self::div::Div<()> {
    self::div::Div::default()
}

/// Creates a new [`Anchor`] element.
pub fn anchor() -> self::anchor::Anchor<()> {
    self::anchor::Anchor::default()
}

/// Creates a new [`Text`] element.
pub fn label() -> self::text::Text<self::text::UniformStyle> {
    self::text::Text::default()
}

/// Creates a new [`Flex`] element.
pub fn flex<'a>() -> self::flex::Flex<'a> {
    self::flex::Flex::default()
}

/// Creates a new [`FlexChild`] element.
pub fn flex_child() -> self::flex::FlexChild<()> {
    self::flex::FlexChild::default()
}
