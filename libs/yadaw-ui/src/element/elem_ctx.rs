use {
    crate::{App, Window},
    std::time::Instant,
    vello::kurbo::{Rect, Size},
};

/// A context that is passed along to element methods.
pub struct ElemCtx {
    /// The current clip rectangle. Anything outside of that rectangle will not be rendered.
    ///
    /// This can be used to avoid rendering parts of the element that are not visible.
    pub(crate) clip_rect: Rect,

    /// The size of the parent element.
    ///
    /// This is the size that the parent element has determined for this element.
    pub(crate) parent_size: Size,

    /// The current scale factor of the element.
    pub(crate) scale_factor: f64,

    /// The current instant in time.
    pub(crate) now: Instant,

    /// The window handle that the element is a part of.
    pub(crate) window: Window,

    /// The application handle.
    pub(crate) app: App,
}

impl ElemCtx {
    /// Re-creates a new [`ElemCtx`] with the same properties as this one, but with different
    /// element-specific properties.
    pub fn inherit_all(&self, clip_rect: Rect, parent_size: Size, scale_factor: f64) -> Self {
        Self {
            clip_rect,
            parent_size,
            scale_factor,
            now: self.now,
            window: self.window.clone(),
            app: self.app.clone(),
        }
    }

    /// Re-creates a new [`ElemCtx`] with the same properties as this one, but with a different
    /// parent size.
    pub fn inherit_parent_size(&self, parent_size: Size) -> Self {
        Self {
            clip_rect: self.clip_rect,
            parent_size,
            scale_factor: self.scale_factor,
            now: self.now,
            window: self.window.clone(),
            app: self.app.clone(),
        }
    }

    /// Re-creates a new [`ElemCtx`] with the same properties as this one, but with a different
    /// clip rectangle.
    pub fn inherit_clip_rect(&self, clip_rect: Rect) -> Self {
        Self {
            clip_rect,
            parent_size: self.parent_size,
            scale_factor: self.scale_factor,
            now: self.now,
            window: self.window.clone(),
            app: self.app.clone(),
        }
    }

    /// Re-creates a new [`ElemCtx`] with the same properties as this one, but with a different
    /// scale factor.
    pub fn inherit_scale_factor(&self, scale_factor: f64) -> Self {
        Self {
            clip_rect: self.clip_rect,
            parent_size: self.parent_size,
            scale_factor,
            now: self.now,
            window: self.window.clone(),
            app: self.app.clone(),
        }
    }

    /// Returns the current clip rectangle. Anything outside of that rectangle will not be rendered.
    ///
    /// This can be used to avoid rendering parts of the element that are not visible.
    #[inline]
    pub fn clip_rect(&self) -> Rect {
        self.clip_rect
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

    /// Returns the current instant in time.
    #[inline]
    pub fn now(&self) -> Instant {
        self.now
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
