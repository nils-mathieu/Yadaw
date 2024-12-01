use yadaw_ui::{
    elem::utils::exp_decay,
    element::{ElemCtx, Element, Event, EventResult},
    event::{self, MouseButton, MouseScrollDelta},
    kurbo::{Point, Vec2},
};

/// An event that the sequencer dispatches in order to control its children.
#[derive(Debug, Clone)]
pub enum SequencerEvent {
    /// Indicates that the zoom level of the sequencer has changed.
    SetZoom(Vec2),
    /// Indicates that the drag offset of the sequencer has changed.
    SetDragOffset(Vec2),
    /// Indicates that the sequencer should start its animation routine.
    StartAnimating,
}

/// The state of the dragging operation.
struct DraggingState {
    /// The last cursor position.
    ///
    /// This is updated every time a new position is received.
    last_position: Point,
}

/// The current state of the sequencer's UI.
pub struct SequencerUiState {
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

    /// If the sequencer is currently being dragged around, this
    /// field contains the state of the dragging.
    dragging_state: Option<DraggingState>,
}

impl Default for SequencerUiState {
    fn default() -> Self {
        Self {
            target_zoom: Vec2::new(300.0, 100.0),
            zoom: Vec2::new(300.0, 100.0),
            drag_offset: Vec2::ZERO,
            target_drag_offset: Vec2::ZERO,
            dragging_state: None,
        }
    }
}

impl SequencerUiState {
    /// Animates the sequencer's content.
    ///
    /// # Returns
    ///
    /// This function returns whether the sequencer is done animating itself.
    pub fn animate(&mut self, child: &mut dyn Element, cx: &ElemCtx, dt: f64) -> bool {
        let mut still_animating = false;

        //
        // ZOOM ANIMATION
        //
        if self.zoom != self.target_zoom {
            let diff = self.target_zoom - self.zoom;
            if diff.x.abs() < 0.5 && diff.y.abs() < 0.5 {
                self.zoom = self.target_zoom;
            } else {
                self.zoom = exp_decay(self.zoom, self.target_zoom, 10.0 * dt);
                still_animating = true;
            }

            child.event(cx, &SequencerEvent::SetZoom(self.zoom));
        }

        //
        // DRAG ANIMATION
        //
        if self.drag_offset != self.target_drag_offset {
            let diff = self.target_drag_offset - self.drag_offset;
            if diff.x.abs() < 0.5 && diff.y.abs() < 0.5 {
                self.drag_offset = self.target_drag_offset;
            } else {
                self.drag_offset = exp_decay(self.drag_offset, self.target_drag_offset, 10.0 * dt);
                still_animating = true;
            }

            child.event(cx, &SequencerEvent::SetDragOffset(self.drag_offset));
        }

        still_animating
    }

    /// Attempts to handle the provided event.
    pub fn event(&mut self, child: &mut dyn Element, cx: &ElemCtx, ev: &dyn Event) -> EventResult {
        let hover = cx.is_cursor_present()
            && cx
                .window()
                .last_reported_cursor_position()
                .is_some_and(|pos| child.hit_test(cx, pos));

        if let Some(ev) = ev.downcast::<event::WheelInput>() {
            if !hover {
                return EventResult::Continue;
            }

            if cx.window().modifiers().control_key() {
                //
                // CTRL is down
                // ZOOMING with mouse wheel
                //

                let mut amount = match ev.delta {
                    MouseScrollDelta::LineDelta(dx, dy) => Vec2::new(dx as f64, dy as f64),
                    MouseScrollDelta::PixelDelta(delta) => Vec2::new(delta.x, delta.y),
                };

                let animate = matches!(ev.delta, MouseScrollDelta::LineDelta(..));

                if cx.window().modifiers().shift_key() {
                    std::mem::swap(&mut amount.x, &mut amount.y);
                }

                if !animate {
                    self.target_zoom = self.zoom;
                }

                self.target_zoom.x *= 1.1_f64.powf(amount.x);
                self.target_zoom.x = self.target_zoom.x.clamp(100.0, 1000.0);
                self.target_zoom.y *= 1.1_f64.powf(amount.y);
                self.target_zoom.y = self.target_zoom.y.clamp(32.0, 512.0);

                if animate {
                    child.event(cx, &SequencerEvent::StartAnimating);
                } else {
                    self.zoom = self.target_zoom;
                    child.event(cx, &SequencerEvent::SetZoom(self.zoom));
                }
            } else {
                //
                // CTRL is released
                // SCROLLING/DRAGGING with mouse wheel
                //

                let amount = match ev.delta {
                    MouseScrollDelta::LineDelta(dx, dy) => {
                        let mut ret = Vec2::new(dx as f64, dy as f64) * 60.0;

                        if cx.window().modifiers().shift_key() {
                            std::mem::swap(&mut ret.x, &mut ret.y);
                        }

                        ret
                    }
                    MouseScrollDelta::PixelDelta(delta) => Vec2::new(delta.x, delta.y),
                };

                let animate = matches!(ev.delta, MouseScrollDelta::LineDelta(..));

                if !animate {
                    self.target_drag_offset = self.drag_offset;
                }

                self.target_drag_offset += amount;
                self.target_drag_offset.y = self.target_drag_offset.y.min(0.0);
                self.target_drag_offset.x = self.target_drag_offset.x.min(0.0);

                if animate {
                    child.event(cx, &SequencerEvent::StartAnimating);
                } else {
                    self.drag_offset = self.target_drag_offset;
                    child.event(cx, &SequencerEvent::SetDragOffset(self.drag_offset));
                }
            }

            return EventResult::StopPropagation;
        } else if let Some(ev) = ev.downcast::<event::MouseInput>() {
            if !hover {
                return EventResult::Continue;
            }

            if ev.button == MouseButton::Middle {
                //
                // START/STOP DRAGGING
                //

                if ev.state.is_pressed() && self.dragging_state.is_none() {
                    if let Some(pos) = cx.window().last_reported_cursor_position() {
                        self.dragging_state = Some(DraggingState { last_position: pos });
                    }
                } else if !ev.state.is_pressed() && self.dragging_state.is_some() {
                    self.dragging_state = None;
                }
            }

            return EventResult::StopPropagation;
        } else if let Some(ev) = ev.downcast::<event::CursorMoved>() {
            //
            // CONTINUE DRAGGING
            //

            if let Some(drag_state) = self.dragging_state.as_mut() {
                let diff = ev.position - drag_state.last_position;
                drag_state.last_position = ev.position;

                self.drag_offset += diff;
                self.drag_offset.x = self.drag_offset.x.min(0.0);
                self.drag_offset.y = self.drag_offset.y.min(0.0);
                self.target_drag_offset = self.drag_offset;

                child.event(cx, &SequencerEvent::SetDragOffset(self.drag_offset));
                cx.window().request_redraw();
            }
        } else if let Some(ev) = ev.downcast::<event::PinchGesture>() {
            let amount = if cx.window().modifiers().shift_key() {
                Vec2::new(ev.delta, 0.0) * 50.0
            } else {
                Vec2::new(0.0, ev.delta) * 50.0
            };

            self.zoom = self.target_zoom;
            self.target_zoom += amount;
            self.target_zoom.x = self.target_zoom.x.clamp(100.0, 1000.0);
            self.target_zoom.y = self.target_zoom.y.clamp(32.0, 512.0);
            self.zoom = self.target_zoom;
            child.event(cx, &SequencerEvent::SetZoom(self.zoom));
        }

        EventResult::Continue
    }
}
