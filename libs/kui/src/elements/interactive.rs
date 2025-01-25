use {
    crate::event::{Event, EventResult, PointerButton, PointerLeft, PointerMoved},
    bitflags::bitflags,
    vello::kurbo::Point,
    winit::event::{ButtonSource, MouseButton},
};

bitflags! {
    /// Represents the state of an element capable of reacting to a user's inputs.
    #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct InteractiveState: u16 {
        /// The pointer is hover the button.
        const HOVER = 1 << 0;
        /// The pointer is pressing the button.
        const ACTIVE = 1 << 1;
        /// The button is disabled.
        const DISABLED = 1 << 2;
        /// The button is focused.
        const FOCUS = 1 << 3;
        /// The button is visibly focused.
        const FOCUS_VISIBLE = 1 << 4;

        /// The element was just pressed.
        ///
        /// When "act on press" is enabled, this will be the moment where the callback of a
        /// button is called.
        const JUST_PRESSED = 1 << 6;
        /// The element was just released.
        const JUST_RELEASED = 1 << 7;
        /// The element was clicked.
        ///
        /// When "act on release" is enabled, this will be the moment where the callback of a
        /// button is called.
        const JUST_CLICKED = 1 << 8;
        /// The pointer entered the element.
        const JUST_ENTERED = 1 << 9;
        /// The pointer left the element.
        const JUST_LEFT = 1 << 10;
        /// Whether the element is just focused.
        const JUST_FOCUSED = 1 << 11;
        /// Whether the element is just unfocused.
        const JUST_UNFOCUSED = 1 << 12;

        /// The value of the element changed.
        const VALUE_CHANGED = 1 << 11;
    }
}

impl InteractiveState {
    /// Removes transient states from the element.
    pub fn remove_transient_states(&mut self) {
        self.remove(
            InteractiveState::JUST_PRESSED
                | InteractiveState::JUST_RELEASED
                | InteractiveState::JUST_CLICKED
                | InteractiveState::JUST_ENTERED
                | InteractiveState::JUST_LEFT
                | InteractiveState::JUST_FOCUSED
                | InteractiveState::JUST_UNFOCUSED
                | InteractiveState::VALUE_CHANGED,
        );
    }

    /// Whether the element is being hovered.
    #[inline]
    pub fn hover(self) -> bool {
        self.contains(InteractiveState::HOVER)
    }

    /// Whether the element is active (being pressed).
    #[inline]
    pub fn active(self) -> bool {
        self.contains(InteractiveState::ACTIVE)
    }

    /// Whether the element is disabled.
    #[inline]
    pub fn disabled(self) -> bool {
        self.contains(InteractiveState::DISABLED)
    }

    /// Whether the element is focused.
    #[inline]
    pub fn focused(self) -> bool {
        self.contains(InteractiveState::FOCUS)
    }

    /// Whether the element is visibly focused.
    #[inline]
    pub fn focus_visible(self) -> bool {
        self.contains(InteractiveState::FOCUS_VISIBLE)
    }

    /// Whether the element was just pressed.
    #[inline]
    pub fn just_pressed(self) -> bool {
        self.contains(InteractiveState::JUST_PRESSED)
    }

    /// Whether the element was just released.
    #[inline]
    pub fn just_released(self) -> bool {
        self.contains(InteractiveState::JUST_RELEASED)
    }

    /// Whether the element was clicked.
    #[inline]
    pub fn just_clicked(self) -> bool {
        self.contains(InteractiveState::JUST_CLICKED)
    }

    /// Whether the pointer just entered the element.
    #[inline]
    pub fn just_entered(self) -> bool {
        self.contains(InteractiveState::JUST_ENTERED)
    }

    /// Whether the pointer just left the element.
    #[inline]
    pub fn just_left(self) -> bool {
        self.contains(InteractiveState::JUST_LEFT)
    }

    /// Whether the value of the element changed.
    #[inline]
    pub fn value_changed(self) -> bool {
        self.contains(InteractiveState::VALUE_CHANGED)
    }

    /// Whether the element was just focused.
    #[inline]
    pub fn just_focused(self) -> bool {
        self.contains(InteractiveState::JUST_FOCUSED)
    }

    /// Whether the element was just unfocused.
    #[inline]
    pub fn just_unfocused(self) -> bool {
        self.contains(InteractiveState::JUST_UNFOCUSED)
    }

    /// Handles the provided event, updating the state of the element accordingly.
    pub fn handle_pointer_interactions(
        &mut self,
        hit_test: &mut dyn FnMut(Point) -> bool,
        event: &dyn Event,
    ) -> EventResult {
        if let Some(ev) = event.downcast_ref::<PointerMoved>() {
            if !ev.primary {
                return EventResult::Continue;
            }

            let now_hover = hit_test(ev.position);

            if self.hover() == now_hover {
                return EventResult::Continue;
            }

            if now_hover {
                self.insert(InteractiveState::HOVER | InteractiveState::JUST_ENTERED);
                return EventResult::Continue;
            } else {
                self.remove(InteractiveState::HOVER);
                self.insert(InteractiveState::JUST_LEFT);
                return EventResult::Continue;
            }
        }

        if let Some(ev) = event.downcast_ref::<PointerButton>() {
            if !ev.primary {
                return EventResult::Continue;
            }
            if !matches!(ev.button, ButtonSource::Mouse(MouseButton::Left)) {
                return EventResult::Continue;
            }

            let hover = hit_test(ev.position);
            self.set(InteractiveState::HOVER, hover);

            if ev.state.is_pressed() {
                if hover {
                    self.insert(
                        InteractiveState::ACTIVE
                            | InteractiveState::FOCUS
                            | InteractiveState::JUST_FOCUSED
                            | InteractiveState::JUST_PRESSED,
                    );
                    return EventResult::Handled;
                } else {
                    self.remove(InteractiveState::FOCUS);
                    self.insert(InteractiveState::JUST_UNFOCUSED);
                    return EventResult::Continue;
                }
            } else if self.active() {
                self.remove(InteractiveState::ACTIVE);

                if hover {
                    self.insert(InteractiveState::JUST_RELEASED | InteractiveState::JUST_CLICKED);
                    return EventResult::Handled;
                } else {
                    self.insert(InteractiveState::JUST_RELEASED);
                    return EventResult::Continue;
                }
            }
        }

        if let Some(ev) = event.downcast_ref::<PointerLeft>() {
            if !ev.primary {
                return EventResult::Continue;
            }

            if self.hover() {
                self.remove(InteractiveState::HOVER | InteractiveState::JUST_LEFT);
                return EventResult::Continue;
            }

            return EventResult::Continue;
        }

        EventResult::Continue
    }
}
