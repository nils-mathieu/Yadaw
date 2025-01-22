//! This module is responsible for defining how events sent by the underlying platform's windowing
//! system should be handled by the application.

use {
    crate::{
        Ctx,
        event::{PointerButton, PointerEnetered, PointerLeft, PointerMoved},
        private::CtxInner,
    },
    std::rc::Rc,
    winit::{
        application::ApplicationHandler,
        event::{StartCause, WindowEvent},
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
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
    let el = EventLoop::builder()
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
pub fn run_with_event_loop(init_fn: InitFn, el: EventLoop) {
    // Ensures that the control flow is initially set to `Wait`.
    // This is done to override a potential user-provided control flow that we don't want.
    el.set_control_flow(ControlFlow::Wait);

    // Run the application to completion.
    el.run_app(&mut EventHandler::new(init_fn))
        .unwrap_or_else(|err| panic!("Failed to run the event loop: {err}"));
}

/// The [`ApplicationHandler`] implementation that will be passed to [`winit`] to handle
/// events for the application.
#[allow(
    clippy::large_enum_variant,
    reason = "The value of the type won't ever be moved around. It's always accessed by reference once created."
)]
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
    /// The scratch scene used when rendering a window.
    scratch_scene: vello::Scene,
}

impl AppState {
    /// Initializes the application state.
    pub fn initialize(el: &dyn ActiveEventLoop, init_fn: InitFn) -> Self {
        let ctx = Rc::new(CtxInner::default());
        ctx.set_active_event_loop(el, || init_fn(Ctx(Rc::downgrade(&ctx))));
        Self {
            ctx,
            scratch_scene: vello::Scene::default(),
        }
    }

    /// Handles a window event for the application.
    pub fn handle_window_event(
        &mut self,
        el: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.ctx.set_active_event_loop(el, || match event {
            WindowEvent::CloseRequested => {
                el.exit();
            }
            WindowEvent::SurfaceResized(new_size) => {
                self.ctx
                    .with_window(window_id, |window| window.notify_resized(new_size));
            }
            WindowEvent::RedrawRequested => {
                self.ctx.redraw_window(&mut self.scratch_scene, window_id);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.ctx.with_window(window_id, |window| {
                    window.notify_scale_factor_changed(scale_factor)
                });
            }
            WindowEvent::PointerMoved {
                position,
                device_id,
                primary,
                source,
            } => self.ctx.with_window(window_id, |window| {
                window.set_last_pointer_position(position);
                window.dispatch_event(&PointerMoved {
                    device_id,
                    primary,
                    source,
                    position: physical_position_to_point(position),
                });
            }),
            WindowEvent::PointerButton {
                device_id,
                state,
                position,
                primary,
                button,
            } => {
                self.ctx.with_window(window_id, |window| {
                    window.dispatch_event(&PointerButton {
                        device_id,
                        state,
                        primary,
                        button,
                        position: physical_position_to_point(position),
                    });
                });
            }
            WindowEvent::PointerLeft {
                device_id,
                position,
                primary,
                kind,
            } => {
                self.ctx.with_window(window_id, |window| {
                    if let Some(pos) = position {
                        window.set_last_pointer_position(pos);
                    }
                    window.dispatch_event(&PointerLeft {
                        device_id,
                        primary,
                        kind,
                    });
                });
            }
            WindowEvent::PointerEntered {
                device_id,
                position,
                primary,
                kind,
            } => {
                self.ctx.with_window(window_id, |window| {
                    window.set_last_pointer_position(position);
                    window.dispatch_event(&PointerEnetered {
                        device_id,
                        position: physical_position_to_point(position),
                        primary,
                        kind,
                    });
                });
            }
            _ => {}
        });
    }

    /// Notifies the application that the event loop has started running again.
    pub fn handle_start_cause(&mut self, el: &dyn ActiveEventLoop, cause: StartCause) {
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
    pub fn handle_about_to_wait(&mut self, el: &dyn ActiveEventLoop) {
        match self.ctx.next_callback_time() {
            Some(time) => el.set_control_flow(ControlFlow::WaitUntil(time)),
            None => el.set_control_flow(ControlFlow::Wait),
        }
    }
}

impl ApplicationHandler for EventHandler<'_> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        match self {
            Self::Uninitialized(_) => {
                // We can create surfaces now. Let's start initializing the application.
                let app_state = AppState::initialize(event_loop, self.take_user_init_fn());
                *self = Self::Initialized(app_state);
            }
            Self::Initializing => unreachable!(),
            Self::Initialized(_state) => {
                // FIXME: Restore the surfaces that were destroyed in `destroy_surfaces`.
            }
        }
    }

    fn destroy_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {
        // FIXME: Destroy surfaces here.
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Self::Initialized(state) = self {
            state.handle_window_event(event_loop, window_id, event);
        }
    }

    fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        if let Self::Initialized(state) = self {
            state.handle_start_cause(event_loop, cause);
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Self::Initialized(state) = self {
            state.handle_about_to_wait(event_loop);
        }
    }
}

/// Turns a physical position into a kurbo point.
#[inline]
fn physical_position_to_point(pos: winit::dpi::PhysicalPosition<f64>) -> vello::kurbo::Point {
    vello::kurbo::Point::new(pos.x, pos.y)
}
