use {
    crate::{
        ElemContext, Element, LayoutContext,
        event::{Event, EventResult},
    },
    vello::kurbo::{Point, Size},
};

/// A simple element that hooks into the event system with a function.
#[derive(Default, Clone, Debug)]
pub struct HookEvent<F, E: ?Sized> {
    /// The hook function.
    pub on_event: F,
    /// The child element.
    pub child: E,
}

impl<F, E> HookEvent<F, E> {
    /// Creates a new `HookEvent` element.
    #[inline]
    pub fn new(on_event: F, child: E) -> Self
    where
        F: FnMut(&mut E, &ElemContext, &dyn Event) -> EventResult,
    {
        Self { on_event, child }
    }

    /// The hook function of this [`HookEvent`].
    #[inline]
    pub fn on_event<F2>(self, on_event: F2) -> HookEvent<F2, E>
    where
        F2: FnMut(&mut E, &ElemContext, &dyn Event) -> EventResult,
    {
        HookEvent {
            on_event,
            child: self.child,
        }
    }

    /// The child element of this [`HookEvent`].
    #[inline]
    pub fn child<E2>(self, child: E2) -> HookEvent<F, E2> {
        HookEvent {
            on_event: self.on_event,
            child,
        }
    }
}

impl<F, E> Element for HookEvent<F, E>
where
    F: FnMut(&mut E, &ElemContext, &dyn Event) -> EventResult,
    E: Element + ?Sized,
{
    #[inline]
    fn draw(&mut self, elem_context: &ElemContext, scene: &mut vello::Scene) {
        self.child.draw(elem_context, scene);
    }

    #[inline]
    fn begin(&mut self, elem_context: &ElemContext) {
        self.child.begin(elem_context);
    }

    #[inline]
    fn hit_test(&self, point: Point) -> bool {
        self.child.hit_test(point)
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
    fn size_hint(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        space: Size,
    ) -> crate::SizeHint {
        self.child.size_hint(elem_context, layout_context, space)
    }

    fn event(&mut self, elem_context: &ElemContext, event: &dyn Event) -> EventResult {
        if (self.on_event)(&mut self.child, elem_context, event).is_handled() {
            return EventResult::Handled;
        }

        self.child.event(elem_context, event)
    }
}
