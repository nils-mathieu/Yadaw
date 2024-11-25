use {
    std::{cell::Cell, rc::Rc},
    winit::window::Window as OsWindow,
};

/// Contains the state of a window created by the UI framework.
///
/// This type is expected to be used through a reference counted pointer.
pub struct WindowState {
    /// The underlying window object that is managed by the `winit` crate.
    window: OsWindow,

    /// Whether the window has been requested to close.
    ///
    /// It's not possible to close the window directly because some references to the window
    /// may still be held by callbacks in the UI framework. Instead, this flag will be checked
    /// at the end of the current event loop iteration to see if the window should be closed.
    closing: Cell<bool>,
}

impl WindowState {
    /// Creates a new [`WindowState`] instance.
    pub fn new(window: OsWindow) -> Rc<Self> {
        Rc::new(Self {
            window,
            closing: Cell::new(false),
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
}
