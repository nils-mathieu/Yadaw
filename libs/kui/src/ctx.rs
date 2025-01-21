use {
    crate::{Window, private::CtxInner},
    slotmap::new_key_type,
    std::{
        fmt::Debug,
        rc::{Rc, Weak},
        time::{Duration, Instant},
    },
    winit::{event_loop::ActiveEventLoop, window::WindowAttributes},
};

new_key_type! {
    /// A key that is used to identify a callback within the application.
    pub struct CallbackId;
}

/// The global application context that is provided the user's UI code to interact with the
/// application.
///
/// # Remarks
///
/// Values of this type are only valid while the event loop is currently running. Once the event
/// loop has finished running, the context is no longer valid and any attempt to use it will
/// result in a panic.
#[derive(Clone)]
pub struct Ctx(pub(crate) Weak<CtxInner>);

impl Ctx {
    /// Returns whether the event loop is currently running.
    ///
    /// This is the only function of [`Ctx`] that won't panic if called after the event loop has
    /// finished running.
    #[inline]
    pub fn is_running(&self) -> bool {
        self.0.strong_count() > 0
    }

    /// Upgrades the context and panics if the event loop has finished running.
    #[track_caller]
    fn inner(&self) -> Rc<CtxInner> {
        self.0
            .upgrade()
            .expect("Attempted to use a `Ctx` after the event loop has finished running")
    }

    /// Stops the event loop and exits the application.
    #[track_caller]
    pub fn exit(&self) {
        self.inner().with_active_event_loop(ActiveEventLoop::exit);
    }

    /// Creates a new window with the provided attributes.
    #[track_caller]
    pub fn create_window(&self, attrs: WindowAttributes) -> Window {
        let inner = self.inner().create_window(attrs);
        Window(Rc::downgrade(&inner))
    }

    /// Calls the provided function at the specified time.
    ///
    /// The callback can be cancelled by calling [`cancel_callback`](Self::cancel_callback) with the
    /// returned ID.
    #[track_caller]
    pub fn call_at(&self, time: Instant, callback: impl FnOnce() + 'static) -> CallbackId {
        self.call_boxed_at(time, Box::new(callback))
    }

    /// Calls the provided function at the specified time.
    ///
    /// The callback can be cancelled by calling [`cancel_callback`](Self::cancel_callback) with the
    /// returned ID.
    #[track_caller]
    pub fn call_boxed_at(&self, time: Instant, callback: Box<dyn FnOnce()>) -> CallbackId {
        self.inner().register_callback(time, callback)
    }

    /// Calls the provided function after the specified duration.
    ///
    /// The callback can be cancelled by calling [`cancel_callback`](Self::cancel_callback) with the
    /// returned ID.
    #[track_caller]
    pub fn call_after(&self, duration: Duration, callback: impl FnOnce() + 'static) -> CallbackId {
        self.call_at(Instant::now() + duration, callback)
    }

    /// Calls the provided function after the specified duration.
    ///
    /// The callback can be cancelled by calling [`cancel_callback`](Self::cancel_callback) with the
    /// returned ID.
    #[track_caller]
    pub fn call_boxed_after(&self, duration: Duration, callback: Box<dyn FnOnce()>) -> CallbackId {
        self.call_boxed_at(Instant::now() + duration, callback)
    }

    /// Cancels a callback that was previously scheduled.
    ///
    /// # Returns
    ///
    /// This function returns whether the callback was successfully removed. Otherwise it wasn't
    /// found (either because it was already removed, or because it was called).
    #[track_caller]
    pub fn cancel_callback(&self, id: CallbackId) -> bool {
        self.inner().cancel_callback(id)
    }
}

impl Debug for Ctx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ctx { ... }")
    }
}
