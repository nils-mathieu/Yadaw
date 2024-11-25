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
}
