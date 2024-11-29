use vello::kurbo::Size;

/// Represents a hint for the size of an element.
#[derive(Debug, Clone, Copy)]
pub struct SizeHint {
    /// The minimum size of the element.
    ///
    /// Attempting to provide a size smaller than this will result in the element rendering
    /// incorrectly.
    pub min: Size,
    /// The maximum size of the element.
    ///
    /// Attempting to provide a size larger than this will result in the element rendering
    /// incorrectly.
    pub max: Size,
}

impl SizeHint {
    /// A size hint that allows any size.
    pub const ANY: Self = Self {
        min: Size::ZERO,
        max: Size::new(f64::INFINITY, f64::INFINITY),
    };

    /// A size hint that requires a specific size.
    pub const EMPTY: Self = Self {
        min: Size::ZERO,
        max: Size::ZERO,
    };

    /// Returns the [`SizeHint`] that includes both the constraints of `self` and `other`.
    pub fn union(&self, other: &Self) -> Self {
        let min_x = self.min.width.max(other.min.width);
        let min_y = self.min.height.max(other.min.height);
        let max_x = self.max.width.min(other.max.width);
        let max_y = self.max.height.min(other.max.height);

        Self {
            min: Size::new(min_x, min_y),
            max: Size::new(max_x, max_y),
        }
    }
}

/// A way to set the size of an element.
#[derive(Default, Debug, Clone, Copy)]
pub enum SetSize {
    /// A specific size is requested for the element.
    Specific(Size),
    /// A specific width is requested for the element. The height is derived from the width.
    Width(f64),
    /// A specific height is requested for the element. The width is derived from the height.
    Height(f64),
    /// The element is expected to provide a size of its own.
    #[default]
    Unconstrained,
}

impl SetSize {
    /// Returns the size stored in this [`SetSize`] instance, but falls back to `fallback` if the
    /// size is not specific.
    pub fn fallback(self, fallback: Size) -> Size {
        match self {
            Self::Specific(size) => size,
            Self::Width(width) => Size::new(width, fallback.height),
            Self::Height(height) => Size::new(fallback.width, height),
            Self::Unconstrained => fallback,
        }
    }
}
