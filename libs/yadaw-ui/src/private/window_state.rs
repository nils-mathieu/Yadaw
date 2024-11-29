use {
    crate::{
        element::{ElemCtx, Element, Event, SetSize},
        private::{AppState, Renderer, SurfaceTarget},
        App, Window,
    },
    std::{
        cell::Cell,
        mem::ManuallyDrop,
        rc::{Rc, Weak},
    },
    vello::{
        kurbo::{Point, Size},
        peniko::Color,
        Scene,
    },
    winit::{dpi::PhysicalSize, window::Window as OsWindow},
};

/// The dirty state of the window.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DirtyState {
    /// The window is not dirty. Nothing needs to be done.
    #[default]
    Clean,
    /// The layout of the elements in the window needs to be re-calculated.
    Layout,
    /// The surface needs to be re-configured.
    Surface,
}

/// Contains the state of a window created by the UI framework.
///
/// This type is expected to be used through a reference counted pointer.
pub struct WindowState {
    /// The underlying window object that is managed by the `winit` crate.
    window: SurfaceTarget,

    /// The global application state.
    app: Weak<AppState>,

    /// The dirty state of the window.
    dirty_state: Cell<DirtyState>,

    /// Whether the window has been requested to close.
    ///
    /// It's not possible to close the window directly because some references to the window
    /// may still be held by callbacks in the UI framework. Instead, this flag will be checked
    /// at the end of the current event loop iteration to see if the window should be closed.
    closing: Cell<bool>,
    /// The color to clear the window with.
    clear_color: Cell<Color>,

    /// The current inner size of the window.
    size: Cell<PhysicalSize<u32>>,

    /// The root element of the window.
    root_element: Cell<Option<Box<dyn Element>>>,
    /// The current scale factor.
    ///
    /// This is used to scale the window's content to match the actual size of the window
    /// on high-DPI displays.
    scale_factor: Cell<f64>,
}

impl WindowState {
    /// Creates a new [`WindowState`] instance.
    pub fn new(window: SurfaceTarget, app: Weak<AppState>) -> Rc<Self> {
        let size = window.inner_size();
        let scale_factor = window.scale_factor();

        Rc::new(Self {
            window,
            app,
            dirty_state: Cell::new(DirtyState::Surface),
            closing: Cell::new(false),
            size: Cell::new(size),
            clear_color: Cell::new(Color::WHITE),
            root_element: Cell::new(Some(Box::new(()))),
            scale_factor: Cell::new(scale_factor),
        })
    }

    /// Returns the underlying window object.
    #[inline]
    pub fn os_window(&self) -> &OsWindow {
        &self.window
    }

    /// Sets the `closing` flag of the window.
    ///
    /// This will be checked at the end of the current event loop iteration to see if the window
    /// should be closed.
    #[inline]
    pub fn close(&self) {
        self.closing.set(true);
    }

    /// Returns whether the window has been requested to close.
    #[inline]
    pub fn closing(&self) -> bool {
        self.closing.get()
    }

    /// Adds dirt to the window state.
    fn add_dirt(&self, dirt: DirtyState) {
        self.dirty_state.set(self.dirty_state.get().max(dirt));
        self.window.request_redraw();
    }

    /// Returns a [`ElemCtx`] instance for the window.
    fn elem_ctx(self: &Rc<Self>) -> ElemCtx {
        let size = self.size.get();
        let size = Size::new(size.width as f64, size.height as f64);

        ElemCtx {
            clip_rect: size.to_rect(),
            parent_size: size,
            scale_factor: self.scale_factor.get(),
            window: Window::from_state(Rc::downgrade(self)),
            app: App::from_state(self.app.clone()),
        }
    }

    /// Calls the provided closure with the root element of the window.
    fn with_root_element<R>(self: &Rc<Self>, f: impl FnOnce(&mut dyn Element) -> R) -> R {
        struct Guard<'a> {
            slot: &'a Cell<Option<Box<dyn Element>>>,
            elem: ManuallyDrop<Box<dyn Element>>,
        }

        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                let elem = unsafe { ManuallyDrop::take(&mut self.elem) };

                if let Some(replaced_by) = self.slot.replace(Some(elem)) {
                    // The element has been replaced.
                    self.slot.set(Some(replaced_by));
                }
            }
        }

        let elem = self
            .root_element
            .take()
            .expect("Root element is not available");

        let mut guard = Guard {
            slot: &self.root_element,
            elem: ManuallyDrop::new(elem),
        };

        f(&mut *guard.elem)
    }

    /// Notifies the window state that the size of the window has changed.
    pub fn set_size(&self, size: PhysicalSize<u32>) {
        self.size.set(size);
        self.add_dirt(DirtyState::Surface);
    }

    /// Re-renders the window's content.
    ///
    /// # Parameters
    ///
    /// * `renderer` - The renderer that was used to create the underlying surface.
    ///
    /// * `scratch_scene` - The scratch scene that is used to render the window's content. Note
    ///   that the scene's content will be ignored and cleared before rendering.
    pub fn render(self: &Rc<Self>, renderer: &mut Renderer, scratch_scene: &mut Scene) {
        if self.size.get().width == 0 || self.size.get().height == 0 {
            return;
        }

        let dirty_state = self.dirty_state.take();

        if dirty_state >= DirtyState::Surface {
            self.window
                .re_configure_swapchain(renderer, self.size.get());
        }

        self.with_root_element(|root| {
            let elem_ctx = self.elem_ctx();

            if dirty_state >= DirtyState::Layout {
                root.set_size(&elem_ctx, SetSize::Specific(elem_ctx.parent_size()));
                root.set_position(&elem_ctx, Point::ZERO);
            }

            scratch_scene.reset();
            root.render(&elem_ctx, scratch_scene);
            self.window.render(
                renderer,
                self.size.get(),
                self.clear_color.get(),
                scratch_scene,
            );
        });
    }

    /// Sets the root element of the window.
    pub fn set_root_element(&self, root: Box<dyn Element>) {
        self.root_element.set(Some(root));
        self.add_dirt(DirtyState::Layout);
    }

    /// Updates the scale factor of the window.
    pub fn set_scale_factor(&self, scale_factor: f64) {
        self.scale_factor.set(scale_factor);
        self.add_dirt(DirtyState::Layout);
    }

    /// Dispatches an event to the window and its element tree.
    pub fn dispatch_event(self: &Rc<Self>, event: &dyn Event) {
        self.with_root_element(|root| {
            let elem_ctx = self.elem_ctx();
            root.event(&elem_ctx, event);
        });
    }
}
