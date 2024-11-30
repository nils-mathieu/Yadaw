use {
    crate::ui::FOREGROUND_COLOR,
    std::rc::{Rc, Weak},
    yadaw_ui::{
        elem::{self, ElementExt},
        element::{Element, EventResult},
        event::{self, MouseScrollDelta},
        kurbo::Vec2,
        parley::FontWeight,
        peniko::Color,
        CursorIcon,
    },
};

/// Builds the sequencer UI.
pub fn build() -> impl Element {
    let mut header_layout = Weak::new();

    elem::LinearLayout::<Box<dyn Element>>::default()
        .with_horizontal()
        .with_align_stretch()
        .with_child(
            elem::LazyLinearLayout::new(|index| {
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
                                elem::ShapeElement::<elem::shapes::Ellipse>::default()
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
                    .into_dyn_element()
            })
            .with_direction_vertical()
            .with_child_width(elem::Length::Pixels(300.0))
            .with_child_height(elem::Length::Pixels(100.0))
            .with_gap(elem::Length::Pixels(16.0))
            .with_scroll_y()
            .into_ref_bind(|e| header_layout = Rc::downgrade(e))
            .with_clip_rect()
            .with_radius(elem::Length::Pixels(16.0))
            .into_dyn_element(),
        )
        .hook_events({
            let header_layout = header_layout.clone();
            let target_zoom = Vec2::new(300.0, 100.0);
            let mut current_zoom = target_zoom;
            move |_el, cx, ev| {
                if let Some(ev) = ev.downcast::<event::WheelInput>() {
                    match ev.delta {
                        MouseScrollDelta::LineDelta(dx, dy) => {
                            current_zoom.x *= 1.1_f64.powf(dx as f64);
                            current_zoom.y *= 1.1_f64.powf(dy as f64);
                            header_layout
                                .upgrade()
                                .unwrap()
                                .borrow_mut()
                                .child
                                .set_child_width(elem::Length::Pixels(current_zoom.x));
                            header_layout
                                .upgrade()
                                .unwrap()
                                .borrow_mut()
                                .child
                                .set_child_height(elem::Length::Pixels(current_zoom.y));
                            cx.window().request_redraw();
                        }
                        MouseScrollDelta::PixelDelta(_) => {}
                    }

                    return EventResult::Handled;
                }

                EventResult::Ignored
            }
        })
        .with_margin(elem::Length::Pixels(16.0))
}
