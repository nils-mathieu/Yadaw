use {
    crate::element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    vello::{
        kurbo::{Point, Vec2},
        Scene,
    },
};

/// Translates the position of the children by the requested amount.
pub struct Translate<E: ?Sized> {
    /// The translation applied to the children.
    pub translation: Vec2,
    /// The element to apply the translation to.
    pub child: E,
}

impl<E: ?Sized + Element> Element for Translate<E> {
    #[inline]
    fn ready(&mut self, cx: &ElemCtx) {
        self.child.ready(cx);
    }

    #[inline]
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        self.child.set_size(cx, size);
    }

    #[inline]
    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.child.set_position(cx, position + self.translation);
    }

    #[inline]
    fn metrics(&mut self, cx: &ElemCtx) -> Metrics {
        self.child.metrics(cx)
    }

    #[inline]
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        self.child.render(cx, scene);
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
