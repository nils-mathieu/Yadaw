use {crate::LayoutInfo, std::fmt::Debug};

/// Represents a length.
#[derive(Clone)]
pub enum Length {
    /// A length in *unscaled* pixels.
    ///
    /// Unscaled pixels do not take the scale factor of the window into account. One unscaled
    /// pixel is always equal to one pixel in the window's client area.
    UnscaledPixels(f64),
    /// A length in *scaled* pixels.
    ///
    /// Scaled pixels take the scale factor of the window into account.
    Pixels(f64),

    /// A fraction of the parent element's width.
    ParentWidth(f64),
    /// A fraction of the parent element's height.
    ParentHeight(f64),

    /// Computes the length using a runtime function.
    Compute(Box<dyn LengthCalculation>),
}

impl Length {
    /// A length of zero.
    pub const ZERO: Self = Self::UnscaledPixels(0.0);
    /// A infinite length.
    pub const INFINITY: Self = Self::UnscaledPixels(f64::INFINITY);

    /// Resolves the length to a concrete value in unscaled pixels.
    pub fn resolve(&self, info: &LayoutInfo) -> f64 {
        match self {
            Length::UnscaledPixels(pixels) => *pixels,
            Length::Pixels(pixels) => pixels * info.scale_factor,
            Length::ParentWidth(fraction) => info.parent.width * fraction,
            Length::ParentHeight(fraction) => info.parent.height * fraction,
            Length::Compute(f) => f.resolve(info),
        }
    }
}

impl Default for Length {
    fn default() -> Self {
        Length::ZERO
    }
}

impl Debug for Length {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Length::UnscaledPixels(pixels) => write!(f, "{}upx", pixels),
            Length::Pixels(pixels) => write!(f, "{}px", pixels),
            Length::ParentWidth(fraction) => write!(f, "{}%", fraction * 100.0),
            Length::ParentHeight(fraction) => write!(f, "{}%", fraction * 100.0),
            Length::Compute(calc) => calc.fmt_debug(f),
        }
    }
}

/// Defines how to compute a length in unscaled pixels.
pub trait LengthCalculation {
    /// Resolves the length.
    fn resolve(&self, info: &LayoutInfo) -> f64;

    /// Clones the length calculation.
    fn dyn_clone(&self) -> Box<dyn LengthCalculation>;

    /// Formats the length calculation for debugging.
    #[inline]
    fn fmt_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("calc")
    }
}

impl Clone for Box<dyn LengthCalculation> {
    #[inline]
    fn clone(&self) -> Self {
        (**self).dyn_clone()
    }
}

impl<F: 'static + Clone + Fn(&LayoutInfo) -> f64> LengthCalculation for F {
    #[inline]
    fn resolve(&self, info: &LayoutInfo) -> f64 {
        self(info)
    }

    #[inline]
    fn dyn_clone(&self) -> Box<dyn LengthCalculation> {
        Box::new(self.clone())
    }
}
