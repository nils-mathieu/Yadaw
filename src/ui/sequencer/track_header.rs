use {
    crate::ui::FOREGROUND_COLOR,
    yadaw_ui::{
        elem::{self, ElementExt},
        element::Element,
        parley::FontWeight,
        peniko::Color,
        CursorIcon,
    },
};

/// Builds the element that displays a track header (on the left of the sequencer).
///
/// The size of the returned element is directed by its parent.
pub fn build(index: usize) -> impl Element {
    elem::LinearLayout::default()
        .with_vertical()
        .with_align_stretch()
        .with_justify_start()
        .with_child(
            elem::LinearLayout::<Box<dyn Element>>::default()
                .with_align_center()
                .with_justify_start()
                .with_child(
                    elem::Text::new(format!("Track {}", index))
                        .with_basic_style()
                        .with_break_lines(false)
                        .with_brush(FOREGROUND_COLOR)
                        .with_font_size(elem::Length::Pixels(16.0))
                        .with_weight(FontWeight::BOLD)
                        .into_dyn_element(),
                )
                .with_space(1.0)
                .with_child(
                    elem::SolidShape::<elem::shapes::Ellipse>::default()
                        .with_brush(Color::rgb8(0x1E, 0xBA, 0xFF))
                        .with_default_width(elem::Length::Pixels(12.0))
                        .with_default_height(elem::Length::Pixels(12.0))
                        .with_cursor(CursorIcon::Pointer)
                        .into_dyn_element(),
                ),
        )
        .with_margin(elem::Length::Pixels(16.0))
        .with_background(FOREGROUND_COLOR.multiply_alpha(0.05))
        .with_radius(elem::Length::Pixels(16.0))
}
