use vello::kurbo::Rect;

/// The metrics associated with an element.
#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    /// The rectangle that the element occupies.
    pub rect: Rect,
    /// The baseline of the element, measured from the bottom of the element.
    ///
    /// This is used to align text elements with each other.
    pub baseline: f64,
}

impl Metrics {
    /// An empty metrics object.
    pub const EMPTY: Self = Self {
        rect: Rect::ZERO,
        baseline: 0.0,
    };
}
