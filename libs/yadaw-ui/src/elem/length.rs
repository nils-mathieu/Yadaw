use crate::element::ElemCtx;

/// Represents a length in a 2D space.
#[derive(Debug, Clone)]
pub enum Length {
    /// A length in unscaled pixels.
    ///
    /// It's generally discouraged to use pixel lengths in UIs, as they might not look the same
    /// on different devices.
    UnscaledPixels(f64),

    /// A length in (scaled) pixels.
    ///
    /// This is the most common unit of length. Pixel lengths are scaled based on the DPI of the
    /// display, which makes them look roughly the same on different devices. Or at least, they
    /// follow the user's preferences for text size and other UI elements.
    Pixels(f64),

    /// A fraction of the parent's width.
    ///
    /// This is a relative length that will be calculated based on the parent's available space
    /// after all fixed lengths have been calculated.
    ParentWidth(f64),

    /// A fraction of the parent's height.
    ///
    /// This is a relative length that will be calculated based on the parent's available space
    /// after all fixed lengths have been calculated.
    ParentHeight(f64),

    /// A fraction of the parent's minimum side (width or height).
    ///
    /// This is a relative length that will be calculated based on the parent's available space
    /// after all fixed lengths have been calculated.
    ParentMin(f64),

    /// A fraction of the parent's maximum side (width or height).
    ///
    /// This is a relative length that will be calculated based on the parent's available space
    /// after all fixed lengths have been calculated.
    ParentMax(f64),
}

impl Length {
    /// A length of zero.
    pub const ZERO: Self = Self::UnscaledPixels(0.0);

    /// A infinite length.
    pub const INFINITY: Self = Self::UnscaledPixels(f64::INFINITY);

    /// Resolves the [`Length`] to a pixel value based on the provided [`ElemCtx`].
    pub fn resolve(&self, cx: &ElemCtx) -> f64 {
        match self {
            Length::UnscaledPixels(v) => *v,
            Length::Pixels(v) => *v * cx.scale_factor(),
            Length::ParentWidth(v) => *v * cx.parent_size().width,
            Length::ParentHeight(v) => *v * cx.parent_size().height,
            Length::ParentMin(v) => *v * cx.parent_size().min_side(),
            Length::ParentMax(v) => *v * cx.parent_size().max_side(),
        }
    }
}

impl Default for Length {
    #[inline]
    fn default() -> Self {
        Length::ZERO
    }
}
