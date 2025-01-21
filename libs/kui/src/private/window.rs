use {
    crate::{
        Ctx, ElemContext, SizeConstraint, Window,
        element::{Element, LayoutInfo},
        private::{CtxInner, Renderer, WindowAndSurface},
    },
    std::{cell::Cell, rc::Rc},
    vello::{
        kurbo::{self, Point},
        peniko, wgpu,
    },
    winit::{dpi::PhysicalSize, window::Window as WinitWindow},
};

/// The inner state associated with a window.
pub struct WindowInner {
    /// The context that owns the window.
    ctx: Rc<CtxInner>,

    /// The concrete winit object that can be used to manipulate
    /// the underlying window.
    window_and_surface: WindowAndSurface,

    /// Whether the layout of the UI tree needs to be re-computed.
    recompute_layout: Cell<bool>,

    /// The root element of the window.
    root_element: Cell<Box<dyn Element>>,

    /// The scale factor of the window.
    scale_factor: Cell<f64>,
}

impl WindowInner {
    /// Creates a new [`WindowInner`] object.
    pub fn new(ctx: Rc<CtxInner>, window_and_surface: WindowAndSurface) -> Self {
        let scale_factor = window_and_surface.winit_window().scale_factor();

        Self {
            ctx,
            window_and_surface,
            recompute_layout: Cell::new(true),
            root_element: Cell::new(Box::new(())),
            scale_factor: Cell::new(scale_factor),
        }
    }

    /// Draws the content of the window to the provided scene.
    ///
    /// # Remarks
    ///
    /// This function might call user-defined functions!
    pub fn draw_to_scene(self: &Rc<Self>, scene: &mut vello::Scene) {
        // This custom element is used as a sentinel to check whether the root element of the
        // window has changed during the draw callback.
        struct PrivateElement;
        impl Element for PrivateElement {
            #[inline]
            fn __private_implementation_detail_do_not_use(&self) -> bool {
                true
            }
        }

        let mut root_element = self.root_element.replace(Box::new(PrivateElement));
        let elem_context = ElemContext {
            ctx: Ctx(Rc::downgrade(&self.ctx)),
            window: Window(Rc::downgrade(self)),
        };

        if self.recompute_layout.get() {
            let size = self.window_and_surface.cached_size();
            let size = kurbo::Size::new(size.width as f64, size.height as f64);

            root_element.layout(&elem_context, LayoutInfo {
                parent: size,
                available: SizeConstraint::from_size(size),
                scale_factor: self.scale_factor.get(),
            });
            root_element.place(&elem_context, Point::ORIGIN);
            self.recompute_layout.set(false);
        }

        scene.reset();
        root_element.draw(&elem_context, scene);

        let potentially_replaced = self.root_element.replace(root_element);
        if !potentially_replaced.__private_implementation_detail_do_not_use() {
            // The root element has been modified during one of the callbacks.
            // Let's restore the requested new root element and destroy the temporary one.
            self.root_element.set(potentially_replaced);
        }
    }

    /// Renders the provided scene to this window.
    #[inline]
    pub fn render_scene(&self, renderer: &mut Renderer, scene: &vello::Scene) {
        self.window_and_surface.render(renderer, scene);
    }

    /// Notifies the window that it has been resized.
    #[inline]
    pub fn notify_resized(&self, size: PhysicalSize<u32>) {
        self.recompute_layout.set(true);
        self.window_and_surface.set_size(size);
    }

    /// Notifies the window that the scale factor of the window has changed.
    pub fn notify_scale_factor_changed(&self, scale_factor: f64) {
        self.scale_factor.set(scale_factor);
        self.recompute_layout.set(true);
    }

    /// Returns a reference to the context that owns this window.
    #[inline]
    pub fn ctx(&self) -> &CtxInner {
        &self.ctx
    }

    /// Returns a reference to the concrete winit [`Window`](WinitWindow) object.
    #[inline]
    pub fn winit_window(&self) -> &WinitWindow {
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

    /// Sets the root element of the window.
    #[inline]
    pub fn set_root_element(&self, elem: Box<dyn Element>) {
        self.root_element.set(elem);
        self.recompute_layout.set(true);
        self.window_and_surface.winit_window().request_redraw();
    }
}
