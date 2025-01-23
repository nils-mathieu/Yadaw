mod types;
pub use self::types::*;

pub mod anchor;
pub mod button;
pub mod div;
pub mod flex;
pub mod hooks;
pub mod text;

pub mod interactive;

/// Creates a new [`Div`] element.
///
/// [`Div`]: self::div::Div
pub fn div() -> self::div::Div<()> {
    self::div::Div::default()
}

/// Creates a new [`Anchor`] element.
///
/// [`Anchor`]: self::anchor::Anchor
pub fn anchor() -> self::anchor::Anchor<()> {
    self::anchor::Anchor::default()
}

/// Creates a new [`Text`] element.
///
/// [`Text`]: self::text::Text
pub fn label() -> self::text::Text<self::text::UniformStyle> {
    self::text::Text::default()
}

/// Creates a new [`Flex`] element.
///
/// [`Flex`]: self::flex::Flex
pub fn flex<'a>() -> self::flex::Flex<'a> {
    self::flex::Flex::default()
}

/// Creates a new [`FlexChild`] element.
///
/// [`FlexChild`]: self::flex::FlexChild
pub fn flex_child() -> self::flex::FlexChild<()> {
    self::flex::FlexChild::default()
}

/// Creates a new [`Button`] element.
///
/// [`Button`]: self::button::Button
pub fn button() -> self::button::Button<(), ()> {
    self::button::Button::new((), ())
}

/// Creates a new [`HookEvents`] element.
///
/// [`HookEvents`]: self::hooks::HookEvent
pub fn hook_events() -> self::hooks::HookEvent<(), ()> {
    self::hooks::HookEvent::new((), ())
}
