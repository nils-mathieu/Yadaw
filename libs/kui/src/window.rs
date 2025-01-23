use {
    crate::{
        element::Element,
        event::Event,
        private::{WindowInner, WindowProxyInner},
    },
    std::{
        fmt::Debug,
        rc::{Rc, Weak},
        sync::Arc,
    },
    vello::{
        kurbo::{Point, Size},
        peniko, wgpu,
    },
    winit::event_loop::EventLoopProxy,
};

/// Allows accessing a window from any thread (rather than only the UI thread).
#[derive(Clone)]
pub struct WindowProxy {
    inner: std::sync::Weak<WindowProxyInner>,
    event_loop_proxy: EventLoopProxy,
}

impl WindowProxy {
    /// Returns the [`WindowProxyInner`].
    #[track_caller]
    fn inner(&self) -> Arc<WindowProxyInner> {
        self.inner
            .upgrade()
            .expect("Attempted to use a `WindowProxy` after the window has been closed")
    }

    /// Sends an event to the window's UI tree.
    #[track_caller]
    pub fn send_event(&self, event: impl Send + Event) {
        self.send_event_boxed(Box::new(event));
    }

    /// Sends an event to the window's UI tree.
    #[track_caller]
    pub fn send_event_boxed(&self, event: Box<dyn Send + Event>) {
        self.inner().send_event(event);
        self.event_loop_proxy.wake_up();
    }

    /// Requests the layout to be recomputed.
    #[track_caller]
    pub fn request_relayout(&self) {
        self.inner().request_relayout();
    }

    /// Requests the window to be redrawn.
    #[track_caller]
    pub fn request_redraw(&self) {
        self.inner().winit_window().request_redraw();
    }

    /// Calls the provided closure with a reference to the concrete [`winit::window::Window`]
    /// object.
    #[track_caller]
    pub fn with_winit_window<R>(&self, f: impl FnOnce(&dyn winit::window::Window) -> R) -> R {
        f(self.inner().winit_window())
    }
}

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
        let id = inner.proxy().winit_window().id();
        inner.ctx().remove_window(id);
    }

    /// Calls the provided function with a reference to the concrete winit [`Window`] object
    /// backing this window.
    #[track_caller]
    pub fn with_winit_window<R>(&self, f: impl FnOnce(&dyn winit::window::Window) -> R) -> R {
        f(self.inner().proxy().winit_window())
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
        self.inner().proxy().winit_window().request_redraw();
    }

    /// Requests the UI tree associated with the window to be re-built (and the window to be
    /// re-rendered).
    #[track_caller]
    pub fn request_relayout(&self) {
        self.inner().proxy().request_relayout();
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

    /// Returns the scale factor of the window.
    #[track_caller]
    pub fn scale_factor(&self) -> f64 {
        self.inner().scale_factor()
    }

    /// Returns the size of the window.
    #[track_caller]
    pub fn size(&self) -> Size {
        let cached_size = self.inner().cached_size();
        Size::new(cached_size.width as f64, cached_size.height as f64)
    }

    /// Returns the last known position of the pointer over the window's client area.
    #[track_caller]
    pub fn pointer_position(&self) -> Point {
        let pos = self.inner().last_pointer_position();
        Point::new(pos.x, pos.y)
    }

    /// Creates a [`WindowProxy`] for this window.
    ///
    /// The proxy can be used to send events to the window's UI tree from other threads.
    #[track_caller]
    pub fn make_proxy(&self) -> WindowProxy {
        let inner = self.inner();

        WindowProxy {
            event_loop_proxy: inner.ctx().event_loop_proxy(),
            inner: Arc::downgrade(inner.proxy()),
        }
    }
}

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Window { ... }")
    }
}
