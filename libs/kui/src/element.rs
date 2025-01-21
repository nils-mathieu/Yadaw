use {
    crate::{Ctx, Window},
    vello::kurbo::{Point, Size},
};

/// Represents the constrains of a size.
#[derive(Clone, Copy)]
pub struct SizeConstraint {
    width: f64,
    height: f64,
}

impl SizeConstraint {
    /// Creates a new [`SizeConstraint`] from the provided width and height.
    ///
    /// If either components are negative, it means that that component is not constrained.
    pub const fn from_raw(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Creates a new [`SizeConstraint`] from the provided width and height.
    pub const fn new(width: Option<f64>, height: Option<f64>) -> Self {
        debug_assert!(width.is_none() || width.unwrap().is_sign_positive());
        debug_assert!(height.is_none() || height.unwrap().is_sign_positive());
        Self {
            width: match width {
                Some(width) => width,
                None => -1.0,
            },
            height: match height {
                Some(height) => height,
                None => -1.0,
            },
        }
    }

    /// Returns a new [`SizeConstraint`] from the provided size.
    pub const fn from_size(size: Size) -> Self {
        debug_assert!(size.width.is_sign_positive());
        debug_assert!(size.height.is_sign_positive());
        Self {
            width: size.width,
            height: size.height,
        }
    }

    /// Creates a new [`SizeConstraint`] from the provided width and height.
    pub const fn from_constraints(width: f64, height: f64) -> Self {
        debug_assert!(width.is_sign_positive());
        debug_assert!(height.is_sign_positive());
        Self { width, height }
    }

    /// Creates a new unconstrained [`SizeConstraint`].
    pub const fn unconstrained() -> Self {
        Self {
            width: -1.0,
            height: -1.0,
        }
    }

    /// Creates a new [`SizeConstraint`] with the provided width and an unconstrained height.
    pub const fn from_width(width: f64) -> Self {
        debug_assert!(width.is_sign_positive());
        Self {
            width,
            height: -1.0,
        }
    }

    /// Creates a new [`SizeConstraint`] with the provided height and an unconstrained width.
    pub const fn from_height(height: f64) -> Self {
        debug_assert!(height.is_sign_positive());
        Self {
            width: -1.0,
            height,
        }
    }

    /// Returns the maximum width available.
    ///
    /// If no width constraint is provided (the element is free to use any width), this function
    /// returns `None`.
    pub const fn width(&self) -> Option<f64> {
        if self.has_width_constraint() {
            Some(self.width)
        } else {
            None
        }
    }

    /// Returns whether the element has a width constraint.
    pub const fn has_width_constraint(&self) -> bool {
        self.width.is_sign_positive()
    }

    /// Returns the maximum height available.
    ///
    /// If no height constraint is provided (the element is free to use any height), this function
    /// returns `None`.
    pub const fn height(&self) -> Option<f64> {
        if self.has_height_constraint() {
            Some(self.height)
        } else {
            None
        }
    }

    /// Returns whether the element has a height constraint.
    pub const fn has_height_constraint(&self) -> bool {
        self.height.is_sign_positive()
    }

    /// Returns a [`SizeConstraint`] with the provided additional width constraint.
    pub const fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Returns a [`SizeConstraint`] with the provided additional height constraint.
    pub const fn with_height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// Returns a [`SizeConstraint`] with the provided additional width constraint. Unconstrained
    /// sides are set to zero.
    pub fn size_or_zero(&self) -> Size {
        Size::new(self.width.max(0.0), self.height.max(0.0))
    }
}

impl std::fmt::Debug for SizeConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SizeConstraint")
            .field(
                "width",
                if self.has_height_constraint() {
                    &self.width
                } else {
                    &"unconstrained"
                },
            )
            .field(
                "height",
                if self.has_height_constraint() {
                    &self.height
                } else {
                    &"unconstrained"
                },
            )
            .finish()
    }
}

impl Default for SizeConstraint {
    #[inline]
    fn default() -> Self {
        Self::unconstrained()
    }
}

/// The amount of space availble for an element.
#[derive(Clone, Debug, Default)]
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
#[derive(Debug, Default, Clone)]
pub struct ElementMetrics {
    /// The position of the element.
    pub position: Point,
    /// The size of the element.
    pub size: Size,
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
    fn layout(&mut self, elem_context: &ElemContext, info: LayoutInfo) {}

    /// Places the element at the provided position.
    ///
    /// # Requirements
    ///
    /// This function should generally be called after the element has been properly laid
    /// out through the [`layout`](Element::layout) function.
    #[inline]
    fn place(&mut self, elem_context: &ElemContext, pos: Point) {}

    /// Returns the metrics of the element.
    ///
    /// # Requirements
    ///
    /// This function only returns valid values when the element has been laid out by calling
    /// [`layout`](Element::layout).
    ///
    /// Additionally, the `position` field of the returned metrics will not be valid until the
    /// [`place`] function has been called. It should always return the same value as whatever
    /// [`place`] would have set it to.
    ///
    /// [`place`]: Element::place
    #[inline]
    fn metrics(&self) -> ElementMetrics {
        ElementMetrics::default()
    }

    /// Moves the focus to the next element in the focus order.
    #[inline]
    fn move_focus(&mut self, elem_context: &ElemContext, dir: FocusDirection) -> FocusResult {
        FocusResult::Ignored
    }

    /// Returns whether the provided point is included in the element.
    ///
    /// # Requirements
    ///
    /// This function must be called after the element has been laid out and placed.
    #[inline]
    fn hit_test(&self, elem_context: &ElemContext, point: Point) -> bool {
        false
    }

    /// Draws the element to the provided scene.
    ///
    /// # Requirements
    ///
    /// This function must be called after the element has been laid out and placed.
    #[inline]
    fn draw(&mut self, elem_context: &ElemContext, scene: &mut vello::Scene) {}

    #[doc(hidden)]
    #[inline]
    fn __private_implementation_detail_do_not_use(&self) -> bool {
        false
    }
}

impl Element for () {}
