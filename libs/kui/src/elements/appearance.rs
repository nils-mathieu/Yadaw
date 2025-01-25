use {
    crate::{
        ElemContext, Element, LayoutContext, SizeHint,
        elements::interactive::InteractiveState,
        event::{Event, EventResult},
    },
    vello::kurbo::{Point, Size},
};

/// Represents the appearance of an input element.
pub trait Appearance<T: ?Sized>: Element {
    /// The state of the input element has changed.
    fn state_changed(&mut self, cx: &ElemContext, state: InteractiveState, payload: &T);
}

impl<T: ?Sized> Appearance<T> for () {
    fn state_changed(&mut self, _cx: &ElemContext, _state: InteractiveState, _payload: &T) {}
}

/// A [`InputAppearance`] implementation that uses a function to control the appearance of
/// the element.
///
/// Instances of this type are created through [`make_appearance`].
pub struct AppearanceFn<F, E: ?Sized> {
    /// The function itself.
    pub state_changed: F,
    /// The child element.
    pub child: E,
}

impl<F, E> Element for AppearanceFn<F, E>
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

impl<F, E, T> Appearance<T> for AppearanceFn<F, E>
where
    T: ?Sized,
    F: FnMut(&mut E, &ElemContext, InteractiveState, &T),
    E: ?Sized + Element,
{
    #[inline]
    fn state_changed(&mut self, cx: &ElemContext, state: InteractiveState, payload: &T) {
        (self.state_changed)(&mut self.child, cx, state, payload);
    }
}

/// Creates a new [`InputElementAppearance`] instance from the provided function,
pub fn make_appearance<F, E, T>(elem: E, state_changed: F) -> AppearanceFn<F, E>
where
    T: ?Sized,
    E: Element,
    F: FnMut(&mut E, &ElemContext, InteractiveState, &T),
{
    AppearanceFn {
        state_changed,
        child: elem,
    }
}
