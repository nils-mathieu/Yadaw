use {
    super::SequencerEvent,
    crate::ui::{BACKGROUND_COLOR, FONT_FAMILY, FOREGROUND_COLOR},
    yadaw_ui::{
        elem::{self, utils::text_color_for_background, ElementExt},
        element::{Element, EventResult, SetSize},
        peniko::Color,
    },
};

/// Builds a clip of audio data.
pub fn audio_clip() -> impl Element {
    elem::LinearLayout::<Box<dyn Element>>::default()
        .with_vertical()
        .with_align_stretch()
        .with_justify_start()
        .with_child(
            elem::SolidShape::<elem::shapes::RoundedRect>::default()
                .with_top_left_radius(elem::Length::Pixels(8.0))
                .with_top_right_radius(elem::Length::Pixels(8.0))
                .with_brush(Color::rgb8(0x5E, 0xF3, 0x8C))
                .with_child(
                    elem::Text::new("Audio Clip")
                        .with_basic_style()
                        .with_font_family(FONT_FAMILY)
                        .with_brush(
                            if text_color_for_background(Color::rgb8(0x5E, 0xF3, 0x8C)) {
                                BACKGROUND_COLOR
                            } else {
                                FOREGROUND_COLOR
                            },
                        )
                        .with_margin_left(elem::Length::Pixels(8.0)),
                )
                .with_default_height(elem::Length::Pixels(24.0))
                .into_dyn_element(),
        )
        .with_child(
            elem::SolidShape::<elem::shapes::RoundedRect>::default()
                .with_bottom_left_radius(elem::Length::Pixels(8.0))
                .with_bottom_right_radius(elem::Length::Pixels(8.0))
                .with_brush(FOREGROUND_COLOR.multiply_alpha(0.05))
                .into_dyn_element()
                .with_grow(1.0),
        )
        .with_default_size(elem::Length::ZERO, elem::Length::ZERO)
        .on_event(|el, cx, ev: &SequencerEvent| {
            if let SequencerEvent::SetZoom(zoom) = ev {
                el.default_width = Some(elem::Length::Pixels(zoom.x));
                el.default_height = Some(elem::Length::Pixels(zoom.y));
                el.set_size(cx, SetSize::from_specific((zoom.x, zoom.y)));
                cx.window().request_redraw();
            }

            EventResult::Continue
        })
        .with_translation((0.0, 0.0))
}
