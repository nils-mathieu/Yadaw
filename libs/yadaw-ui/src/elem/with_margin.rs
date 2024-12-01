use {
    crate::{
        elem::Length,
        element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    },
    vello::{
        kurbo::{Point, Size, Vec2},
        Scene,
    },
};

/// An element that adds a margin around its child element.
#[derive(Debug)]
pub struct WithMargin<E: ?Sized> {
    /// The top margin.
    pub top: Length,
    /// The right margin.
    pub right: Length,
    /// The bottom margin.
    pub bottom: Length,
    /// The left margin.
    pub left: Length,

    /// The child element that will have a margin around it.
    pub child: E,
}

impl<E> WithMargin<E> {
    /// Creates a new [`WithMargin`] element.
    pub fn new(child: E) -> Self {
        Self {
            top: Length::ZERO,
            right: Length::ZERO,
            bottom: Length::ZERO,
            left: Length::ZERO,
            child,
        }
    }

    /// Sets the top margin.
    #[inline]
    pub fn with_margin_top(mut self, top: Length) -> Self {
        self.top = top;
        self
    }

    /// Sets the right margin.
    #[inline]
    pub fn with_margin_right(mut self, right: Length) -> Self {
        self.right = right;
        self
    }

    /// Sets the bottom margin.
    #[inline]
    pub fn with_margin_bottom(mut self, bottom: Length) -> Self {
        self.bottom = bottom;
        self
    }

    /// Sets the left margin.
    #[inline]
    pub fn with_margin_left(mut self, left: Length) -> Self {
        self.left = left;
        self
    }

    /// Sets the margin on all sides.
    pub fn with_margin(mut self, margin: Length) -> Self {
        self.top = margin.clone();
        self.right = margin.clone();
        self.bottom = margin.clone();
        self.left = margin;
        self
    }
}

impl<E: ?Sized + Element> Element for WithMargin<E> {
    #[inline]
    fn ready(&mut self, cx: &ElemCtx) {
        self.child.ready(cx);
    }

    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        let top = self.top.resolve(cx);
        let right = self.right.resolve(cx);
        let bottom = self.bottom.resolve(cx);
        let left = self.left.resolve(cx);

        let new_width = size.width().map(|w| (w - left - right).max(0.0));
        let new_height = size.height().map(|h| (h - top - bottom).max(0.0));

        self.child.set_size(cx, SetSize::new(new_width, new_height));
    }

    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        let top = self.top.resolve(cx);
        let left = self.left.resolve(cx);

        self.child.set_position(cx, position + Vec2::new(left, top));
    }

    fn metrics(&mut self, cx: &ElemCtx) -> Metrics {
        let top = self.top.resolve(cx);
        let left = self.left.resolve(cx);
        let right = self.right.resolve(cx);
        let bottom = self.bottom.resolve(cx);

        let child_metrics = self.child.metrics(cx);

        Metrics {
            position: child_metrics.position - Vec2::new(left, top),
            size: child_metrics.size + Size::new(left + right, top + bottom),
            baseline: child_metrics.baseline - bottom,
        }
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
