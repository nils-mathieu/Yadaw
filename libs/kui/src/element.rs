use {
    crate::{Ctx, Window},
    vello::kurbo::{Point, Rect, Size},
};

pub enum SizeConstraint {
    /// The element is free to use as much space as it wants.
    None,

    /// The width of the element is constrained to the provided maximum width.
    Width(f64),

    /// The height of the element is constrained to the provided maximum height.
    Height(f64),

    /// The width and height of the element are constrained to the provided maximum size.
    Size(Size),
}

/// The amount of space availble for an element.
pub struct LayoutInfo {
    /// The size of the parent element.
    pub parent: Size,

    /// The available size for the element.
    ///
    /// The element is free to use all of this space, or only part of it.
    pub available: SizeConstraint,

    /// The scale factor of the element.
    pub scale_factor: f64,
}

/// The computed metrics of a element.
#[derive(Debug, Default)]
pub struct ElementMetrics {
    /// The rectangle that the element occupies.
    pub rect: Rect,
}

/// The result of navigating the focus out of an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FocusResult {
    /// The keyboard focus was moved out of the element's subtree.
    Exit,
    /// The keyboard focus was moved to the next element in the subtree.
    Continue,
    /// The keyboard focus is not handled by this element.
    Ignored,
}

/// A direction the keyboard focus can be moved in.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FocusDirection {
    /// Move the focus forward.
    ///
    /// Usually controlled by the <kbd>Tab</kbd> key.
    #[default]
    Forward,
    /// Move the focus backward.
    ///
    /// Usually controlled by the <kbd>Shift</kbd> + <kbd>Tab</kbd> keys.
    Backward,
    /// Move the focus up.
    ///
    /// Usually controlled by the <kbd>Up</kbd> key.
    Up,
    /// Move the focus down.
    ///
    /// Usually controlled by the <kbd>Down</kbd> key.
    Down,
    /// Move the focus left.
    ///
    /// Usually controlled by the <kbd>Left</kbd> key.
    Left,
    /// Move the focus right.
    ///
    /// Usually controlled by the <kbd>Right</kbd> key.
    Right,
}

/// The context passed to the methods of an element.
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
    /// Lays the element out, given the provided context.
    ///
    /// # Consequences
    ///
    /// Calling this function may reset the element's position.
    #[inline]
    fn layout(&mut self, info: LayoutInfo) {}

    /// Returns the metrics of the element.
    ///
    /// # Requirements
    ///
    /// This function only returns valid values when the element has been laid out by calling
    /// [`layout`](Element::layout).
    #[inline]
    fn metrics(&self) -> ElementMetrics {
        ElementMetrics::default()
    }

    /// Places the element at the provided position.
    ///
    /// # Requirements
    ///
    /// This function should generally be called after the element has been properly laid
    /// out through the [`layout`](Element::layout) function.
    #[inline]
    fn place(&mut self, pos: Point) {}

    /// Moves the focus to the next element in the focus order.
    #[inline]
    fn move_focus(&mut self, dir: FocusDirection) -> FocusResult {
        FocusResult::Ignored
    }

    /// Returns whether the provided point is included in the element.
    ///
    /// # Requirements
    ///
    /// This function must be called after the element has been laid out and placed.
    #[inline]
    fn hit_test(&self, point: Point) -> bool {
        false
    }

    /// Draws the element to the provided scene.
    ///
    /// # Requirements
    ///
    /// This function must be called after the element has been laid out and placed.
    #[inline]
    fn draw(&mut self, scene: &mut vello::Scene) {}

    #[doc(hidden)]
    #[inline]
    fn __private_implementation_detail_do_not_use(&self) -> bool {
        false
    }
}

impl Element for () {}
