use {
    crate::{
        elem::Length,
        element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize, SizeHint},
    },
    vello::{
        kurbo::{Point, Size},
        Scene,
    },
};

/// An [`Element`] that restricts the size of its child.
#[derive(Debug, Clone)]
pub struct WithSize<E: ?Sized> {
    /// The minimum width of the element.
    pub min_width: Length,
    /// The maximum width of the element.
    pub max_width: Length,

    /// The minimum height of the element.
    pub min_height: Length,
    /// The maximum height of the element.
    pub max_height: Length,

    /// The child element.
    pub child: E,
}

impl<E> WithSize<E> {
    /// Creates a new [`WithSize<E>`] element with the provided child.
    pub fn new(child: E) -> Self {
        Self {
            min_width: Length::ZERO,
            max_width: Length::INFINITY,
            min_height: Length::ZERO,
            max_height: Length::INFINITY,
            child,
        }
    }

    /// Sets the minimum width of the element.
    pub fn with_min_width(mut self, min_width: Length) -> Self {
        self.min_width = min_width;
        self
    }

    /// Sets the maximum width of the element.
    pub fn with_max_width(mut self, max_width: Length) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the minimum height of the element.
    pub fn with_min_height(mut self, min_height: Length) -> Self {
        self.min_height = min_height;
        self
    }

    /// Sets the maximum height of the element.
    pub fn with_max_height(mut self, max_height: Length) -> Self {
        self.max_height = max_height;
        self
    }
}

impl<E: ?Sized> WithSize<E> {
    /// Returns the added [`SizeHint`] of the element.
    pub fn added_size_hint(&self, cx: &ElemCtx) -> SizeHint {
        let min_width = self.min_width.resolve(cx);
        let max_width = self.max_width.resolve(cx);
        let min_height = self.min_height.resolve(cx);
        let max_height = self.max_height.resolve(cx);

        SizeHint {
            min: Size::new(min_width, min_height),
            max: Size::new(max_width, max_height),
        }
    }
}

impl<E: ?Sized + Element> Element for WithSize<E> {
    #[inline]
    fn size_hint(&mut self, cx: &ElemCtx) -> SizeHint {
        self.child.size_hint(cx).union(&self.added_size_hint(cx))
    }

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
