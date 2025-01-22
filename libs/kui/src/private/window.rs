use {
    crate::{
        Ctx, ElemContext, LayoutContext, Window,
        element::Element,
        event::{Event, EventResult},
        private::{CtxInner, Renderer, WindowAndSurface},
    },
    core::f64,
    std::{cell::Cell, rc::Rc},
    vello::{
        kurbo::{self, Point},
        peniko, wgpu,
    },
    winit::{
        dpi::{PhysicalPosition, PhysicalSize},
        window::Window as WinitWindow,
    },
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
    /// The last reported position of the pointer.
    last_pointer_position: Cell<PhysicalPosition<f64>>,
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
            last_pointer_position: Cell::new(PhysicalPosition::new(f64::INFINITY, f64::INFINITY)),
        }
    }

    /// Creates the [`ElemContext`] for the elements that are part of this window.
    fn make_elem_context(self: &Rc<Self>) -> ElemContext {
        ElemContext {
            ctx: Ctx(Rc::downgrade(&self.ctx)),
            window: Window(Rc::downgrade(self)),
        }
    }

    /// Calls the provided function with the root element of the window.
    ///
    /// This function takes care of the case were the root element is replaced while the
    /// closure is running.
    fn with_root_element<R>(&self, f: impl FnOnce(&mut dyn Element) -> R) -> R {
        // This custom element is used as a sentinel to check whether the root element of the
        // window has changed during the draw callback.
        struct PrivateElement;
        impl Element for PrivateElement {
            #[inline]
            fn __private_implementation_detail_do_not_use(&self) -> bool {
                true
            }
        }

        /// The guard responisble for restoring the root element.
        struct Guard<'a> {
            slot: &'a Cell<Box<dyn Element>>,
            root_element: Box<dyn Element>,
        }

        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                self.slot.swap(Cell::from_mut(&mut self.root_element));

                if !self
                    .root_element
                    .__private_implementation_detail_do_not_use()
                {
                    // The root element has been modified during one of the callbacks.
                    // Let's restore the requested new root element and destroy the temporary one.
                    self.slot.swap(Cell::from_mut(&mut self.root_element));
                }
            }
        }

        let root_element = self.root_element.replace(Box::new(PrivateElement));

        let mut guard = Guard {
            slot: &self.root_element,
            root_element,
        };

        f(guard.root_element.as_mut())
    }

    /// Draws the content of the window to the provided scene.
    ///
    /// # Remarks
    ///
    /// This function might call user-defined functions!
    pub fn draw_to_scene(self: &Rc<Self>, scene: &mut vello::Scene) {
        let elem_context = self.make_elem_context();

        self.with_root_element(|elem| {
            if self.recompute_layout.get() {
                let size = self.window_and_surface.cached_size();
                let size = kurbo::Size::new(size.width as f64, size.height as f64);
                elem.place(
                    &elem_context,
                    LayoutContext {
                        parent: size,
                        scale_factor: self.scale_factor.get(),
                    },
                    Point::ORIGIN,
                    size,
                );
                self.recompute_layout.set(false);
            }

            scene.reset();
            elem.draw(&elem_context, scene);
        });
    }

    /// Dispatches an event to the window.
    pub fn dispatch_event(self: &Rc<Self>, event: &dyn Event) -> EventResult {
        let elem_context = self.make_elem_context();
        self.with_root_element(|elem| elem.event(&elem_context, event))
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
    pub fn winit_window(&self) -> &dyn WinitWindow {
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
    pub fn set_root_element(self: &Rc<Self>, mut elem: Box<dyn Element>) {
        let elem_ctx = self.make_elem_context();
        elem.begin(&elem_ctx);
        self.root_element.set(elem);
        self.recompute_layout.set(true);
        self.window_and_surface.winit_window().request_redraw();
    }

    /// Requests a re-layout of the UI tree.
    pub fn request_relayout(&self) {
        self.recompute_layout.set(true);
        self.window_and_surface.winit_window().request_redraw();
    }

    /// Returns the window's scale factor.
    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor.get()
    }

    /// Sets the last pointer position for the window.
    #[inline]
    pub fn set_last_pointer_position(&self, position: PhysicalPosition<f64>) {
        self.last_pointer_position.set(position);
    }

    /// Returns the last reported position of the pointer over the window's
    /// surface area.
    #[inline]
    pub fn last_pointer_position(&self) -> PhysicalPosition<f64> {
        self.last_pointer_position.get()
    }

    /// Returns the window's size.
    #[inline]
    pub fn cached_size(&self) -> PhysicalSize<u32> {
        self.window_and_surface.cached_size()
    }
}
