use {
    crate::{private::AppState, Window},
    std::{
        fmt::Debug,
        rc::{Rc, Weak},
    },
    winit::window::WindowAttributes,
};

/// The application context of the UI framework.
///
/// This type is cheap to be cloned and used in multiple places. It can be used to access the
/// runtime, create windows, and manage the global state of the application.
#[derive(Clone)]
pub struct App(Weak<AppState>);

impl App {
    /// Creates a new [`App`] instance from the provided [`AppState`] reference.
    #[inline]
    pub(crate) fn from_state(state: Weak<AppState>) -> Self {
        Self(state)
    }

    /// Upgrades the reference counted pointer to the application state.
    ///
    /// Panics if the application context has been destroyed.
    #[track_caller]
    fn state(&self) -> Rc<AppState> {
        self.0
            .upgrade()
            .expect("The application context has been destroyed")
    }

    /// Checks whether the application context is still valid. In other words, this function checks
    /// whether the UI thread is still running.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0.strong_count() != 0
    }

    /// Creates a new window.
    #[track_caller]
    pub fn create_window(&self, attrs: WindowAttributes) -> Window {
        Window::from_state(self.state().create_window(attrs))
    }

    /// Exits the event loop.
    ///
    /// # Remarks
    ///
    /// It is possible for more events to be dispatched after this function has been called.
    /// However, the runtime will exit as soon as the control flow reaches the end of the current
    /// event loop iteration.
    #[inline]
    #[track_caller]
    pub fn exit(&self) {
        self.state().exit();
    }
}

impl Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("App")
    }
}
