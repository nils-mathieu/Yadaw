use {
    crate::{element::Element, private::WindowInner},
    std::{
        fmt::Debug,
        rc::{Rc, Weak},
    },
    vello::{peniko, wgpu},
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

    /// Calls the provided function with a reference to the concrete winit [`Window`] object
    /// backing this window.
    #[track_caller]
    pub fn with_winit_window<R>(&self, f: impl FnOnce(&winit::window::Window) -> R) -> R {
        f(self.inner().winit_window())
    }

    /// Sets the clear color of the window.
    #[track_caller]
    pub fn set_clear_color(&self, color: impl Into<peniko::Color>) {
        self.inner().set_base_color(color.into());
    }

    /// Sets whether the window should use V-Sync or not.
    #[track_caller]
    pub fn set_vsync(&self, vsync: bool) {
        self.inner().set_present_mode(if vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        });
    }

    /// Requests a redraw of the window.
    #[track_caller]
    pub fn request_redraw(&self) {
        self.inner().winit_window().request_redraw();
    }

    /// Sets the root element of the window as a boxed value.
    #[track_caller]
    pub fn set_root_element_boxed(&self, elem: Box<dyn Element>) {
        self.inner().set_root_element(elem);
    }

    /// Sets the root element of the window.
    #[track_caller]
    pub fn set_root_element(&self, elem: impl 'static + Element) {
        self.set_root_element_boxed(Box::new(elem));
    }
}

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Window { ... }")
    }
}
