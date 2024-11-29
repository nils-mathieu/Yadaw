use vello::kurbo::{Point, Size};

/// The metrics associated with an element.
#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    /// The position of the element.
    ///
    /// This is specifically the top-left corner of the element.
    pub position: Point,
    /// The size of the element.
    pub size: Size,
    /// The baseline of the element, measured from the bottom of the element.
    ///
    /// This is used to align text elements with each other.
    pub baseline: f64,
}

impl Metrics {
    /// An empty metrics object.
    pub const EMPTY: Self = Self {
        position: Point::ZERO,
        size: Size::ZERO,
        baseline: 0.0,
    };
}
