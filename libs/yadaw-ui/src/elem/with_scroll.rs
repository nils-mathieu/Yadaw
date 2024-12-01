use {
    crate::{
        elem::utils::exp_decay,
        element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
        event,
    },
    std::time::Instant,
    vello::{
        kurbo::{Point, Size, Vec2},
        Scene,
    },
    winit::event::MouseScrollDelta,
};

/// An element that allows its content to scroll.
pub struct WithScroll<E: ?Sized> {
    /// The size of the element.
    size: Size,
    /// The position of the element.
    position: Point,

    /// Whether the element should be able to scroll horizontally.
    pub scroll_x: bool,
    /// Whether the element should be able to scroll vertically.
    pub scroll_y: bool,

    /// The amount of scroll in each direction.
    pub scroll_amount: Vec2,

    /// The child element that should be scrollable.
    pub child: E,
}

impl<E> WithScroll<E> {
    /// Creates a new [`WithScroll`] element with the provided child.
    pub fn new(child: E) -> Self {
        Self {
            size: Size::ZERO,
            position: Point::ZERO,
            scroll_x: false,
            scroll_y: false,
            scroll_amount: Vec2::ZERO,
            child,
        }
    }

    /// Sets whether the element should be able to scroll vertically.
    pub fn with_scroll_x(mut self, scroll_x: bool) -> Self {
        self.scroll_x = scroll_x;
        self
    }

    /// Sets whether the element should be able to scroll horizontally.
    pub fn with_scroll_y(mut self, scroll_y: bool) -> Self {
        self.scroll_y = scroll_y;
        self
    }

    /// Sets the child element that should be scrollable.
    pub fn with_controls(self) -> WithScrollAndControls<E> {
        WithScrollAndControls {
            target_scroll_amount: self.scroll_amount,
            smooth_scroll_decay: 10.0,
            line_size: 100.0,
            last_instant: None,
            inner: self,
        }
    }
}

impl<E: ?Sized + Element> WithScroll<E> {
    /// Clamps the scroll amount to the bounds of the child element.
    pub fn set_scroll_amount(&mut self, cx: &ElemCtx, new: Vec2) {
        self.scroll_amount = self.clamp_scroll_amount(new, cx);
        self.child
            .set_position(cx, self.position + self.scroll_amount);
    }

    /// Clamps the provided scroll amount to the bounds of the child element.
    ///
    /// This function does not modify the scroll amount of the element.
    fn clamp_scroll_amount(&mut self, mut scroll_amount: Vec2, cx: &ElemCtx) -> Vec2 {
        let child_metrics = self.child.metrics(cx);

        if self.scroll_x {
            scroll_amount.x = scroll_amount
                .x
                .max(child_metrics.size.width - self.size.width)
                .min(0.0);
        }

        if self.scroll_y {
            scroll_amount.y = scroll_amount
                .y
                .max(self.size.height - child_metrics.size.height)
                .min(0.0);
        }

        scroll_amount
    }
}

impl<E: ?Sized + Element> Element for WithScroll<E> {
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        let mut child_size = size;
        if self.scroll_x {
            child_size = child_size.without_width();
        }
        if self.scroll_y {
            child_size = child_size.without_height();
        }
        self.child.set_size(cx, child_size);
        self.size = size.or_fallback(self.child.metrics(cx).size);

        self.set_scroll_amount(cx, self.scroll_amount);
    }

    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.position = position;
        self.set_scroll_amount(cx, self.scroll_amount);
    }

    #[inline]
    fn metrics(&mut self, _cx: &ElemCtx) -> Metrics {
        Metrics {
            size: self.size,
            position: self.position,
            baseline: 0.0,
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

/// A wrapper around [`WithScroll`] that also adds smooth scrolling and scroll controls.
pub struct WithScrollAndControls<E: ?Sized> {
    /// The amount of smooth scrolling that should be applied.
    pub smooth_scroll_decay: f64,
    /// The size of a single line of scrolling.
    pub line_size: f64,

    /// The target (unsmoothed) scroll offset.
    target_scroll_amount: Vec2,
    /// If the scroll amount is currently being animated, the last instant of the render
    /// callback.
    last_instant: Option<Instant>,

    /// The wrapped scrollable element.
    pub inner: WithScroll<E>,
}

impl<E> WithScrollAndControls<E> {
    /// Sets the amount of smooth scrolling that should be applied.
    pub fn with_smooth_scroll_decay(mut self, decay: f64) -> Self {
        self.smooth_scroll_decay = decay;
        self
    }

    /// Sets the size of a single line of scrolling.
    pub fn with_line_size(mut self, line_size: f64) -> Self {
        self.line_size = line_size;
        self
    }
}

impl<E> Element for WithScrollAndControls<E>
where
    E: ?Sized + Element,
{
    #[inline]
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        self.inner.set_size(cx, size);
    }

    #[inline]
    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.inner.set_position(cx, position);
    }

    #[inline]
    fn metrics(&mut self, cx: &ElemCtx) -> Metrics {
        self.inner.metrics(cx)
    }

    #[inline]
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        if let Some(last_instant) = self.last_instant {
            let dt = cx.app().now().duration_since(last_instant).as_secs_f64();

            let dist = self.target_scroll_amount - self.inner.scroll_amount;
            if dist.x.abs() < 0.5 && dist.y.abs() < 0.5 {
                self.inner.set_scroll_amount(cx, self.target_scroll_amount);
                self.last_instant = None;
            } else {
                self.inner.set_scroll_amount(
                    cx,
                    exp_decay(
                        self.inner.scroll_amount,
                        self.target_scroll_amount,
                        self.smooth_scroll_decay * dt,
                    ),
                );

                self.last_instant = Some(cx.app().now());
            }

            cx.window().request_redraw();
        }

        self.inner.render(cx, scene);
    }

    #[inline]
    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        self.inner.hit_test(cx, point)
    }

    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        if self.inner.event(cx, event).is_handled() {
            return EventResult::Handled;
        }

        if self.inner.scroll_x || self.inner.scroll_y {
            if let Some(event) = event.downcast::<event::WheelInput>() {
                if cx.is_cursor_present()
                    && cx
                        .window()
                        .last_reported_cursor_position()
                        .is_some_and(|pos| self.inner.hit_test(cx, pos))
                {
                    let mut delta = match event.delta {
                        MouseScrollDelta::LineDelta(x, y) => {
                            Vec2::new(x as f64, y as f64) * self.line_size * cx.scale_factor()
                        }
                        MouseScrollDelta::PixelDelta(delta) => Vec2::new(delta.x, delta.y),
                    };

                    if !self.inner.scroll_x {
                        delta.x = 0.0;
                    }
                    if !self.inner.scroll_y {
                        delta.y = 0.0;
                    }

                    self.target_scroll_amount = self
                        .inner
                        .clamp_scroll_amount(self.target_scroll_amount + delta, cx);

                    self.last_instant = Some(cx.app().now());
                    cx.window().request_redraw();
                    return EventResult::Handled;
                }
            }
        }

        EventResult::Ignored
    }
}
