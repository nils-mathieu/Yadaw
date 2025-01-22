use {
    crate::{
        ElemContext, Element, LayoutContext, SizeHint,
        event::{Event, EventResult, PointerButton, PointerLeft, PointerMoved},
    },
    bitflags::bitflags,
    vello::{
        Scene,
        kurbo::{Point, Size},
    },
    winit::event::{ButtonSource, MouseButton},
};

bitflags! {
    /// Represents the state of a button.
    #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct ButtonState: u8 {
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

/// Represents the appearance of a button.
pub trait ButtonAppearance: Element {
    /// The state of the button has changed.
    fn state_changed(&mut self, cx: &ElemContext, state: ButtonState);
}

/// A [`ButtonAppearance`] implementation that uses a function to control the appearance of
/// the button.
pub struct ButtonAppearanceFn<F, E: ?Sized> {
    /// The function itself.
    state_changed: F,
    /// The child element.
    child: E,
}

impl<F, E> ButtonAppearanceFn<F, E> {
    /// Creates a new [`ButtonAppearanceFn`] with the provided function and child element.
    pub fn new(state_changed: F, child: E) -> Self
    where
        F: FnMut(&mut E, &ElemContext, ButtonState),
    {
        Self {
            state_changed,
            child,
        }
    }
}

impl<F, E> Element for ButtonAppearanceFn<F, E>
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
}

impl<F, E> ButtonAppearance for ButtonAppearanceFn<F, E>
where
    F: FnMut(&mut E, &ElemContext, ButtonState),
    E: ?Sized + Element,
{
    #[inline]
    fn state_changed(&mut self, cx: &ElemContext, state: ButtonState) {
        (self.state_changed)(&mut self.child, cx, state);
    }
}

/// Represents a button.
#[derive(Clone, Debug, Default)]
pub struct Button<F, A: ?Sized> {
    state: ButtonState,

    /// The callback to call when the button is clicked.
    pub on_click: F,
    /// The appearance of the button.
    pub appearance: A,
}

impl<F, A> Button<F, A> {
    /// Creates a new [`Button`] with the provided callback and appearance.
    pub fn new(on_click: F, appearance: A) -> Self {
        Self {
            state: ButtonState::empty(),
            on_click,
            appearance,
        }
    }

    /// Sets whether the button is disabled or not.
    pub fn disabled(mut self, yes: bool) -> Self {
        self.state.set(ButtonState::DISABLED, yes);
        self
    }

    /// Sets the function that will be called when this [`Button`] is clicked.
    pub fn on_click<F2>(self, on_click: F2) -> Button<F2, A>
    where
        F2: FnMut(),
    {
        Button {
            state: self.state,
            on_click,
            appearance: self.appearance,
        }
    }

    /// Sets the appearance of the button.
    pub fn child<A2: ButtonAppearance>(self, appearance: A2) -> Button<F, A2> {
        Button {
            state: self.state,
            on_click: self.on_click,
            appearance,
        }
    }
}

impl<F, A> Button<F, A>
where
    F: FnMut(),
    A: ?Sized + ButtonAppearance,
{
    /// Attempts to handle the provided event.
    fn handle_event(&mut self, event: &dyn Event) -> EventResult {
        if let Some(ev) = event.downcast_ref::<PointerMoved>() {
            if !ev.primary {
                return EventResult::Continue;
            }

            self.state
                .set(ButtonState::HOVER, self.hit_test(ev.position));
            return EventResult::Continue;
        }

        if let Some(ev) = event.downcast_ref::<PointerButton>() {
            if !ev.primary {
                return EventResult::Continue;
            }
            if !matches!(ev.button, ButtonSource::Mouse(MouseButton::Left)) {
                return EventResult::Continue;
            }

            let hover = self.hit_test(ev.position);
            self.state.set(ButtonState::HOVER, hover);

            if ev.state.is_pressed() {
                if hover {
                    self.state.insert(ButtonState::ACTIVE);
                    return EventResult::Handled;
                }
            } else if self.state.contains(ButtonState::ACTIVE) {
                self.state.remove(ButtonState::ACTIVE);

                if hover {
                    (self.on_click)();
                }

                return EventResult::Handled;
            }
        }

        if let Some(ev) = event.downcast_ref::<PointerLeft>() {
            if !ev.primary {
                return EventResult::Continue;
            }

            self.state.remove(ButtonState::HOVER);
        }

        EventResult::Continue
    }
}

impl<F, A> Element for Button<F, A>
where
    F: FnMut(),
    A: ?Sized + ButtonAppearance,
{
    #[inline]
    fn size_hint(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        space: Size,
    ) -> SizeHint {
        self.appearance
            .size_hint(elem_context, layout_context, space)
    }

    #[inline]
    fn place(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        pos: Point,
        size: Size,
    ) {
        self.appearance
            .place(elem_context, layout_context, pos, size);
    }

    #[inline]
    fn draw(&mut self, elem_context: &ElemContext, scene: &mut Scene) {
        self.appearance.draw(elem_context, scene);
    }

    #[inline]
    fn hit_test(&self, point: Point) -> bool {
        self.appearance.hit_test(point)
    }

    #[inline]
    fn event(&mut self, elem_context: &ElemContext, event: &dyn Event) -> EventResult {
        let og_state = self.state;
        let ret = self.handle_event(event);
        if og_state != self.state {
            self.appearance.state_changed(elem_context, self.state);
        }
        if ret.is_handled() {
            return EventResult::Handled;
        }

        self.appearance.event(elem_context, event)
    }
}
