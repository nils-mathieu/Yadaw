use {
    super::interactive::{InputAppearance, InteractiveState},
    crate::{
        ElemContext, Element, LayoutContext, SizeHint,
        event::{Event, EventResult},
    },
    vello::{
        Scene,
        kurbo::{Point, Size},
    },
    winit::window::CursorIcon,
};

/// Represents a button.
#[derive(Clone, Debug, Default)]
pub struct Button<F, A: ?Sized> {
    state: InteractiveState,

    /// The callback to call when the button is clicked.
    pub on_click: F,
    /// The appearance of the button.
    pub appearance: A,
}

impl<F, A> Button<F, A> {
    /// Creates a new [`Button`] with the provided callback and appearance.
    pub fn new(on_click: F, appearance: A) -> Self
    where
        F: FnMut(&mut A, &ElemContext),
    {
        Self {
            state: InteractiveState::empty(),
            on_click,
            appearance,
        }
    }

    /// Sets whether the button is disabled or not.
    pub fn disabled(mut self, yes: bool) -> Self {
        self.state.set(InteractiveState::DISABLED, yes);
        self
    }

    /// Sets the function that will be called when this [`Button`] is clicked.
    pub fn on_click<F2>(self, on_click: F2) -> Button<F2, A>
    where
        F2: FnMut(&mut A, &ElemContext),
    {
        Button {
            state: self.state,
            on_click,
            appearance: self.appearance,
        }
    }

    /// Sets the appearance of the button.
    pub fn child<A2>(self, appearance: A2) -> Button<F, A2> {
        Button {
            state: self.state,
            on_click: self.on_click,
            appearance,
        }
    }
}

impl<F, A> Element for Button<F, A>
where
    F: FnMut(&mut A, &ElemContext),
    A: InputAppearance,
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
        let interaction = self.state.handle_interactions(&mut self.appearance, event);
        if interaction.entered() {
            elem_context
                .window
                .with_winit_window(|w| w.set_cursor(CursorIcon::Pointer.into()));
        }
        if interaction.left() {
            elem_context
                .window
                .with_winit_window(|w| w.set_cursor(CursorIcon::Default.into()));
        }
        if og_state != self.state {
            self.appearance.state_changed(elem_context, self.state);
        }
        if interaction.clicked() {
            (self.on_click)(&mut self.appearance, elem_context);
        }
        if interaction.event_handled() {
            return EventResult::Handled;
        }
        self.appearance.event(elem_context, event)
    }

    #[inline]
    fn begin(&mut self, elem_context: &ElemContext) {
        self.appearance.begin(elem_context);
        self.appearance.state_changed(elem_context, self.state);
    }
}
