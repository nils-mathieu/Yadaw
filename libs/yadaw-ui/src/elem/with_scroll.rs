use {
    crate::{
        element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize, SizeHint},
        event,
    },
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
    /// The size of the lines (when lines are used to scroll).
    pub line_size: f64,

    /// Whether the element should use the user's input to scroll.
    pub user_input: bool,

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
            scroll_x: true,
            scroll_y: true,
            user_input: true,
            scroll_amount: Vec2::ZERO,
            line_size: 10.0,
            child,
        }
    }

    /// Sets whether the element should use the user's input to scroll.
    pub fn with_user_input(mut self, user_input: bool) -> Self {
        self.user_input = user_input;
        self
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

    /// Sets the size of the lines (when lines are used to scroll).
    pub fn with_line_size(mut self, line_size: f64) -> Self {
        self.line_size = line_size;
        self
    }
}

impl<E: ?Sized + Element> Element for WithScroll<E> {
    fn size_hint(&mut self, cx: &ElemCtx) -> SizeHint {
        let mut size_hint = self.child.size_hint(cx);
        if self.scroll_x {
            size_hint.min.width = 0.0;
        }
        if self.scroll_y {
            size_hint.min.height = 0.0;
        }
        size_hint
    }

    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        // let child_hint = self.child.size_hint(cx);

        // let mut child_size = size;
        // if self.scroll_x {
        //     child_size.width = child_hint.max.width;
        // }
        // if self.scroll_y {
        //     child_size.height = child_hint.max.height
        // }

        // self.child.set_size(cx, child_size);
        todo!();
    }

    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.position = position;
        self.child.set_position(cx, position + self.scroll_amount);
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
        if self.user_input && (self.scroll_x || self.scroll_y) {
            if let Some(event) = event.downcast::<event::WheelInput>() {
                let delta = match event.delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        Vec2::new(x as f64, y as f64) * self.line_size
                    }
                    MouseScrollDelta::PixelDelta(delta) => Vec2::new(delta.x, delta.y),
                };

                let child_metrics = self.child.metrics(cx);

                if self.scroll_x && child_metrics.size.width > self.size.width {
                    self.scroll_amount.x = (self.scroll_amount.x + delta.x)
                        .min(0.0)
                        .max(child_metrics.size.width - self.size.width);
                }

                if self.scroll_y && child_metrics.size.height > self.size.height {
                    self.scroll_amount.y = (self.scroll_amount.y + delta.y)
                        .min(0.0)
                        .max(child_metrics.size.height - self.size.height);
                }

                self.child
                    .set_position(cx, self.position + self.scroll_amount);

                cx.window().request_redraw();
            }
        }

        self.child.event(cx, event)
    }
}
