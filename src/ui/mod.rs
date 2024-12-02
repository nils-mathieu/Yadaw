use yadaw_ui::{element::Element, peniko::Color};

pub mod sequencer;

pub const FONT_FAMILY: &str = "nunito, sans-serif";
pub const BACKGROUND_COLOR: Color = Color::rgb8(0x11, 0x11, 0x11);
pub const FOREGROUND_COLOR: Color = Color::rgb8(0xE5, 0xE5, 0xE5);

/// Builds the application tree.
pub fn app() -> impl Element {
    self::sequencer::build()
}
