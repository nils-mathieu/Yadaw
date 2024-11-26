use {
    crate::{private::AppState, UiResources, Window},
    std::{
        any::Any,
        fmt::Debug,
        rc::{Rc, Weak},
        time::{Duration, Instant},
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
    #[track_caller]
    pub fn exit(&self) {
        self.state().exit();
    }

    /// Requests a timed callback to be executed at the specified instant.
    #[track_caller]
    pub fn request_callback_at(&self, instant: Instant, callback: impl FnOnce() + 'static) {
        let b = Box::new(callback);
        self.state().request_callback(instant, b);
    }

    /// Requests a timed callback to be executed in the specified amount of time.
    ///
    /// # Remarks
    ///
    /// The callback will be executed after the specified duration has elapsed.
    #[track_caller]
    pub fn request_callback(&self, duration: Duration, callback: impl FnOnce() + 'static) {
        let b = Box::new(callback);
        let at = Instant::now() + duration;
        self.state().request_callback(at, b);
    }

    /// Calls the provided function with the [`UiResources`] instance.
    #[track_caller]
    pub fn with_resources_mut<R>(&self, f: impl FnOnce(&mut UiResources) -> R) -> R {
        f(&mut self.state().ui_resources().borrow_mut())
    }

    /// Calls the provided function with the [`UiResources`] instance.
    #[track_caller]
    pub fn with_resources<R>(&self, f: impl FnOnce(&UiResources) -> R) -> R {
        f(&self.state().ui_resources().borrow())
    }

    /// Inserts a new resource into the UI resources.
    ///
    /// # Returns
    ///
    /// This function returns the previous resource of the same type, if any.
    #[track_caller]
    pub fn insert_resource<T: Any>(&self, resource: T) -> Option<T> {
        self.with_resources_mut(|res| res.insert(resource))
    }

    /// Calls the provided function with the resource of type `T`.
    ///
    /// # Panics
    ///
    /// The function panics if the resource is not available.
    #[track_caller]
    pub fn with_resource<R, T: Any>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.with_resources(|res| f(res.get::<T>().expect("Resource not found")))
    }

    /// Calls the provided function with the resource of type `T`.
    ///
    /// # Panics
    ///
    /// The function panics if the resource is not available.
    #[track_caller]
    pub fn with_resource_mut<R, T: Any>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.with_resources_mut(|res| f(res.get_mut::<T>().expect("Resource not found")))
    }
}

impl Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("App")
    }
}
