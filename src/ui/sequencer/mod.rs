use {
    crate::ui::FOREGROUND_COLOR,
    yadaw_ui::{
        elem::{self, ElementExt},
        element::Element,
        parley::FontWeight,
    },
};

/// Builds the sequencer UI.
pub fn build() -> impl Element {
    elem::LinearLayout::<Box<dyn Element>>::default()
        .with_horizontal()
        .with_align_stretch()
        .with_child(
            elem::LazyLinearLayout::new(|index| {
                elem::Text::new(format!("Track {}", index + 1))
                    .with_basic_style()
                    .with_break_lines(false)
                    .with_brush(FOREGROUND_COLOR)
                    .with_font_size(elem::Length::Pixels(16.0))
                    .with_weight(FontWeight::BOLD)
                    .with_margin(elem::Length::Pixels(16.0))
                    .with_background(FOREGROUND_COLOR.multiply_alpha(0.05))
                    .with_radius(elem::Length::Pixels(16.0))
                    .with_margin_left(elem::Length::Pixels(16.0))
            })
            .with_direction_vertical()
            .with_child_width(elem::Length::Pixels(300.0))
            .with_child_height(elem::Length::Pixels(100.0))
            .with_gap(elem::Length::Pixels(16.0))
            .with_scroll_y()
            .with_controls()
            .into_dyn_element(),
        )
}
