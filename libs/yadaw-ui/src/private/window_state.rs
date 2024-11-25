use {
    crate::private::{Renderer, SurfaceTarget},
    std::{cell::Cell, rc::Rc},
    vello::{peniko::Color, Scene},
    winit::{dpi::PhysicalSize, window::Window as OsWindow},
};

/// The dirty state of the window.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DirtyState {
    /// The window is not dirty. Nothing needs to be done.
    #[default]
    Clean,
    /// The surface needs to be re-configured.
    Surface,
}

/// Contains the state of a window created by the UI framework.
///
/// This type is expected to be used through a reference counted pointer.
pub struct WindowState {
    /// The underlying window object that is managed by the `winit` crate.
    window: SurfaceTarget,

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
}

impl WindowState {
    /// Creates a new [`WindowState`] instance.
    pub fn new(window: SurfaceTarget) -> Rc<Self> {
        let size = window.inner_size();

        Rc::new(Self {
            window,
            dirty_state: Cell::new(DirtyState::Surface),
            closing: Cell::new(false),
            size: Cell::new(size),
            clear_color: Cell::new(Color::WHITE),
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
    pub fn render(&self, renderer: &mut Renderer, scratch_scene: &mut Scene) {
        let dirty_state = self.dirty_state.take();

        if dirty_state >= DirtyState::Surface {
            self.window
                .re_configure_swapchain(renderer, self.size.get());
        }

        scratch_scene.reset();
        self.window.render(
            renderer,
            self.size.get(),
            self.clear_color.get(),
            scratch_scene,
        );
    }
}
