use {
    crate::{element::Element, private::WindowState},
    std::{
        fmt::Debug,
        rc::{Rc, Weak},
    },
};

/// Represents a window that has been opened by the UI framework.
///
/// This is a lightweight handle to the window that can be used to interact with the window. It can
/// be cloned and passed around freely.
///
/// # Remarks
///
/// This handle is not tied to the lifetime of the actual window. This means that after the
/// [`close`] method is called on the window, this handle may still be used. If it is used after
/// the window has been closed, most of the methods that involve the window will cause a panic.
///
/// [`close`]: WindowState::close
#[derive(Clone)]
pub struct Window(Weak<WindowState>);

impl Window {
    /// Creates a new [`Window`] instance from the provided [`WindowState`] reference.
    #[inline]
    pub(crate) fn from_state(state: Weak<WindowState>) -> Self {
        Self(state)
    }

    /// Returns whether the window is still open.
    ///
    /// When this function returns `false`, most of the methods on this type will cause a panic.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.0.strong_count() != 0
    }

    /// Upgrades the internal reference counted pointer to the window state.
    #[track_caller]
    fn state(&self) -> Rc<WindowState> {
        self.0
            .upgrade()
            .expect("The window has been closed previously")
    }

    /// Sets the root element of the window to the provided element.
    #[track_caller]
    pub fn set_root_element(&self, element: impl Element + 'static) {
        self.state().set_root_element(Box::new(element));
    }

    /// Sets the root element of the window to the provided element.
    #[track_caller]
    pub fn set_root_element_boxed(&self, element: Box<dyn Element>) {
        self.state().set_root_element(element)
    }

    /// Requests the window to close.
    ///
    /// Note that the window will not be closed until the end of the current event loop iteration.
    ///
    /// # Remarks
    ///
    /// This method is safe to call multiple times. If the window has already been requested to
    /// close, this method will have no effect. If the window has already been closed, this method
    /// will have no effect either.
    pub fn close(&self) {
        if let Some(state) = self.0.upgrade() {
            state.close();
        }
    }

    /// Sets the title of the window.
    ///
    /// # Parameters
    ///
    /// * `title`: The new title of the window.
    #[track_caller]
    pub fn set_title(&self, title: &str) {
        self.state().os_window().set_title(title);
    }

    /// Requests the window to redraw its contents.
    #[track_caller]
    pub fn request_redraw(&self) {
        self.state().os_window().request_redraw();
    }
}

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Window")
    }
}
