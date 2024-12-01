use {
    crate::element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    vello::{kurbo::Point, Scene},
};

/// An element that carries data along with a child.
///
/// The data is ignored and is simply here to be used externally.
pub struct WithData<T, E: ?Sized> {
    /// The data.
    pub data: T,
    /// The child element.
    pub child: E,
}

impl<T, E: ?Sized + Element> Element for WithData<T, E> {
    #[inline]
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        self.child.set_size(cx, size);
    }

    #[inline]
    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.child.set_position(cx, position);
    }

    #[inline]
    fn metrics(&mut self, cx: &ElemCtx) -> Metrics {
        self.child.metrics(cx)
    }

    #[inline]
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        self.child.render(cx, scene)
    }

    #[inline]
    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        self.child.hit_test(cx, point)
    }

    #[inline]
    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        self.child.event(cx, event)
    }
}
