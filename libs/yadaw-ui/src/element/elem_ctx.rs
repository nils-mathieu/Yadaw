use {
    crate::{App, Window},
    vello::kurbo::{Rect, Size},
};

/// A context that is passed along to element methods.
#[derive(Clone)]
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

    /// Contains whether the cursor is currently absent from the current
    /// element tree.
    ///
    /// This happens either because it is outside of the window, or because it
    /// is outside of the current clipping shape.
    pub(crate) cursor_present: bool,

    /// The window handle that the element is a part of.
    pub(crate) window: Window,

    /// The application handle.
    pub(crate) app: App,
}

impl ElemCtx {
    /// Re-creates a new [`ElemCtx`] with the same properties as this one, but with a different
    /// parent size.
    pub fn inherit_parent_size(&self, parent_size: Size) -> Self {
        Self {
            parent_size,
            ..self.clone()
        }
    }

    /// Re-creates a new [`ElemCtx`] with the same properties as this one, but with a different
    /// clip rectangle.
    pub fn inherit_clip_rect(&self, clip_rect: Rect) -> Self {
        Self {
            clip_rect,
            ..self.clone()
        }
    }

    /// Re-creates a new [`ElemCtx`] with the same properties as this one, but with a different
    /// scale factor.
    pub fn inherit_scale_factor(&self, scale_factor: f64) -> Self {
        Self {
            scale_factor,
            ..self.clone()
        }
    }

    /// Re-creates a new [`ElemCtx`] with the same properties as this one, but with a different
    /// cursor present state.
    pub fn inherit_cursor_present(&self, yes: bool) -> Self {
        Self {
            cursor_present: yes,
            ..self.clone()
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

    /// Returns whether the cursor is currently present from the current element tree.
    ///
    /// This can become `false` either because the cursor is outside of the window, or because it
    /// is outside of the current clipping shape.
    #[inline]
    pub fn is_cursor_present(&self) -> bool {
        self.cursor_present
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
