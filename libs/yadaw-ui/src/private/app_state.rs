use {
    crate::{
        private::{Renderer, WindowState},
        UiResources,
    },
    hashbrown::HashMap,
    rustc_hash::FxBuildHasher,
    std::{
        cell::{Cell, RefCell},
        rc::{Rc, Weak},
        time::Instant,
    },
    vello::Scene,
    winit::{
        event_loop::{ActiveEventLoop, ControlFlow},
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

    /// The renderer responsible for rendering 2D graphics.
    ///
    /// # Remarks
    ///
    /// This field is left as a `None` value until the first window is created. This is because
    /// creating the renderer requires an existing window to be available.
    renderer: RefCell<Option<Renderer>>,

    /// The windows that are currently managed by the UI framework.
    ///
    /// The reference counted pointers here are supposed to be the only one with strong references,
    /// so that the windows can be destroyed easily by dropping that reference.
    windows: RefCell<HashMap<WindowId, Rc<WindowState>, FxBuildHasher>>,
    /// The current instant in time of the event loop.
    ///
    /// This does not change for a complete iteration.
    now: Cell<Instant>,

    /// The list of functions that need to be called at a specific time.
    #[allow(clippy::type_complexity)]
    timed_callbacks: RefCell<Vec<(Instant, Box<dyn FnOnce()>)>>,

    /// The resources that are available to the UI.
    ui_resources: RefCell<UiResources>,
}

impl AppState {
    /// Creates a new [`AppState`] instance.
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            active_event_loop: Cell::new(std::ptr::null()),
            renderer: RefCell::new(None),
            windows: RefCell::new(HashMap::default()),
            timed_callbacks: RefCell::new(Vec::new()),
            ui_resources: RefCell::new(UiResources::default()),
            now: Cell::new(Instant::now()),
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
    pub fn create_window(self: &Rc<Self>, mut attrs: WindowAttributes) -> Weak<WindowState> {
        let show_window = attrs.visible;
        attrs.visible = false;

        let ael = unsafe { self.active_event_loop() };
        let window = ael
            .create_window(attrs)
            .expect("Failed to create a new window");
        let wid = window.id();

        let mut renderer = self.renderer.borrow_mut();
        let (window, renderer) = if let Some(renderer) = renderer.as_mut() {
            (renderer.create_surface(window), renderer)
        } else {
            let (created_renderer, window) = Renderer::new_with_surface(window);
            (window, renderer.insert(created_renderer))
        };

        let window = WindowState::new(window, Rc::downgrade(self));

        if show_window {
            window.render(renderer, &mut Scene::new());
            window.os_window().set_visible(true);
        }

        let weak = Rc::downgrade(&window);
        self.windows.borrow_mut().insert(wid, window);
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

        for window in self.windows.borrow().values() {
            window.notify_end_of_event_loop_iteration();
        }
    }

    /// Returns the number of windows that are currently managed by the UI framework.
    pub fn window_count(&self) -> usize {
        self.windows.borrow().len()
    }

    /// Calls the provided function with the renderer.
    ///
    /// # Panics
    ///
    /// This function panics if the renderer is not available.
    pub fn with_renderer_mut<R>(&self, f: impl FnOnce(&mut Renderer) -> R) -> R {
        f(self
            .renderer
            .borrow_mut()
            .as_mut()
            .expect("The renderer is not available"))
    }

    /// Requests a timed callback to be executed at the specified instant.
    ///
    /// # Remarks
    ///
    /// The callback is guaranteed to be called *after* the specified instant. In other words,
    /// it is possible for the callback to be called later than requested.
    pub fn request_callback(&self, instant: Instant, f: Box<dyn FnOnce() + 'static>) {
        let ael = unsafe { self.active_event_loop() };
        self.timed_callbacks.borrow_mut().push((instant, f));
        if !matches!(ael.control_flow(), ControlFlow::WaitUntil(when) if when < instant) {
            ael.set_control_flow(ControlFlow::WaitUntil(instant));
        }
    }

    /// Dispatches the timed callbacks that are due.
    pub fn dispatch_callbacks(&self) {
        let now = Instant::now();
        let mut timed_callbacks = self.timed_callbacks.borrow_mut();

        let mut wake_up_at = None::<Instant>;

        let mut i = 0;
        while i < timed_callbacks.len() {
            if timed_callbacks[i].0 <= now {
                (timed_callbacks.swap_remove(i).1)();
            } else {
                if !matches!(wake_up_at, Some(when) if when < timed_callbacks[i].0) {
                    wake_up_at = Some(timed_callbacks[i].0);
                }

                i += 1;
            }
        }

        let ael = unsafe { self.active_event_loop() };
        match wake_up_at {
            Some(when) => ael.set_control_flow(ControlFlow::WaitUntil(when)),
            None => ael.set_control_flow(ControlFlow::Wait),
        }
    }

    /// Returns the resources that are available to the UI.
    #[inline]
    pub fn ui_resources(&self) -> &RefCell<UiResources> {
        &self.ui_resources
    }

    /// Sets the current instant of the event loop.
    #[inline]
    pub fn update_now(&self) {
        self.now.set(Instant::now());
    }

    /// Returns the current instant of the event loop.
    #[inline]
    pub fn now(&self) -> Instant {
        self.now.get()
    }
}
