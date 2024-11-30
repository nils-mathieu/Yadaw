//! Defines the [`Element`] trait and related types.

mod set_size;
pub use self::set_size::*;

mod elem_ctx;
pub use self::elem_ctx::*;

mod metrics;
pub use self::metrics::*;

mod event;
pub use self::event::*;

use vello::{kurbo::Point, Scene};

/// Represents an element that can be rendered to the screen.
pub trait Element {
    /// Sets the size of the element.
    ///
    /// After this function has been called, the element should be ready to be rendered at
    /// the position `(0, 0)`. If the position need to change, the [`set_position`] method
    /// should be used.
    ///
    /// # Parameters
    ///
    /// * `cx`: The context that is passed along to element methods.
    ///
    /// * `size`: The size of the element. This size *should* be within the bounds returned by
    ///   the [`size_hint`] method. Note that this is not enforced by the framework, so it is
    ///   up to the implementor to handle invalid sizes properly.
    ///
    /// [`set_position`]: Element::set_position
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize);

    /// Sets the position of the element.
    ///
    /// This method is called by the layout system to position the element within its parent
    /// after the size has been set.
    ///
    /// This function should be called *after* [`set_size`] has been called.
    ///
    /// # Parameters
    ///
    /// * `cx`: The context that is passed along to element methods.
    ///
    /// * `position`: The position of the element. This position is absolute and is expressed in
    ///   screen coordinates.
    fn set_position(&mut self, cx: &ElemCtx, position: Point);

    /// Requests information about the metrics of the element.
    ///
    /// This method is called by the layout system to determine how to size the element.
    ///
    /// # Remarks
    ///
    /// This function assumes that the [`set_size`] and [`set_position`] methods have been called
    /// previously.
    ///
    /// [`set_size`]: Element::set_size
    /// [`set_position`]: Element::set_position
    ///
    /// # Parameters
    ///
    /// * `cx`: The context that is passed along to element methods.
    fn metrics(&mut self, cx: &ElemCtx) -> Metrics;

    /// Renders the element to the screen.
    ///
    /// # Remarks
    ///
    /// This function assumes that the [`set_size`] and [`set_position`] methods have been called
    /// previously.
    ///
    /// [`set_size`]: Element::set_size
    /// [`set_position`]: Element::set_position
    ///
    /// # Parameters
    ///
    /// * `cx`: The context that is passed along to element methods.
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene);

    /// Determines whether the provided point is within the bounds of the element.
    ///
    /// # Remarks
    ///
    /// This function assumes that the [`set_size`] and [`set_position`] methods have been called
    /// previously.
    ///
    /// [`set_size`]: Element::set_size
    /// [`set_position`]: Element::set_position
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
    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool;

    /// Handles an event that occurred on the element.
    ///
    /// # Remarks
    ///
    /// This function assumes that the [`set_size`] and [`set_position`] methods have been called
    /// previously.
    ///
    /// [`set_size`]: Element::set_size
    /// [`set_position`]: Element::set_position
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
    fn set_size(&mut self, _cx: &ElemCtx, _size: SetSize) {}

    #[inline]
    fn set_position(&mut self, _cx: &ElemCtx, _position: Point) {}

    #[inline]
    fn metrics(&mut self, _cx: &ElemCtx) -> Metrics {
        Metrics::EMPTY
    }

    #[inline]
    fn render(&mut self, _cx: &ElemCtx, _scene: &mut Scene) {}

    #[inline]
    fn hit_test(&mut self, _cx: &ElemCtx, _point: Point) -> bool {
        false
    }

    #[inline]
    fn event(&mut self, _cx: &ElemCtx, _event: &dyn Event) -> EventResult {
        EventResult::Ignored
    }
}

impl<E: ?Sized + Element> Element for Box<E> {
    #[inline]
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        (**self).set_size(cx, size)
    }

    #[inline]
    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        (**self).set_position(cx, position)
    }

    #[inline]
    fn metrics(&mut self, cx: &ElemCtx) -> Metrics {
        (**self).metrics(cx)
    }

    #[inline]
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        (**self).render(cx, scene)
    }

    #[inline]
    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        (**self).hit_test(cx, point)
    }

    #[inline]
    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        (**self).event(cx, event)
    }
}
