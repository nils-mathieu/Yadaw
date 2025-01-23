use {
    crate::{
        ElemContext, Element, LayoutContext, SizeHint,
        event::{Event, EventResult, PointerButton, PointerLeft, PointerMoved},
    },
    bitflags::bitflags,
    vello::kurbo::{Point, Size},
    winit::event::{ButtonSource, MouseButton},
};

bitflags! {
    /// The result of interacting with an input element.
    #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Interaction: u8 {
        /// Whether the provided event was handled or not.
        const EVENT_HANDLED = 1 << 0;
        /// The element was pressed.
        const PRESSED = 1 << 1;
        /// The element was released.
        const RELEASED = 1 << 2;
        // The element was clicked.
        const CLICKED = 1 << 3;
        /// The pointer entered the element.
        const ENTERED = 1 << 4;
        /// The pointer left the element.
        const LEFT = 1 << 5;
    }
}

impl Interaction {
    /// Turns this interaction into an [`EventResult`].
    pub fn to_event_result(self) -> EventResult {
        if self.contains(Interaction::EVENT_HANDLED) {
            EventResult::Handled
        } else {
            EventResult::Continue
        }
    }

    /// Returns whether the event was handled.
    pub fn event_handled(self) -> bool {
        self.contains(Interaction::EVENT_HANDLED)
    }

    /// Returns whether the element was pressed.
    #[inline]
    pub fn pressed(self) -> bool {
        self.contains(Interaction::PRESSED)
    }

    /// Returns whether the element was released.
    #[inline]
    pub fn released(self) -> bool {
        self.contains(Interaction::RELEASED)
    }

    /// Returns whether the element was clicked.
    #[inline]
    pub fn clicked(self) -> bool {
        self.contains(Interaction::CLICKED)
    }

    /// Returns whether the pointer entered the element.
    #[inline]
    pub fn entered(self) -> bool {
        self.contains(Interaction::ENTERED)
    }

    /// Returns whether the pointer left the element.
    #[inline]
    pub fn left(self) -> bool {
        self.contains(Interaction::LEFT)
    }
}

bitflags! {
    /// Represents the state of an element capable of reacting to a user's inputs.
    #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct InteractiveState: u8 {
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
    }
}

impl InteractiveState {
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

    /// Handles the provided event, updating the state of the element accordingly.
    pub fn handle_interactions(
        &mut self,
        appearance: &mut dyn InputAppearance,
        event: &dyn Event,
    ) -> Interaction {
        if let Some(ev) = event.downcast_ref::<PointerMoved>() {
            if !ev.primary {
                return Interaction::empty();
            }

            if appearance.hit_test(ev.position) {
                self.insert(InteractiveState::HOVER);
                return Interaction::ENTERED;
            } else {
                self.remove(InteractiveState::HOVER);
                return Interaction::LEFT;
            }
        }

        if let Some(ev) = event.downcast_ref::<PointerButton>() {
            if !ev.primary {
                return Interaction::empty();
            }
            if !matches!(ev.button, ButtonSource::Mouse(MouseButton::Left)) {
                return Interaction::empty();
            }

            let hover = appearance.hit_test(ev.position);
            self.set(InteractiveState::HOVER, hover);

            if ev.state.is_pressed() {
                if hover {
                    self.insert(InteractiveState::ACTIVE);
                    return Interaction::EVENT_HANDLED | Interaction::PRESSED;
                }
            } else if self.contains(InteractiveState::ACTIVE) {
                self.remove(InteractiveState::ACTIVE);

                if hover {
                    return Interaction::EVENT_HANDLED
                        | Interaction::RELEASED
                        | Interaction::CLICKED;
                } else {
                    return Interaction::EVENT_HANDLED | Interaction::RELEASED;
                }
            }
        }

        if let Some(ev) = event.downcast_ref::<PointerLeft>() {
            if !ev.primary {
                return Interaction::empty();
            }

            if self.contains(InteractiveState::HOVER) {
                self.remove(InteractiveState::HOVER);
                return Interaction::LEFT | Interaction::EVENT_HANDLED;
            }
        }

        Interaction::empty()
    }
}

/// Represents the appearance of an input element.
pub trait InputAppearance: Element {
    /// The state of the input element has changed.
    fn state_changed(&mut self, cx: &ElemContext, state: InteractiveState);
}

impl InputAppearance for () {
    fn state_changed(&mut self, _cx: &ElemContext, _state: InteractiveState) {}
}

/// A [`InputAppearance`] implementation that uses a function to control the appearance of
/// the element.
///
/// Instances of this type are created through [`make_appearance`].
pub struct InputAppearanceFn<F, E: ?Sized> {
    /// The function itself.
    state_changed: F,
    /// The child element.
    child: E,
}

impl<F, E> Element for InputAppearanceFn<F, E>
where
    E: ?Sized + Element,
{
    #[inline]
    fn size_hint(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        space: Size,
    ) -> SizeHint {
        self.child.size_hint(elem_context, layout_context, space)
    }

    #[inline]
    fn place(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        pos: Point,
        size: Size,
    ) {
        self.child.place(elem_context, layout_context, pos, size);
    }

    #[inline]
    fn draw(&mut self, elem_context: &ElemContext, scene: &mut vello::Scene) {
        self.child.draw(elem_context, scene);
    }

    #[inline]
    fn event(&mut self, elem_context: &ElemContext, event: &dyn Event) -> EventResult {
        self.child.event(elem_context, event)
    }

    #[inline]
    fn hit_test(&self, point: Point) -> bool {
        self.child.hit_test(point)
    }

    #[inline]
    fn begin(&mut self, elem_context: &ElemContext) {
        self.child.begin(elem_context);
    }
}

impl<F, E> InputAppearance for InputAppearanceFn<F, E>
where
    F: FnMut(&mut E, &ElemContext, InteractiveState),
    E: ?Sized + Element,
{
    #[inline]
    fn state_changed(&mut self, cx: &ElemContext, state: InteractiveState) {
        (self.state_changed)(&mut self.child, cx, state);
    }
}

/// Creates a new [`InputElementAppearance`] instance from the provided function,
pub fn make_appearance<F, E>(elem: E, state_changed: F) -> InputAppearanceFn<F, E>
where
    E: Element,
    F: FnMut(&mut E, &ElemContext, InteractiveState),
{
    InputAppearanceFn {
        state_changed,
        child: elem,
    }
}
