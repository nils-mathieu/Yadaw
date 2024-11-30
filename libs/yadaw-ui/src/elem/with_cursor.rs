use {
    crate::{
        element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
        event, LiveCursor,
    },
    vello::{kurbo::Point, Scene},
    winit::window::Cursor,
};

/// A wrapper around an element that changes the cursor under the mouse when the element
/// is hovered.
pub struct WithCursor<E: ?Sized> {
    /// Whether the element is currently hovered.
    hovered: bool,
    /// The cursor to use when the element is hovered.
    live_cursor: LiveCursor,

    /// The cursor to use when the element is hovered.
    cursor: Cursor,
    /// The child element.
    pub child: E,
}

impl<E> WithCursor<E> {
    /// Creates a new [`WithCursor`] element with the provided child.
    pub fn new(child: E) -> Self {
        Self {
            hovered: false,
            live_cursor: LiveCursor::default(),
            cursor: Cursor::default(),
            child,
        }
    }

    /// Sets the cursor to use when the element is hovered.
    pub fn with_cursor(mut self, cursor: impl Into<Cursor>) -> Self {
        self.cursor = cursor.into();
        self
    }
}

impl<E: ?Sized + Element> Element for WithCursor<E> {
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
        self.child.render(cx, scene);
    }

    #[inline]
    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        self.child.hit_test(cx, point)
    }

    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        if let Some(event) = event.downcast::<event::CursorMoved>() {
            let now_hovered = self.child.hit_test(cx, event.position);
            if now_hovered != self.hovered {
                self.hovered = now_hovered;
                if now_hovered {
                    self.live_cursor = cx.window().push_cursor(self.cursor.clone());
                } else {
                    cx.window().pop_cursor(self.live_cursor);
                }
            }
        }

        self.child.event(cx, event)
    }
}
