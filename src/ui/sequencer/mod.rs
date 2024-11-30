use {
    crate::ui::FOREGROUND_COLOR,
    std::{
        cell::RefCell,
        rc::{Rc, Weak},
    },
    yadaw_ui::{
        elem::{self, utils::exp_decay, ElementExt},
        element::{Element, EventResult},
        event::{self, MouseScrollDelta},
        kurbo::Vec2,
        parley::FontWeight,
        peniko::Color,
        CursorIcon,
    },
};

/// The current state of the sequencer's UI.
struct SequencerUiState {
    /// The non-animated zoom level.
    ///
    /// See [`zoom`] for more information.
    ///
    /// [`zoom`]: SequencerUiState::zoom
    target_zoom: Vec2,

    /// The current zoom level.
    ///
    /// This value is animated and follows the [`target_zoom`] value.
    ///
    /// - The horizontal component of this value is the width of each second as represented
    ///   in the sequencer.
    ///
    /// - The vertical component of this value is the height of each track as represented
    ///   in the sequencer (not that a gap of 16 pixels is added between each track).
    ///
    /// [`target_zoom`]: SequencerUiState::target_zoom
    zoom: Vec2,

    /// The non-animated drag offset of the sequencer.
    ///
    /// See [`drag_offset`] for more information.
    ///
    /// [`drag_offset`]: SequencerUiState::drag_offset
    target_drag_offset: Vec2,

    /// The drag offset of the sequencer.
    ///
    /// This value is animated and follows the [`target_drag_offset`] value.
    ///
    /// - The horizontal component of this value is the horizontal offset of the sequencer.
    ///
    /// - The vertical component of this value is the vertical offset of the sequencer.
    ///
    /// Both values are in pixels.
    ///
    /// [`target_drag_offset`]: SequencerUiState::target_drag_offset
    drag_offset: Vec2,
}

impl Default for SequencerUiState {
    fn default() -> Self {
        Self {
            target_zoom: Vec2::new(300.0, 100.0),
            zoom: Vec2::new(300.0, 100.0),
            drag_offset: Vec2::ZERO,
            target_drag_offset: Vec2::ZERO,
        }
    }
}

/// Builds the sequencer UI.
pub fn build() -> impl Element {
    let mut header_layout = Weak::new();
    let state = Rc::new(RefCell::new(SequencerUiState::default()));

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
        .hook_animation({
            let header_layout = header_layout.clone();
            let state = state.clone();
            move |_el, cx, dt| {
                let mut state = state.borrow_mut();
                let header_layout = header_layout.upgrade().unwrap();
                let mut header_layout = header_layout.borrow_mut();

                if state.zoom != state.target_zoom {
                    let diff = state.target_zoom - state.zoom;
                    if diff.x.abs() < 0.5 && diff.y.abs() < 0.5 {
                        state.zoom = state.target_zoom;
                    } else {
                        state.zoom = exp_decay(state.zoom, state.target_zoom, 10.0 * dt);
                    }

                    header_layout
                        .child
                        .set_child_height(elem::Length::Pixels(state.zoom.y));
                }

                if state.drag_offset != state.target_drag_offset {
                    let diff = state.target_drag_offset - state.drag_offset;
                    if diff.x.abs() < 0.5 && diff.y.abs() < 0.5 {
                        state.drag_offset = state.target_drag_offset;
                    } else {
                        state.drag_offset =
                            exp_decay(state.drag_offset, state.target_drag_offset, 10.0 * dt);
                    }

                    header_layout.set_scroll_amount(cx, Vec2::new(0.0, state.drag_offset.y));
                }

                state.zoom != state.target_zoom || state.drag_offset != state.target_drag_offset
            }
        })
        .hook_events({
            let state = state.clone();
            move |el, cx, ev| {
                let mut state = state.borrow_mut();

                if let Some(ev) = ev.downcast::<event::WheelInput>() {
                    if cx.window().modifiers().control_key() {
                        let mut amount = match ev.delta {
                            MouseScrollDelta::LineDelta(dx, dy) => Vec2::new(dx as f64, dy as f64),
                            MouseScrollDelta::PixelDelta(delta) => Vec2::new(delta.x, delta.y),
                        };

                        if cx.window().modifiers().shift_key() {
                            std::mem::swap(&mut amount.x, &mut amount.y);
                        }

                        state.target_zoom.x *= 1.1_f64.powf(amount.x);
                        state.target_zoom.x = state.target_zoom.x.clamp(100.0, 1000.0);
                        state.target_zoom.y *= 1.1_f64.powf(amount.y);
                        state.target_zoom.y = state.target_zoom.y.clamp(32.0, 512.0);
                        el.start_animation(cx);
                    } else {
                        let mut amount = match ev.delta {
                            MouseScrollDelta::LineDelta(dx, dy) => {
                                Vec2::new(dx as f64, dy as f64) * 60.0
                            }
                            MouseScrollDelta::PixelDelta(delta) => Vec2::new(delta.x, delta.y),
                        };

                        if cx.window().modifiers().shift_key() {
                            std::mem::swap(&mut amount.x, &mut amount.y);
                        }

                        state.target_drag_offset += amount;
                        state.target_drag_offset.y = state.target_drag_offset.y.min(0.0);
                        state.target_drag_offset.x = state.target_drag_offset.x.min(0.0);
                        el.start_animation(cx);
                    }
                    return EventResult::Handled;
                }

                EventResult::Ignored
            }
        })
        .with_margin(elem::Length::Pixels(16.0))
}
