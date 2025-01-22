use {
    crate::{
        Ctx, Window,
        event::{Event, EventResult},
    },
    vello::kurbo::{Point, Size},
};

/// Contains information about the layout of an element.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutContext {
    /// The size of the parent element.
    ///
    /// Used to compute some layout metrics.
    pub parent: Size,
    /// The scale factor of the element.
    pub scale_factor: f64,
}

/// Represents the size that an element may be.
#[derive(Clone, Copy, Debug)]
pub struct SizeHint {
    /// The preferred size of the element.
    ///
    /// This "preferred" size should be relative to the `space` parameter of the
    /// [`size_hint`] method.
    ///
    /// [`size_hint`]: Element::size_hint
    pub preferred: Size,
    /// The minimum size of the element.
    ///
    /// This value should be independent of the `space` parameter of the
    /// [`size_hint`] method.
    pub min: Size,
    /// The maximum size of the element.
    ///
    /// This value should be independent of the `space` parameter of the
    /// [`size_hint`] method.
    pub max: Size,
}

impl Default for SizeHint {
    fn default() -> Self {
        Self {
            preferred: Size::ZERO,
            min: Size::ZERO,
            max: Size::new(f64::INFINITY, f64::INFINITY),
        }
    }
}

/// The context passed to the methods of an element.
#[derive(Debug)]
pub struct ElemContext {
    /// The application context.
    pub ctx: Ctx,
    /// The window in which the element is located.
    pub window: Window,
}

/// Represents a single element in the UI.
///
/// UI elements are the building blocks of the UI tree. They can be laid out, drawn, and respond to
/// events from the user.
#[allow(unused_variables)]
pub trait Element {
    /// Computes a hint of the element's preferred size metrics for a given layout context and
    /// constraints.
    ///
    /// # Parameters
    ///
    /// - `elem_context`: Contextual information about the current element. This allows interacting
    ///   with the application context and the window in which the element is located.
    ///
    /// - `layout_context`: Contextual information about the layout of the element. This includes
    ///   the size of the parent element and the scale factor of the element. This is mainly used
    ///   to resolve relative sizes.
    ///
    /// - `space`: The available space for the element. It's possible for the size to report an
    ///   infinite size if there is no constraint on the element size. It's also possible to report
    ///   a null size if the element should attempt to take the least amount of space possible.
    ///
    /// # Remarks
    ///
    /// Calling this function will invalidate any state initialized in the [`place`]
    /// function.
    ///
    /// [`place`] must be called again in order to reset the element's position and size.
    ///
    /// [`place`]: Element::place
    #[inline]
    fn size_hint(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        space: Size,
    ) -> SizeHint {
        SizeHint::default()
    }

    /// Places the element at the provided position and size.
    ///
    /// This function is usually called after the [`size_hint`](Element::size_hint) function, but
    /// it's not always the case.
    ///
    /// # Parameters
    ///
    /// - `elem_context`: Contextual information about the current element. This allows interacting
    ///   with the application context and the window in which the element is located.
    ///
    /// - `layout_context`: Contextual information about the layout of the element. This includes
    ///   the size of the parent element and the scale factor of the element. This is mainly used
    ///   to resolve relative sizes.
    ///
    /// - `pos`: The position of the element.
    ///
    /// - `size`: The size of the element.
    #[inline]
    fn place(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        pos: Point,
        size: Size,
    ) {
    }

    /// Returns whether the provided point is included in the element.
    ///
    /// # Parameters
    ///
    /// - `point`: The point to test.
    ///
    /// # Requirements
    ///
    /// This function must be called after the element has been laid out and placed through
    /// [`place`](Element::place).
    #[inline]
    fn hit_test(&self, point: Point) -> bool {
        false
    }

    /// Draws the element to the provided scene.
    ///
    /// # Requirements
    ///
    /// This function must be called after the element has been laid out and placed through
    /// [`place`](Element::place).
    ///
    /// # Parameters
    ///
    /// - `elem_context`: Contextual information about the current element. This allows interacting
    ///   with the application context and the window in which the element is located.
    ///
    /// - `scene`: The scene to draw the element to.
    #[inline]
    fn draw(&mut self, elem_context: &ElemContext, scene: &mut vello::Scene) {}

    /// Handles an event.
    ///
    /// # Requirements
    ///
    /// This function must be called after the element has been laid out and placed through
    /// [`place`](Element::place).
    ///
    /// # Parameters
    ///
    /// - `elem_context`: Contextual information about the current element. This allows interacting
    ///   with the application context and the window in which the element is located.
    ///
    /// - `event`: The event to handle.
    ///
    /// # Returns
    ///
    /// If the event was handled and should not be propagated further, this function returns
    /// [`EventResult::Handled`]. Otherwise, it returns [`EventResult::Continue`].
    #[inline]
    fn event(&mut self, elem_context: &ElemContext, event: &dyn Event) -> EventResult {
        EventResult::Continue
    }

    /// Called when the element is added to the UI tree.
    fn begin(&mut self, elem_context: &ElemContext) {}

    #[doc(hidden)]
    #[inline]
    fn __private_implementation_detail_do_not_use(&self) -> bool {
        false
    }
}

impl Element for () {}
