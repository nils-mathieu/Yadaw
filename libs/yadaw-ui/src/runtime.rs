//! This module provides access to the runtime of the Yadaw digital audio workstation.
//!
//! The runtime is mainly responsible for managing the main event loop of the application, which
//! includes handling window events, input events, and rendering the user interface to the screen.
//! This can be thought of as the UI thread of the application, where all the user interface logic
//! is executed.
//!
//! # Entry Points
//!
//! The main entry point of the runtime is the [`run`] function, which is responsible for starting
//! the main event loop of the application.

use {
    crate::{event, private::AppState, App},
    std::rc::Rc,
    vello::{kurbo::Point, Scene},
    winit::{
        application::ApplicationHandler,
        event::{StartCause, WindowEvent},
        event_loop::{ActiveEventLoop, EventLoop},
        window::WindowId,
    },
};

/// The runtime of the application.
type InitFn<'a> = Box<dyn FnOnce(App) + 'a>;

/// The custom event type used by the `winit` event loop.
///
/// This type will be used to send custom events to the main event loop of the application
/// from other parts of the codebase.
#[derive(Debug)]
struct UiEvent;

/// Starts the main event loop of the application.
///
/// # Panics
///
/// This function will panic if the main event loop fails to start for any reason.
///
/// # Returns
///
/// This function returns once the runtime is exited.
///
/// Note that on some platforms, the runtime may not actually give the control back to the caller
/// (e.g. on iOS, the runtime will never return). In those cases, the function diverges.
pub fn run<F>(f: F)
where
    F: FnOnce(App),
{
    fn inner(init: InitFn) {
        EventLoop::<UiEvent>::with_user_event()
            .build()
            .expect("Failed to create the main event loop")
            .run_app(&mut WinitApp::new(init))
            .expect("Failed to run the main event loop");
    }

    inner(Box::new(f))
}

/// This type will be used to create the main application instance.
struct WinitApp<'a> {
    /// The initialization function provided by the user.
    ///
    /// This initialization function will be called once the main event loop is started (in the
    /// [`resumed`](WinitApp::resumed) event).
    ///
    /// We can't invoke this function directly because it requires an [`ActiveEventLoop`] reference
    /// to be available in order to create new windows. This is because some platforms require
    /// windows and surfaces to be created while the event loop is already running.
    ///
    /// Until the initilization function is invoked, this option will remain filled. When the
    /// function is called, it will be replaced with `None`.
    init_fn: Option<InitFn<'a>>,

    /// The global application state.
    app_state: Rc<AppState>,

    /// The scene that will be used to render the windows. Instead of re-creating the scene
    /// every time a window needs to be rendered, we can re-use the same scene for all windows,
    /// re-using the same resources and allocations.
    scratch_scene: Scene,
}

impl<'a> WinitApp<'a> {
    /// Creates a new [`WinitApp`] instance from the provided initialization function.
    pub fn new(init_fn: InitFn<'a>) -> Self {
        Self {
            init_fn: Some(init_fn),
            app_state: AppState::new(),
            scratch_scene: Scene::new(),
        }
    }
}

impl ApplicationHandler<UiEvent> for WinitApp<'_> {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.app_state.with_active_event_loop(el, || {
            if let Some(init_fn) = self.init_fn.take() {
                init_fn(App::from_state(Rc::downgrade(&self.app_state)));
            }
        });
    }

    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, ev: WindowEvent) {
        self.app_state.with_active_event_loop(el, || {
            if let Some(window) = self.app_state.get_window(wid) {
                // TODO: Move the event handling logic to the element tree.
                match ev {
                    WindowEvent::Resized(new_size) => {
                        window.set_size(new_size);
                    }
                    WindowEvent::RedrawRequested => {
                        self.app_state.with_renderer_mut(|renderer| {
                            window.render(renderer, &mut self.scratch_scene);
                        });
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        window.set_scale_factor(scale_factor);
                    }
                    WindowEvent::CloseRequested => {
                        window.dispatch_event(&event::CloseRequested);
                    }
                    WindowEvent::KeyboardInput {
                        device_id,
                        event,
                        is_synthetic,
                    } => {
                        window.dispatch_event(&event::KeyboardInput {
                            device_id,
                            is_synthetic,
                            inner: event,
                        });
                    }
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                    } => {
                        window.dispatch_event(&event::MouseInput {
                            device_id,
                            state,
                            button,
                        });
                    }
                    WindowEvent::MouseWheel {
                        device_id,
                        delta,
                        phase,
                    } => {
                        window.dispatch_event(&event::WheelInput {
                            device_id,
                            delta,
                            phase,
                        });
                    }
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                    } => {
                        let position = Point::new(position.x, position.y);
                        window.set_last_reported_cursor_position(Some(position));
                        window.dispatch_event(&event::CursorMoved {
                            device_id,
                            position,
                        });
                    }
                    WindowEvent::CursorEntered { .. } => {
                        window.set_cursor_inside(true);
                    }
                    WindowEvent::CursorLeft { .. } => {
                        window.set_cursor_inside(false);
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        window.set_modifiers(modifiers.state());
                    }
                    WindowEvent::PinchGesture {
                        device_id,
                        mut delta,
                        phase,
                    } => {
                        if delta.is_nan() {
                            delta = 0.0;
                        }

                        window.dispatch_event(&event::PinchGesture {
                            device_id,
                            delta,
                            phase,
                        });
                    }
                    _ => (),
                }
            }
        });
    }

    fn new_events(&mut self, el: &ActiveEventLoop, _cause: StartCause) {
        self.app_state.with_active_event_loop(el, || {
            self.app_state.update_now();
            self.app_state.dispatch_callbacks();
        });
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        self.app_state.with_active_event_loop(el, || {
            self.app_state.notify_end_of_event_loop_iteration();
            if self.app_state.window_count() == 0 {
                el.exit();
            }
        });
    }
}
