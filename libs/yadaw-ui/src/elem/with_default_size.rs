use {
    crate::{
        elem::Length,
        element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    },
    vello::{kurbo::Point, Scene},
};

/// An element that constrains the size of its child element.
pub struct WithDefaultSize<E: ?Sized> {
    /// The new width of the child element.
    pub default_width: Option<Length>,
    /// The new height of the child element.
    pub default_height: Option<Length>,

    /// The child element.
    pub child: E,
}

impl<E> WithDefaultSize<E> {
    /// Creates a new [`WithDefaultSize`] element with the provided child.
    pub fn new(child: E) -> Self {
        Self {
            default_width: None,
            default_height: None,
            child,
        }
    }

    /// Sets the new width of the child element.
    pub fn with_default_width(mut self, width: Length) -> Self {
        self.default_width = Some(width);
        self
    }

    /// Sets the new height of the child element.
    pub fn with_default_height(mut self, height: Length) -> Self {
        self.default_height = Some(height);
        self
    }
}

impl<E: ?Sized + Element> Element for WithDefaultSize<E> {
    #[inline]
    fn ready(&mut self, cx: &ElemCtx) {
        self.child.ready(cx);
    }

    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        let new_width = self.default_width.as_ref().map(|width| width.resolve(cx));
        let new_height = self
            .default_height
            .as_ref()
            .map(|height| height.resolve(cx));
        let new_size = SetSize::new(new_width, new_height);
        self.child.set_size(cx, size.or(new_size));
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
