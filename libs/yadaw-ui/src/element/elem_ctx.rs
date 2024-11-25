use {
    crate::{App, Window},
    vello::kurbo::Size,
};

/// A context that is passed along to element methods.
pub struct ElemCtx {
    /// The size of the parent element.
    ///
    /// This is the size that the parent element has determined for this element.
    pub(crate) parent_size: Size,

    /// The current scale factor of the element.
    pub(crate) scale_factor: f64,

    /// The window handle that the element is a part of.
    pub(crate) window: Window,

    /// The application handle.
    pub(crate) app: App,
}

impl ElemCtx {
    /// Re-creates a new [`ElemCtx`] with the same properties as this one, but with different
    /// element-specific properties.
    pub fn inherit(&self, parent_size: Size, scale_factor: f64) -> Self {
        Self {
            parent_size,
            scale_factor,
            window: self.window.clone(),
            app: self.app.clone(),
        }
    }

    /// Returns the size of the element's parent.
    ///
    /// This size is mostly used to compute some lengths which are relative to the parent's size.
    #[inline]
    pub fn parent_size(&self) -> Size {
        self.parent_size
    }

    /// Returns the current scale factor for the element.
    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// Returns the window handle that the element is a part of.
    #[inline]
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Returns the application handle.
    #[inline]
    pub fn app(&self) -> &App {
        &self.app
    }
}
