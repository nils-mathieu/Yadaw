use {
    std::{cell::RefCell, rc::Rc},
    yadaw_ui::{
        elem::{self, ElementExt},
        element::{Element, EventResult},
        kurbo::Vec2,
    },
};

mod state;
pub use self::state::*;

mod clip;
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
        .on_animation({
            let state = state.clone();
            move |el, cx, dt| state.borrow_mut().animate(el, cx, dt)
        })
        .on_event(|el, cx, ev| {
            if matches!(ev, SequencerEvent::StartAnimating) {
                el.start_animation(cx);
            }
            EventResult::Continue
        })
        .on_ready({
            let state = state.clone();
            move |el, cx| state.borrow_mut().ready(el, cx)
        })
        .on_any_event({
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
        .with_child_height(elem::Length::ZERO)
        .with_gap(elem::Length::Pixels(16.0))
        .on_event(|el, cx, ev: &SequencerEvent| {
            if let SequencerEvent::SetZoom(zoom) = ev {
                el.set_child_height(elem::Length::Pixels(zoom.y));
                cx.window().request_redraw();
            }
            EventResult::Continue
        })
        .with_scroll_y()
        .on_event(|el, cx, ev: &SequencerEvent| {
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
    elem::Elements::default()
        .with_child(clip::audio_clip())
        .with_scroll_x()
        .with_scroll_y()
        .on_event(|el, cx, ev: &SequencerEvent| {
            if let SequencerEvent::SetDragOffset(off) = ev {
                el.set_scroll_amount(cx, *off);
                cx.window().request_redraw();
            }

            EventResult::Continue
        })
        .with_clip_rect()
        .with_radius(elem::Length::Pixels(16.0))
}
