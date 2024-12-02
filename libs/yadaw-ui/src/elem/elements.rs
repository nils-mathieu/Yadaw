use {
    crate::element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    core::f64,
    vello::{
        kurbo::{Point, Size},
        Scene,
    },
};

/// A collection of elements.
///
/// This element does not have a specific size, and will always have an infinite size.
pub struct Elements<E> {
    /// The position of the elements/
    pub position: Point,
    /// The children of the canvas.
    pub children: Vec<E>,
}

impl<E> Elements<E> {
    /// Adds a child element to the canvas.
    pub fn with_child(mut self, child: E) -> Self {
        self.children.push(child);
        self
    }
}

impl<E> Default for Elements<E> {
    fn default() -> Self {
        Self {
            position: Point::ZERO,
            children: Vec::new(),
        }
    }
}

impl<E: Element> Element for Elements<E> {
    fn ready(&mut self, cx: &ElemCtx) {
        let child_cx = cx.inherit_parent_size(Size::ZERO);
        self.children.iter_mut().for_each(|c| {
            c.ready(&child_cx);
            c.set_size(&child_cx, SetSize::relaxed());
        });
    }

    fn set_size(&mut self, _cx: &ElemCtx, size: SetSize) {
        assert!(
            size.is_relaxed(),
            "Canvas elements do not support having a specific size",
        );
    }

    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        let child_cx = cx.inherit_parent_size(Size::ZERO);
        self.position = position;
        self.children
            .iter_mut()
            .for_each(|c| c.set_position(&child_cx, position));
    }

    fn metrics(&mut self, _cx: &ElemCtx) -> Metrics {
        Metrics {
            position: self.position,
            size: Size::new(f64::INFINITY, f64::INFINITY),
            baseline: 0.0,
        }
    }

    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        let child_cx = cx.inherit_parent_size(Size::ZERO);
        self.children
            .iter_mut()
            .for_each(|c| c.render(&child_cx, scene));
    }

    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        let child_cx = cx.inherit_parent_size(Size::ZERO);
        self.children
            .iter_mut()
            .any(|c| c.hit_test(&child_cx, point))
    }

    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        let child_cx = cx.inherit_parent_size(Size::ZERO);
        for child in &mut self.children {
            if child.event(&child_cx, event).should_stop_propagation() {
                return EventResult::StopPropagation;
            }
        }

        EventResult::Continue
    }
}
