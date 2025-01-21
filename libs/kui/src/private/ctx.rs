use {
    crate::{
        CallbackId,
        private::{Renderer, WindowAndSurface, WindowInner},
    },
    rustc_hash::FxHashMap,
    slotmap::SlotMap,
    smallvec::SmallVec,
    std::{
        any::{Any, TypeId},
        cell::{Cell, RefCell},
        rc::Rc,
        time::Instant,
    },
    winit::{
        event_loop::ActiveEventLoop,
        window::{WindowAttributes, WindowId},
    },
};

/// A map that holds at most one instance of every static type.
#[derive(Default)]
pub struct TypeMap(FxHashMap<TypeId, Box<dyn Any>>);

impl TypeMap {
    /// If a value of type `T` exists in the map, returns it. Otherwise, inserts the provided value
    /// and returns a mutable reference to it.
    pub fn get_or_insert_with<T: Any>(&mut self, fallback: impl FnOnce() -> T) -> &mut T {
        let id = TypeId::of::<T>();
        let b = self.0.entry(id).or_insert_with(|| Box::new(fallback()));
        unsafe { b.downcast_mut_unchecked() }
    }

    /// If a value of type `T` exists in the map, returns it. Otherwise, inserts a default value
    /// and returns a mutable reference to it.
    pub fn get_or_insert_default<T>(&mut self) -> &mut T
    where
        T: Any + Default,
    {
        self.get_or_insert_with(Default::default)
    }

    /// Attempts to get a vlaue of type `T` from the map.
    ///
    /// If the value is not available, returns `None`.
    pub fn get<T: Any>(&self) -> Option<&T> {
        let id = TypeId::of::<T>();
        self.0
            .get(&id)
            .map(|b| unsafe { b.downcast_ref_unchecked() })
    }

    /// Attempts to get a mutable reference to a value of type `T` from the map.
    ///
    /// If the value is not available, returns `None`.
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let id = TypeId::of::<T>();
        self.0
            .get_mut(&id)
            .map(|b| unsafe { b.downcast_mut_unchecked() })
    }
}

/// Information about a callback that is scheduled to be called at a specific time.
struct Callback {
    /// The callback to be called.
    ///
    /// This is an option in order to tell `.retain` whether it needs to drop the callback or not.
    callback: Option<Box<dyn FnOnce()>>,
    /// The instant at which the callback is scheduled.
    ///
    /// The callback is guaranteed to be called at this time or later.
    time: Instant,
}

/// Just a simple structure that holds the windows and the renderer.
///
/// This avoids having multiple `RefCell` objects for stuff that will always be used together.
#[derive(Default)]
struct RendererAndWindows {
    /// The renderer responsible for drawing stuff on the screen.
    ///
    /// It is only created when the first window is created because it needs to know the window
    /// in order to initialize itself.
    renderer: Option<Renderer>,
    /// A map used to transform window IDs to concrete [`WindowInner`] objects.
    windows: FxHashMap<WindowId, Rc<WindowInner>>,
}

/// The inner state of [`Ctx`](crate::Ctx).
#[derive(Default)]
pub struct CtxInner {
    /// The active event loop object.
    ///
    /// # Safety
    ///
    /// If the inner option is `None`, it means that no `ActiveEventLoop` object is available
    /// right now.
    ///
    /// If the inner option is `Some(_)`, it means that the `ActiveEventLoop` object is available
    /// and the reference will remain valid as long as the inner option remains a `Some(_)` with
    /// the same reference.
    active_event_loop: Cell<Option<&'static ActiveEventLoop>>,

    /// The renderer and the windows.
    renderer_and_windows: RefCell<RendererAndWindows>,

    /// A collection of functions to be called at specific times.
    callbacks: RefCell<SlotMap<CallbackId, Callback>>,
    /// The time at which the next callback is scheduled to be called.
    next_callback_time: Cell<Option<Instant>>,

    /// Some global resources which may be used by the user.
    resources: RefCell<TypeMap>,
}

impl CtxInner {
    //
    // MISC STATE MANAGEMENT
    //

    /// Calls the provided closure.
    ///
    /// During the function call, the internal `active_event_loop` field is initialized with the
    /// provided reference.
    ///
    /// When the function returns (or panics), the internal `active_event_loop` field is reset to
    /// `None` to ensure it is not used past its lifetime.
    pub fn set_active_event_loop<R>(&self, ael: &ActiveEventLoop, f: impl FnOnce() -> R) -> R {
        // Calling this function re-entrantly is not a safety concern, but it's probably
        // an error on our part.
        debug_assert!(self.active_event_loop.get().is_none());

        struct Guard<'a>(&'a Cell<Option<&'static ActiveEventLoop>>);

        impl Drop for Guard<'_> {
            #[inline]
            fn drop(&mut self) {
                self.0.set(None);
            }
        }

        // SAFETY: This is safe because the guard above will make sure the reference is retired
        // from the field before the function returns.
        let ael = unsafe { extend_lifetime(ael) };

        self.active_event_loop.set(Some(ael));
        let _guard = Guard(&self.active_event_loop);
        f()
    }

    /// Calls the provided function with the stored [`ActiveEventLoop`] reference.
    ///
    /// # Panics
    ///
    /// This function panics if the internal `active_event_loop` field is not set.
    pub fn with_active_event_loop<R>(&self, f: impl FnOnce(&ActiveEventLoop) -> R) -> R {
        // SAFETY: The function will not be able to leak the reference because the lifetime we
        // provide is only valid during its execution.
        let ael = self
            .active_event_loop
            .get()
            .expect("No active event loop available");

        f(ael)
    }

    //
    // WINDOWS
    //

    /// Creates a new window and returns its ID.
    pub fn create_window(self: &Rc<Self>, mut window: WindowAttributes) -> Rc<WindowInner> {
        // We only want to show the window once we have rendered a full frame for it.
        let show_window = window.visible;
        window.visible = false;

        let window = self.with_active_event_loop(|el| {
            el.create_window(window)
                .unwrap_or_else(|err| panic!("Failed to create new window: {err}"))
        });
        let id = window.id();

        let mut renderer_and_windows = self.renderer_and_windows.borrow_mut();
        let RendererAndWindows { renderer, windows } = &mut *renderer_and_windows;

        let window_and_surface;
        match renderer {
            Some(renderer) => window_and_surface = WindowAndSurface::new(renderer, window),
            None => {
                let renderer_val;
                (renderer_val, window_and_surface) = Renderer::new_for_window(window);
                *renderer = Some(renderer_val);
            }
        };

        let window_inner = Rc::new(WindowInner::new(self.clone(), window_and_surface));

        if show_window {
            let mut scene = vello::Scene::new();
            window_inner.draw_to_scene(&mut scene);
            window_inner.render_scene(renderer.as_mut().unwrap(), &scene);
            window_inner.winit_window().set_visible(true);
        }

        windows.insert(id, window_inner.clone());
        window_inner
    }

    /// Requests a particular window to redraw itself.
    ///
    /// # Panics
    ///
    /// This function panics if the window with the provided ID does not exist.
    #[track_caller]
    pub fn redraw_window(&self, scratch_scene: &mut vello::Scene, window_id: WindowId) {
        let window = self
            .renderer_and_windows
            .borrow_mut()
            .windows
            .get(&window_id)
            .expect("Window ID not found")
            .clone();

        window.draw_to_scene(scratch_scene);

        let mut renderer_and_windows = self.renderer_and_windows.borrow_mut();
        let RendererAndWindows { renderer, windows } = &mut *renderer_and_windows;
        windows
            .get(&window_id)
            .unwrap()
            .render_scene(renderer.as_mut().unwrap(), scratch_scene);
    }

    /// Calls the provided function with a reference to the window with the provided ID.
    ///
    /// # Panics
    ///
    /// This function panics if the window with the provided ID does not exist.
    #[track_caller]
    pub fn with_window<R>(&self, id: WindowId, f: impl FnOnce(&WindowInner) -> R) -> R {
        f(self
            .renderer_and_windows
            .borrow()
            .windows
            .get(&id)
            .expect("Window ID not found"))
    }

    /// Removes a window from the context.
    ///
    /// # Returns
    ///
    /// This function returns whether the window was successfully removed.
    pub fn remove_window(&self, id: WindowId) -> bool {
        self.renderer_and_windows
            .borrow_mut()
            .windows
            .remove(&id)
            .is_some()
    }

    //
    // CALLBACKS
    //

    /// Updates the `next_callback_time` field with the provided time if it is earlier than the
    /// current value.
    fn request_callback_at(&self, time: Instant) {
        match self.next_callback_time.get() {
            Some(next) => {
                if time < next {
                    self.next_callback_time.set(Some(time));
                }
            }
            None => {
                self.next_callback_time.set(Some(time));
            }
        }
    }

    /// Returns the next callback time.
    #[inline]
    pub fn next_callback_time(&self) -> Option<Instant> {
        self.next_callback_time.get()
    }

    /// Registers a callback into the context.
    pub fn register_callback(&self, time: Instant, callback: Box<dyn FnOnce()>) -> CallbackId {
        self.request_callback_at(time);
        self.callbacks.borrow_mut().insert(Callback {
            callback: Some(callback),
            time,
        })
    }

    /// Cancels a callback that was previously scheduled.
    ///
    /// # Returns
    ///
    /// This function returns whether the callback was successfully removed.
    pub fn cancel_callback(&self, id: CallbackId) -> bool {
        self.callbacks.borrow_mut().remove(id).is_some()
    }

    /// Runs the callbacks that were scheduled to be called before `now`.
    pub fn run_callbacks(&self, now: Instant) {
        // NOTE: This part is a bit annoying because the actual callbacks we're running here
        // might themselves schedule new callbacks. For this reason, we need to make sure that
        // any time we run a callback, we can't be holding a reference to the `callbacks` field.

        // We reached our own callback time, so we can reset the next callback time.
        self.next_callback_time.set(None);

        let mut next_instant: Option<Instant> = None;
        let mut ready_callbacks = SmallVec::<[Box<dyn FnOnce()>; 4]>::new();

        self.callbacks.borrow_mut().retain(|_id, callback| {
            if callback.time <= now {
                ready_callbacks.reserve(1);

                // SAFETY: Callbacks that have been removed are never retained in the map.
                // Even the next `push` operation won't fail because we reserved enough space
                // before taking the callback object.
                let cb = unsafe { callback.callback.take().unwrap_unchecked() };

                ready_callbacks.push(cb);
                false
            } else {
                next_instant = match next_instant {
                    Some(next) => Some(next.min(callback.time)),
                    None => Some(callback.time),
                };
                true
            }
        });

        // Actually execute the callbacks now that we're not holding the `callbacks` lock.
        ready_callbacks.into_iter().for_each(|cb| cb());

        if let Some(next) = next_instant {
            // We can't just override the value because it's possible that a new earlier
            // callback was scheduled while we were running the current ones.
            self.request_callback_at(next);
        }
    }

    /// Calls the provided function with the resources map.
    #[track_caller]
    pub fn with_resources_mut<R>(&self, f: impl FnOnce(&mut TypeMap) -> R) -> R {
        f(&mut self.resources.borrow_mut())
    }

    /// Calls the provided function with the resources map.
    #[track_caller]
    pub fn with_resources<R>(&self, f: impl FnOnce(&TypeMap) -> R) -> R {
        f(&self.resources.borrow())
    }
}

/// Extends the lifetime of the provided reference to an arbitrary (and potentially longer)
/// lifetime.
///
/// # Safety
///
/// The caller must responsible for ensuring the lifetime of the returned reference is not used
/// past the lifetime of the original reference.
#[inline(always)]
unsafe fn extend_lifetime<'unconstrained, T>(a: &T) -> &'unconstrained T {
    unsafe { std::mem::transmute(a) }
}
