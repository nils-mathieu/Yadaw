//! This module is responsible for defining how events sent by the underlying platform's windowing
//! system should be handled by the application.

use {
    crate::{Ctx, private::CtxInner},
    std::rc::Rc,
    winit::{
        application::ApplicationHandler,
        event::{StartCause, WindowEvent},
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        keyboard::KeyCode,
        window::WindowId,
    },
};

/// The signature of the initialization function that is called when the application initially
/// starts.
pub type InitFn<'a> = Box<dyn FnOnce(Ctx) + 'a>;

/// Runs the Kui application.
///
/// See [`run`](crate::run) for more information.
pub fn run(init_fn: InitFn) {
    let el = EventLoop::<KuiEvent>::with_user_event()
        .build()
        .unwrap_or_else(|err| panic!("Failed to create the event loop: {err}"));
    run_with_event_loop(init_fn, el);
}

/// Runs the Kui application with a custom [`EventLoop`] value.
///
/// This is useful when you need to provide a custom event loop to the application (for example
/// on Android or the web).
///
/// Otherwise the semantics of this function are the same as [`run`](crate::run). Check the
/// documentation of that function for more information.
pub fn run_with_event_loop(init_fn: InitFn, el: EventLoop<KuiEvent>) {
    // Ensures that the control flow is initially set to `Wait`.
    // This is done to override a potential user-provided control flow that we don't want.
    el.set_control_flow(ControlFlow::Wait);

    // Run the application to completion.
    el.run_app(&mut EventHandler::new(init_fn))
        .unwrap_or_else(|err| panic!("Failed to run the event loop: {err}"));
}

/// The custom event type used by the Kui event loop.
pub struct KuiEvent(pub(crate) KuiEventInner);

/// The inner enumeration that is actually stored in the [`KuiEvent`] struct.
///
/// This is done to avoid the implementation details of [`KuiEvent`] to the user.
pub(crate) enum KuiEventInner {}

/// The [`ApplicationHandler`] implementation that will be passed to [`winit`] to handle
/// events for the application.
enum EventHandler<'a> {
    /// The event handler has not been initialized yet.
    ///
    /// The user's initialization function has not been called yet.
    Uninitialized(InitFn<'a>),

    /// The event handler is being initialized. The user's initialization function has been
    /// removed from the event handler to be called (and therefor consumed).
    Initializing,

    /// The initialized app state that is stored in the event handler.
    Initialized(AppState),
}

impl<'a> EventHandler<'a> {
    /// Creates a new [`EventHandler`] in the `Uninitialized` state.
    pub fn new(init_fn: InitFn<'a>) -> Self {
        Self::Uninitialized(init_fn)
    }

    /// Takes the user's initialization function out of the event handler, consuming it.
    ///
    /// This function assumes that the current state of the application is `Uninitialized`. If it
    /// isn't, the function will panic, leaving the event handler in the `Initializing` state.
    pub fn take_user_init_fn(&mut self) -> InitFn<'a> {
        match std::mem::replace(self, EventHandler::Initializing) {
            EventHandler::Uninitialized(f) => f,
            _ => unreachable!(),
        }
    }
}

/// The state of the application once it has been properly initialized.
struct AppState {
    /// The global application context.
    ctx: Rc<CtxInner>,
}

impl AppState {
    /// Initializes the application state.
    pub fn initialize(el: &ActiveEventLoop, init_fn: InitFn) -> Self {
        let ctx = Rc::new(CtxInner::default());
        ctx.set_active_event_loop(el, || init_fn(Ctx(Rc::downgrade(&ctx))));
        Self { ctx }
    }

    /// Handles a window event for the application.
    pub fn handle_window_event(
        &mut self,
        el: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.ctx.set_active_event_loop(el, || match event {
            WindowEvent::CloseRequested => {
                el.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() && event.physical_key == KeyCode::Escape {
                    el.exit();
                }
            }
            _ => {}
        });
    }

    /// Notifies the application that the event loop has started running again.
    pub fn handle_start_cause(&mut self, el: &ActiveEventLoop, cause: StartCause) {
        self.ctx.set_active_event_loop(el, || {
            if let StartCause::ResumeTimeReached {
                requested_resume, ..
            } = cause
            {
                self.ctx.run_callbacks(requested_resume);
            }
        });
    }

    /// Notifies the application that the event loop is about to start blocking for new events.
    pub fn handle_about_to_wait(&mut self, _el: &ActiveEventLoop) {
        // self.ctx.set_active_event_loop(el, || {});
    }
}

impl ApplicationHandler<KuiEvent> for EventHandler<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self {
            Self::Uninitialized(_) => {
                // The event loop is being started (first `resume` event). We need to call the
                // function that the user has defined in order to initialize whatever state they
                // need.
                let app_state = AppState::initialize(event_loop, self.take_user_init_fn());
                *self = Self::Initialized(app_state);
            }
            Self::Initializing => unreachable!(),
            Self::Initialized(_state) => {
                // The event loop is being resumed while running. Depending on the platform, we may
                // or may not need to do anything here.

                // TODO: On platforms that need it, invalidate the surfaces here.
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Self::Initialized(state) = self {
            state.handle_window_event(event_loop, window_id, event);
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if let Self::Initialized(state) = self {
            state.handle_start_cause(event_loop, cause);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Self::Initialized(state) = self {
            state.handle_about_to_wait(event_loop);
        }
    }
}
