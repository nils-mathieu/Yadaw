use {
    crate::private::WindowInner,
    std::{
        fmt::Debug,
        rc::{Rc, Weak},
    },
};

/// A window that is managed by the application.
///
/// # Remarks
///
/// This type is a handle to a window that is managed by the application. It's possible to close
/// the window while still having live handles to it. When a function is used on a window that has
/// been closed, that function will panic.
#[derive(Clone)]
pub struct Window(pub(crate) Weak<WindowInner>);

impl Window {
    /// Returns whether the window is currently open.
    ///
    /// This is the only function of [`Window`] that won't panic if called after the event loop has
    /// finished running or if the window has been closed.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.0.strong_count() > 0
    }

    /// Attempts to upgrade the inner [`WindowInner`], and panics if the window has no longer
    /// available.
    #[track_caller]
    fn inner(&self) -> Rc<WindowInner> {
        self.0
            .upgrade()
            .expect("Attempted to use a `Window` after it has been closed")
    }

    /// Closes the window.
    #[track_caller]
    pub fn close(&self) {
        let inner = self.inner();
        let id = inner.winit_window().id();
        inner.ctx().remove_window(id);
    }
}

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Window { ... }")
    }
}
