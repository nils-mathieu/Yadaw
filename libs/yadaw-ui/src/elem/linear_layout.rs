use vello::kurbo::Vec2;

/// A direction in which elements can be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    /// Elements are placed horizontally.
    Horizontal,
    /// Elements are placed vertically.
    Vertical,
}

impl Direction {
    /// Returns a vector representing this direction (positive x for horizontal, positive y for
    /// vertical).
    pub const fn to_vec2(self) -> Vec2 {
        match self {
            Self::Horizontal => Vec2::new(1.0, 0.0),
            Self::Vertical => Vec2::new(0.0, 1.0),
        }
    }
}
