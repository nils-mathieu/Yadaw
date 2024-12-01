use {
    std::{cell::RefCell, rc::Rc},
    yadaw_ui::{
        elem::{self, ElementExt},
        element::{Element, EventResult, SetSize},
        kurbo::Vec2,
        peniko::Color,
    },
};

mod state;
pub use self::state::*;

mod track_header;

/// Builds the sequencer UI.
pub fn build() -> impl Element {
    let state = Rc::new(RefCell::new(SequencerUiState::default()));

    elem::LinearLayout::<Box<dyn Element>>::default()
        .with_horizontal()
        .with_align_stretch()
        .with_gap(elem::Length::Pixels(16.0))
        .with_child(track_header_column().into_dyn_element())
        .with_child(sequencer_content().into_dyn_element().with_grow(1.0))
        .with_margin(elem::Length::Pixels(16.0))
        .hook_animation({
            let state = state.clone();
            move |el, cx, dt| state.borrow_mut().animate(el, cx, dt)
        })
        .catch_event(|el, cx, ev| {
            if matches!(ev, SequencerEvent::StartAnimating) {
                el.start_animation(cx);
            }
            EventResult::Continue
        })
        .hook_events({
            let state = state.clone();
            move |el, cx, ev| state.borrow_mut().event(el, cx, ev)
        })
}

/// Builds the element responsible for displaying the infinite column of track headers
/// on the left of the sequencer.
fn track_header_column() -> impl Element {
    elem::LazyLinearLayout::new(self::track_header::build)
        .with_direction_vertical()
        .with_child_width(elem::Length::Pixels(300.0))
        .with_child_height(elem::Length::Pixels(100.0))
        .with_gap(elem::Length::Pixels(16.0))
        .catch_event(|el, cx, ev: &SequencerEvent| {
            if let SequencerEvent::SetZoom(zoom) = ev {
                el.set_child_height(elem::Length::Pixels(zoom.y));
                cx.window().request_redraw();
            }
            EventResult::Continue
        })
        .with_scroll_y()
        .catch_event(|el, cx, ev: &SequencerEvent| {
            if let SequencerEvent::SetDragOffset(off) = ev {
                el.set_scroll_amount(cx, Vec2::new(0.0, off.y));
                cx.window().request_redraw();
            }
            EventResult::Continue
        })
        .with_clip_rect()
        .with_radius(elem::Length::Pixels(16.0))
}

/// Builds the element responsible for displaying the sequencer's content. This includes
/// an infinite canvas on which clips can be placed.
fn sequencer_content() -> impl Element {
    elem::Canvas::<Box<dyn Element>>::default()
        .with_child(
            elem::canvas::Child::new(
                elem::SolidShape::<elem::shapes::Rect>::default()
                    .with_brush(Color::ALICE_BLUE)
                    .with_default_width(elem::Length::Pixels(100.0))
                    .with_default_height(elem::Length::Pixels(100.0))
                    .catch_event(|el, cx, ev: &SequencerEvent| {
                        if let SequencerEvent::SetZoom(zoom) = ev {
                            el.new_width = Some(elem::Length::Pixels(zoom.x));
                            el.new_height = Some(elem::Length::Pixels(zoom.y));
                            el.set_size(cx, SetSize::relaxed());
                            cx.window().request_redraw();
                        }

                        EventResult::Continue
                    })
                    .into_dyn_element(),
            )
            .with_position(Vec2::new(50.0, 50.0)),
        )
        .with_scroll_x()
        .with_scroll_y()
        .catch_event(|el, cx, ev: &SequencerEvent| {
            if let SequencerEvent::SetDragOffset(off) = ev {
                el.set_scroll_amount(cx, *off);
                cx.window().request_redraw();
            }

            EventResult::Continue
        })
        .with_clip_rect()
        .with_radius(elem::Length::Pixels(16.0))
}
