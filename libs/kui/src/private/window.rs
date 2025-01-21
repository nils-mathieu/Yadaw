use {
    crate::private::{CtxInner, Renderer, WindowAndSurface},
    std::rc::Rc,
    vello::{peniko, wgpu},
    winit::{dpi::PhysicalSize, window::Window},
};

/// The inner state associated with a window.
pub struct WindowInner {
    /// The context that owns the window.
    ctx: Rc<CtxInner>,

    /// The concrete winit object that can be used to manipulate
    /// the underlying window.
    window_and_surface: WindowAndSurface,
}

impl WindowInner {
    /// Creates a new [`WindowInner`] object.
    pub fn new(ctx: Rc<CtxInner>, window_and_surface: WindowAndSurface) -> Self {
        Self {
            ctx,
            window_and_surface,
        }
    }

    /// Draws the content of the window to the provided scene.
    ///
    /// # Remarks
    ///
    /// This function might call user-defined functions!
    pub fn draw_to_scene(&self, scene: &mut vello::Scene) {
        scene.reset();
    }

    /// Renders the provided scene to this window.
    #[inline]
    pub fn render_scene(&self, renderer: &mut Renderer, scene: &vello::Scene) {
        self.window_and_surface.render(renderer, scene);
    }

    /// Notifies the window that it has been resized.
    #[inline]
    pub fn notify_resized(&self, size: PhysicalSize<u32>) {
        self.window_and_surface.set_size(size);
    }

    /// Returns a reference to the context that owns this window.
    #[inline]
    pub fn ctx(&self) -> &CtxInner {
        &self.ctx
    }

    /// Returns a reference to the concrete winit [`Window`] object.
    #[inline]
    pub fn winit_window(&self) -> &Window {
        self.window_and_surface.winit_window()
    }

    /// Sets the present mode to be used by the window.
    #[inline]
    pub fn set_present_mode(&self, present_mode: wgpu::PresentMode) {
        self.window_and_surface.set_present_mode(present_mode);
    }

    /// Sets the base (clear) color of the window.
    #[inline]
    pub fn set_base_color(&self, base_color: peniko::Color) {
        self.window_and_surface.set_base_color(base_color);
    }
}
