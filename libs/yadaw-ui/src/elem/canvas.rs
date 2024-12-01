use {
    crate::element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    core::f64,
    vello::{
        kurbo::{Point, Size, Vec2},
        Scene,
    },
};

/// The state of a child living in a [`Canvas`].
pub struct Child<E: ?Sized> {
    /// The position of the child relative to its canvas parent.
    position: Vec2,

    /// Whether the position of the child element has changed.
    position_dirty: bool,

    /// The concrete element.
    pub element: E,
}

impl<E> Child<E> {
    /// Creates a new [`Child<E>`] with the provided element.
    pub fn new(element: E) -> Self {
        Self {
            position: Vec2::ZERO,
            position_dirty: true,
            element,
        }
    }

    /// Sets the position of the child element relative to the canvas.
    #[inline]
    pub fn with_position(mut self, pos: Vec2) -> Self {
        self.position = pos;
        self
    }
}

impl<E: ?Sized + Element> Child<E> {
    /// Calls the [`set_size`] and [`set_position`] on the child.
    ///
    /// # Remarks
    ///
    /// This function does not check the `dirty` flag. But it does clear it.
    fn rebuild(&mut self, cx: &ElemCtx, origin: Point) {
        if self.position_dirty {
            self.element.set_position(cx, origin + self.position);
        }

        self.position_dirty = false;
    }
}

impl<E> From<E> for Child<E> {
    #[inline]
    fn from(value: E) -> Self {
        Self::new(value)
    }
}

/// A canvas layout that lays its content using absolute position and sizes.
pub struct Canvas<E> {
    /// The origin of the canvas.
    origin: Point,
    /// Indicates that all children should be considered dirty.
    origin_changed: bool,

    /// The children of the canvas.
    pub children: Vec<Child<E>>,
}

impl<E> Canvas<E> {
    /// Adds a child element to the canvas.
    pub fn with_child(mut self, child: impl Into<Child<E>>) -> Self {
        self.children.push(child.into());
        self
    }
}

impl<E: Element> Canvas<E> {
    /// Rebuilds all children if needed.
    fn rebuild_children(&mut self, cx: &ElemCtx) {
        if self.origin_changed {
            for child in &mut self.children {
                child.position_dirty = true;
                child.rebuild(cx, self.origin);
            }

            self.origin_changed = false;
        } else {
            for child in &mut self.children {
                if child.position_dirty {
                    child.rebuild(cx, self.origin);
                }
            }
        }
    }
}

impl<E> Default for Canvas<E> {
    fn default() -> Self {
        Self {
            origin_changed: true,
            origin: Point::ZERO,
            children: Vec::new(),
        }
    }
}

impl<E: Element> Element for Canvas<E> {
    fn ready(&mut self, cx: &ElemCtx) {
        for child in &mut self.children {
            child.element.ready(cx);
            child.element.set_size(cx, SetSize::relaxed());
        }
    }

    fn set_size(&mut self, _cx: &ElemCtx, size: SetSize) {
        assert!(
            size.is_relaxed(),
            "Canvas elements do not support having a specific size",
        );
    }

    fn set_position(&mut self, _cx: &ElemCtx, position: Point) {
        self.origin = position;
        self.origin_changed = true;
    }

    fn metrics(&mut self, _cx: &ElemCtx) -> Metrics {
        Metrics {
            position: self.origin,
            size: Size::new(f64::INFINITY, f64::INFINITY),
            baseline: 0.0,
        }
    }

    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        let child_cx = cx.inherit_parent_size(Size::ZERO);
        self.rebuild_children(&child_cx);
        self.children
            .iter_mut()
            .for_each(|c| c.element.render(&child_cx, scene));
    }

    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        let child_cx = cx.inherit_parent_size(Size::ZERO);
        self.rebuild_children(&child_cx);
        self.children
            .iter_mut()
            .any(|c| c.element.hit_test(&child_cx, point))
    }

    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        let child_cx = cx.inherit_parent_size(Size::ZERO);
        self.rebuild_children(&child_cx);
        for child in &mut self.children {
            if child
                .element
                .event(&child_cx, event)
                .should_stop_propagation()
            {
                return EventResult::StopPropagation;
            }
        }

        EventResult::Continue
    }
}
