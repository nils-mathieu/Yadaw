use {
    crate::private::WindowState,
    rustc_hash::FxHashMap,
    std::{
        cell::{Cell, RefCell},
        rc::{Rc, Weak},
    },
    winit::{
        event_loop::ActiveEventLoop,
        window::{WindowAttributes, WindowId},
    },
};

/// Defines the global application state.
///
/// This type is meant to be used inside of a reference counted pointer.
pub struct AppState {
    /// The active event loop of the application.
    ///
    /// Because references to [`ActiveEventLoop`]s are only available for a short period of time
    /// (e.g. during a single callback of the `winit` runtime), we need to make sure that those
    /// references are always valid.
    ///
    /// When this raw pointer is null, it means that the [`ActiveEventLoop`] is not available. When
    /// it is not null, then the reference is valid. It is important not to leak references to
    /// [`ActiveEventLoop`]s outside of the scope of the callback that provided the reference
    /// (e.g. by storing it in a global variable).
    active_event_loop: Cell<*const ActiveEventLoop>,

    /// The windows that are currently managed by the UI framework.
    ///
    /// The reference counted pointers here are supposed to be the only one with strong references,
    /// so that the windows can be destroyed easily by dropping that reference.
    windows: RefCell<FxHashMap<WindowId, Rc<WindowState>>>,
}

impl AppState {
    /// Creates a new [`AppState`] instance.
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            active_event_loop: Cell::new(std::ptr::null()),
            windows: RefCell::new(FxHashMap::default()),
        })
    }

    /// Calls the provided function while the `active_event_loop` field set to the provided value.
    ///
    /// This function is not unsafe to call because there is no way to use it incorrectly. However,
    /// it is important to note that accessing the `active_event_loop` field must be done with care.
    /// Specifically, leaking the pointer once it has been cleared will result in undefined
    /// behavior.
    pub fn with_active_event_loop<R>(&self, ael: &ActiveEventLoop, f: impl FnOnce() -> R) -> R {
        struct Guard<'a>(&'a Cell<*const ActiveEventLoop>);

        impl Drop for Guard<'_> {
            #[inline]
            fn drop(&mut self) {
                self.0.set(std::ptr::null());
            }
        }

        self.active_event_loop.set(ael);
        let _guard = Guard(&self.active_event_loop);
        f()
    }

    /// Returns the content of the `active_event_loop` field.
    ///
    /// # Panics
    ///
    /// This function panics if the `active_event_loop` field is null.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the returned reference is not used past the lifetime
    /// of the [`ActiveEventLoop`] that it points to.
    ///
    /// This is usually easy enough as long as the reference does not cross function boundaries.
    #[inline]
    #[track_caller]
    pub unsafe fn active_event_loop(&self) -> &ActiveEventLoop {
        let p = self.active_event_loop.get();

        assert!(!p.is_null(), "The active event loop is not available");

        &*p
    }

    /// Requests the event loop to exit.
    #[inline]
    pub fn exit(&self) {
        let ael = unsafe { self.active_event_loop() };
        ael.exit();
    }

    /// Creates a new window with the provided attributes.
    ///
    /// # Panics
    ///
    /// This function panics if the [`ActiveEventLoop`] is not available.
    pub fn create_window(self: &Rc<Self>, attrs: WindowAttributes) -> Weak<WindowState> {
        let ael = unsafe { self.active_event_loop() };
        let window = ael
            .create_window(attrs)
            .expect("Failed to create a new window");
        let wid = window.id();
        let state = WindowState::new(window);
        let weak = Rc::downgrade(&state);
        self.windows.borrow_mut().insert(wid, state);
        weak
    }

    /// Gets a window by its ID.
    pub fn get_window(&self, wid: WindowId) -> Option<Rc<WindowState>> {
        self.windows.borrow().get(&wid).cloned()
    }

    /// Notifies the application state that the event loop iteration has ended.
    pub fn notify_end_of_event_loop_iteration(&self) {
        self.windows
            .borrow_mut()
            .retain(|_, state| !state.closing());
    }

    /// Returns the number of windows that are currently managed by the UI framework.
    pub fn window_count(&self) -> usize {
        self.windows.borrow().len()
    }
}
