use {
    crate::{Ctx, Window},
    vello::kurbo::{Point, Size},
};

/// Contains information about the layout of an element.
#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutContext {
    /// The size of the parent element.
    ///
    /// Used to compute some layout metrics.
    pub parent: Size,
    /// The scale factor of the element.
    pub scale_factor: f64,
}

/// Represents the size that an element may be.
#[derive(Clone, Copy, Debug, Default)]
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
    /// Calling this function will invalidate any state initialized in the [`place`](Element::place)
    /// function.
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
    /// # Requirements
    ///
    /// This function must be called after the element has been laid out and placed through
    /// [`place`](Element::place).
    #[inline]
    fn hit_test(&self, elem_context: &ElemContext, point: Point) -> bool {
        false
    }

    /// Draws the element to the provided scene.
    ///
    /// # Requirements
    ///
    /// This function must be called after the element has been laid out and placed through
    /// [`place`](Element::place).
    #[inline]
    fn draw(&mut self, elem_context: &ElemContext, scene: &mut vello::Scene) {}

    #[doc(hidden)]
    #[inline]
    fn __private_implementation_detail_do_not_use(&self) -> bool {
        false
    }
}

impl Element for () {}
