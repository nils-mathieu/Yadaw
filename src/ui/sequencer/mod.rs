use yadaw_ui::{
    elem::{self, ElementExt},
    element::Element,
    peniko::Color,
    CursorIcon,
};

/// Builds the sequencer UI.
pub fn build() -> impl Element {
    elem::LinearLayout::default()
        .with_horizontal()
        .with_justify_center()
        .with_align_center()
        .with_gap(elem::Length::Pixels(20.0))
        .with_child(rect(Color::RED, 100.0, 100.0))
        .with_child(
            elem::linear_layout::Child::new(rect(Color::GREEN, 100.0, 100.0)).with_grow(1.0),
        )
        .with_child(rect(Color::BLUE, 100.0, 100.0))
        .with_child(
            elem::linear_layout::Child::new(rect(Color::YELLOW, 100.0, 100.0)).with_grow(2.0),
        )
}

fn rect(color: Color, width: f64, height: f64) -> impl Element {
    elem::ShapeElement::<elem::shapes::RoundedRectangle>::default()
        .with_brush(color)
        .with_radius(elem::Length::Pixels(10.0))
        .with_default_size(elem::Length::Pixels(width), elem::Length::Pixels(height))
        .with_cursor(CursorIcon::Pointer)
}
