use {
    crate::{CallbackId, private::WindowInner},
    rustc_hash::FxHashMap,
    slotmap::SlotMap,
    smallvec::SmallVec,
    std::{
        cell::{Cell, RefCell},
        rc::Rc,
        time::Instant,
    },
    winit::{
        event_loop::ActiveEventLoop,
        window::{WindowAttributes, WindowId},
    },
};

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

    /// A map used to transform window IDs to concrete [`WindowInner`] objects.
    windows: RefCell<FxHashMap<WindowId, Rc<WindowInner>>>,

    /// A collection of functions to be called at specific times.
    callbacks: RefCell<SlotMap<CallbackId, Callback>>,
    /// The time at which the next callback is scheduled to be called.
    next_callback_time: Cell<Option<Instant>>,
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
    pub fn create_window(self: &Rc<Self>, window: WindowAttributes) -> Rc<WindowInner> {
        let window = self.with_active_event_loop(|el| {
            el.create_window(window)
                .unwrap_or_else(|err| panic!("Failed to create new window: {err}"))
        });
        let id = window.id();

        let window_inner = Rc::new(WindowInner::new(self.clone(), window));
        self.windows.borrow_mut().insert(id, window_inner.clone());
        window_inner
    }

    /// Removes a window from the context.
    ///
    /// # Returns
    ///
    /// This function returns whether the window was successfully removed.
    pub fn remove_window(&self, id: WindowId) -> bool {
        self.windows.borrow_mut().remove(&id).is_some()
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
