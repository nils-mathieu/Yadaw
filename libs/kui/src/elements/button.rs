use {
    super::interactive::InteractiveState,
    crate::{
        ElemContext, Element, LayoutContext, SizeHint,
        elements::appearance::Appearance,
        event::{Event, EventResult},
    },
    vello::{
        Scene,
        kurbo::{Point, Size},
    },
};

/// Represents a button.
#[derive(Clone, Debug, Default)]
pub struct Button<A: ?Sized> {
    state: InteractiveState,

    /// Whether to act on press.
    ///
    /// Otherwise, the button will act on release.
    pub act_on_press: bool,
    /// The appearance of the button.
    pub appearance: A,
}

impl<A> Button<A> {
    /// Creates a new [`Button`] with the provided callback and appearance.
    pub fn new(appearance: A) -> Self {
        Self {
            act_on_press: false,
            state: InteractiveState::empty(),
            appearance,
        }
    }

    /// Sets whether the button is disabled or not.
    pub fn disabled(mut self, yes: bool) -> Self {
        self.state.set(InteractiveState::DISABLED, yes);
        self
    }

    /// Sets the appearance of the button.
    pub fn child<A2>(self, appearance: A2) -> Button<A2> {
        Button {
            act_on_press: self.act_on_press,
            state: self.state,
            appearance,
        }
    }

    /// Whether to act on press.
    pub fn act_on_press(mut self, yes: bool) -> Self {
        self.act_on_press = yes;
        self
    }
}

impl<A> Element for Button<A>
where
    A: Appearance<()>,
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
        self.state.remove_transient_states();

        let og_state = self.state;
        let mut event_result = self
            .state
            .handle_pointer_interactions(&mut |pt| self.appearance.hit_test(pt), event);
        if (self.act_on_press && self.state.just_pressed())
            || (!self.act_on_press && self.state.just_clicked())
        {
            self.state.insert(InteractiveState::VALUE_CHANGED);
            event_result = EventResult::Handled;
        }
        if og_state != self.state {
            self.appearance.state_changed(elem_context, self.state, &());
        }
        if event_result.is_handled() {
            return EventResult::Handled;
        }
        self.appearance.event(elem_context, event)
    }

    #[inline]
    fn begin(&mut self, elem_context: &ElemContext) {
        self.appearance.begin(elem_context);
        self.appearance.state_changed(elem_context, self.state, &());
    }
}
