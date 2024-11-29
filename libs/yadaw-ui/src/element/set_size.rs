use vello::kurbo::Size;

/// A way to set the size of an element.
#[derive(Debug, Clone, Copy)]
pub struct SetSize {
    /// The width constraint.
    ///
    /// Negative values indicate that the width is unconstrained.
    width: f64,
    /// The height constraint.
    ///
    /// Negative values indicate that the height is unconstrained.
    height: f64,
}

impl SetSize {
    /// Creates a new [`SetSize`] instance with no constraints.
    pub fn unconstrained() -> Self {
        Self {
            width: -1.0,
            height: -1.0,
        }
    }

    /// Creates a new [`SetSize`] instance with the provided constraints.
    pub fn from_specific(size: Size) -> Self {
        debug_assert!(size.width.is_sign_positive());
        debug_assert!(size.height.is_sign_positive());

        Self {
            width: size.width,
            height: size.height,
        }
    }

    /// Creates a new [`SetSize`] instance with the provided constraints.
    pub fn from_width(width: f64) -> Self {
        debug_assert!(width.is_sign_positive());

        Self {
            width,
            height: -1.0,
        }
    }

    /// Creates a new [`SetSize`] instance with the provided constraints.
    pub fn from_height(height: f64) -> Self {
        debug_assert!(height.is_sign_positive());

        Self {
            width: -1.0,
            height,
        }
    }

    /// Returns whether the width constraint is specific.
    #[inline]
    pub fn has_specific_width(self) -> bool {
        self.width.is_sign_positive()
    }

    /// Returns whether the height constraint is specific.
    #[inline]
    pub fn has_specific_height(self) -> bool {
        self.height.is_sign_positive()
    }

    /// Returns the width constraint stored in this [`SetSize`] instance.
    #[inline]
    pub fn width(self) -> Option<f64> {
        if self.has_specific_width() {
            None
        } else {
            Some(self.width)
        }
    }

    /// Returns the height constraint stored in this [`SetSize`] instance.
    #[inline]
    pub fn height(self) -> Option<f64> {
        if self.has_specific_height() {
            None
        } else {
            Some(self.height)
        }
    }

    /// Returns the size stored in this [`SetSize`] instance.
    ///
    /// If any of the constraints are absent, this method will return `None`.
    pub fn specific_size(self) -> Option<Size> {
        if self.width.is_sign_negative() || self.height.is_sign_negative() {
            None
        } else {
            Some(Size {
                width: self.width,
                height: self.height,
            })
        }
    }

    /// Returns the size stored in this [`SetSize`] instance, but falls back to `fallback` if the
    /// size is not specific.
    pub fn fallback(self, fallback: Size) -> Size {
        Size {
            width: self.width().unwrap_or(fallback.width),
            height: self.height().unwrap_or(fallback.height),
        }
    }

    /// Removes the width constraint from the request.
    #[inline]
    pub fn without_width(mut self) -> Self {
        self.width = -1.0;
        self
    }

    /// Removes the height constraint from the request.
    #[inline]
    pub fn without_height(mut self) -> Self {
        self.height = -1.0;
        self
    }
}
