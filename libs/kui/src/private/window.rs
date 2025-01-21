use {crate::private::CtxInner, std::rc::Rc, winit::window::Window};

/// The inner state associated with a window.
pub struct WindowInner {
    /// The context that owns the window.
    ctx: Rc<CtxInner>,

    /// The concrete winit object that can be used to manipulate
    /// the underlying window.
    winit_window: Window,
}

impl WindowInner {
    /// Creates a new [`WindowInner`] object.
    pub fn new(ctx: Rc<CtxInner>, winit_window: Window) -> Self {
        Self { ctx, winit_window }
    }

    /// Returns a reference to the context that owns this window.
    #[inline]
    pub fn ctx(&self) -> &CtxInner {
        &self.ctx
    }

    /// Returns a reference to the concrete winit [`Window`] object.
    #[inline]
    pub fn winit_window(&self) -> &Window {
        &self.winit_window
    }
}
