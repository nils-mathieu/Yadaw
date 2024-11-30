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

    #[track_caller]
    pub fn new(width: Option<f64>, height: Option<f64>) -> Self {
        assert!(width.is_none_or(|w| w.is_sign_positive()));
        assert!(height.is_none_or(|h| h.is_sign_positive()));

        Self {
            width: width.unwrap_or(-1.0),
            height: height.unwrap_or(-1.0),
        }
    }

    /// Creates a new [`SetSize`] instance with the provided constraints.
    #[track_caller]
    pub fn from_specific(size: Size) -> Self {
        debug_assert!(size.width.is_sign_positive());
        debug_assert!(size.height.is_sign_positive());

        Self {
            width: size.width,
            height: size.height,
        }
    }

    /// Creates a new [`SetSize`] instance with the provided constraints.
    #[track_caller]
    pub fn from_width(width: f64) -> Self {
        debug_assert!(width.is_sign_positive());

        Self {
            width,
            height: -1.0,
        }
    }

    /// Creates a new [`SetSize`] instance with the provided constraints.
    #[track_caller]
    pub fn from_height(height: f64) -> Self {
        debug_assert!(height.is_sign_positive());

        Self {
            width: -1.0,
            height,
        }
    }

    /// Sets the width constraint.
    pub fn with_width(mut self, width: f64) -> Self {
        assert!(width.is_sign_positive());

        self.width = width;
        self
    }

    /// Sets the height constraint.
    pub fn with_height(mut self, height: f64) -> Self {
        assert!(height.is_sign_positive());

        self.height = height;
        self
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
            Some(self.width)
        } else {
            None
        }
    }

    /// Returns the height constraint stored in this [`SetSize`] instance.
    #[inline]
    pub fn height(self) -> Option<f64> {
        if self.has_specific_height() {
            Some(self.height)
        } else {
            None
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

    /// Returns a [`Size`] that represents the constraints stored in this [`SetSize`] instance.
    ///
    /// Unconstrained dimensions are returned as `f64::INFINITY`.
    pub fn or_infinity(self) -> Size {
        Size {
            width: self.width().unwrap_or(f64::INFINITY),
            height: self.height().unwrap_or(f64::INFINITY),
        }
    }

    /// Returns a [`SetSize`] that has the current constraints, or the constraints of `other` if
    /// they are more specific.
    pub fn or(self, other: Self) -> Self {
        Self::new(
            self.width().or(other.width()),
            self.height().or(other.height()),
        )
    }
}

impl PartialEq for SetSize {
    fn eq(&self, other: &Self) -> bool {
        self.width() == other.width() && self.height() == other.height()
    }
}

impl From<Size> for SetSize {
    fn from(size: Size) -> Self {
        Self::from_specific(size)
    }
}
