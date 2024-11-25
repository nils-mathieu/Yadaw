//! Defines the [`Element`] trait and related types.

mod size_hint;
pub use self::size_hint::*;

mod elem_ctx;
pub use self::elem_ctx::*;

mod metrics;
pub use self::metrics::*;

mod event;
pub use self::event::*;

use vello::{
    kurbo::{Point, Rect},
    Scene,
};

/// Represents an element that can be rendered to the screen.
pub trait Element {
    /// Requests information about the sizing constraints of the element.
    ///
    /// This method is called by the layout system to determine how to size the element
    /// within its parent.
    ///
    /// # Parameters
    ///
    /// * `cx`: The context that is passed along to element methods.
    fn size_hint(&mut self, cx: &ElemCtx) -> SizeHint;

    /// Places the element within the specified bounds.
    ///
    /// After this function has been called, the element should be ready to be rendered.
    /// Specifically, the [`metrics`], [`render`], [`hit_test`] and [`event`] methods should be
    /// able to be called without causing issues.
    ///
    /// # Parameters
    ///
    /// * `cx`: The context that is passed along to element methods.
    ///
    /// * `bounds`: The bounds within which the element is placed. This rectangle *should* be
    ///   within the bounds returned by the [`size_hint`] method. Note that this is not
    ///   enforced by the framework, so it is up to the implementor to handle invalid bounds
    ///   properly.
    ///
    /// [`size_hint`]: Element::size_hint
    /// [`metrics`]: Element::metrics
    /// [`render`]: Element::render
    /// [`hit_test`]: Element::hit_test
    /// [`event`]: Element::event
    fn place(&mut self, cx: &ElemCtx, bounds: Rect);

    /// Requests information about the metrics of the element.
    ///
    /// This method is called by the layout system to determine how to size the element.
    ///
    /// # Parameters
    ///
    /// * `cx`: The context that is passed along to element methods.
    fn metrics(&self, cx: &ElemCtx) -> Metrics;

    /// Renders the element to the screen.
    ///
    /// # Parameters
    ///
    /// * `cx`: The context that is passed along to element methods.
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene);

    /// Determines whether the provided point is within the bounds of the element.
    ///
    /// # Parameters
    ///
    /// * `cx`: The context that is passed along to element methods.
    ///
    /// * `point`: The point to test. This position is absolute and is expressed in screen
    ///   coordinates.
    ///
    /// # Returns
    ///
    /// Whether the point is within the bounds of the element.
    fn hit_test(&self, cx: &ElemCtx, point: Point) -> bool;

    /// Handles an event that occurred on the element.
    ///
    /// # Parameters
    ///
    /// * `cx`: The context that is passed along to element methods.
    ///
    /// * `event`: The event that occurred.
    ///
    /// # Returns
    ///
    /// The result of the event handling.
    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult;
}

impl Element for () {
    #[inline]
    fn size_hint(&mut self, _cx: &ElemCtx) -> SizeHint {
        SizeHint::EMPTY
    }

    #[inline]
    fn place(&mut self, _cx: &ElemCtx, _bounds: Rect) {}

    #[inline]
    fn metrics(&self, _cx: &ElemCtx) -> Metrics {
        Metrics::EMPTY
    }

    #[inline]
    fn render(&mut self, _cx: &ElemCtx, _scene: &mut Scene) {}

    #[inline]
    fn hit_test(&self, _cx: &ElemCtx, _point: Point) -> bool {
        false
    }

    #[inline]
    fn event(&mut self, _cx: &ElemCtx, _event: &dyn Event) -> EventResult {
        EventResult::Ignored
    }
}

impl<E: ?Sized + Element> Element for Box<E> {
    #[inline]
    fn size_hint(&mut self, cx: &ElemCtx) -> SizeHint {
        (**self).size_hint(cx)
    }

    #[inline]
    fn place(&mut self, cx: &ElemCtx, bounds: Rect) {
        (**self).place(cx, bounds)
    }

    #[inline]
    fn metrics(&self, cx: &ElemCtx) -> Metrics {
        (**self).metrics(cx)
    }

    #[inline]
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        (**self).render(cx, scene)
    }

    #[inline]
    fn hit_test(&self, cx: &ElemCtx, point: Point) -> bool {
        (**self).hit_test(cx, point)
    }

    #[inline]
    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        (**self).event(cx, event)
    }
}
