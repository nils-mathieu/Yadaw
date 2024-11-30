use yadaw_ui::element::Element;

pub mod sequencer;

/// Builds the application tree.
pub fn app() -> impl Element {
    self::sequencer::build()
}
