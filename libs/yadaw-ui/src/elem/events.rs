use {
    crate::element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    std::marker::PhantomData,
    vello::{kurbo::Point, Scene},
};

/// An element that hooks a function into the event system.
pub struct HookEvents<F, E: ?Sized> {
    /// The function that will be called when the event is caught.
    pub on_event: F,
    /// The child element.
    pub child: E,
}

impl<F, E> HookEvents<F, E> {
    /// Creates a new `HookEvents` element.
    pub fn new(on_event: F, child: E) -> Self
    where
        F: FnMut(&mut E, &ElemCtx, &dyn Event) -> EventResult,
    {
        Self { on_event, child }
    }
}

impl<F, E> Element for HookEvents<F, E>
where
    F: FnMut(&mut E, &ElemCtx, &dyn Event) -> EventResult,
    E: Element,
{
    #[inline]
    fn ready(&mut self, cx: &ElemCtx) {
        self.child.ready(cx);
    }

    #[inline]
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        self.child.set_size(cx, size);
    }

    #[inline]
    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.child.set_position(cx, position);
    }

    #[inline]
    fn metrics(&mut self, cx: &ElemCtx) -> Metrics {
        self.child.metrics(cx)
    }

    #[inline]
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        self.child.render(cx, scene);
    }

    #[inline]
    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        self.child.hit_test(cx, point)
    }

    #[inline]
    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        if (self.on_event)(&mut self.child, cx, event).should_stop_propagation() {
            EventResult::StopPropagation
        } else {
            self.child.event(cx, event)
        }
    }
}

/// An element that hooks a function into the event system.
pub struct CatchEvent<T, F, E: ?Sized> {
    /// The type of event that will be caught.
    _marker: PhantomData<fn(T)>,
    /// The function that will be called when the event is caught.
    pub on_event: F,
    /// The child element.
    pub child: E,
}

impl<T, F, E> CatchEvent<T, F, E> {
    /// Creates a new `CatchEvent` element.
    pub fn new(on_event: F, child: E) -> Self
    where
        F: FnMut(&mut E, &ElemCtx, &T) -> EventResult,
    {
        Self {
            _marker: PhantomData,
            on_event,
            child,
        }
    }
}

impl<T, F, E> Element for CatchEvent<T, F, E>
where
    T: 'static,
    F: FnMut(&mut E, &ElemCtx, &T) -> EventResult,
    E: Element,
{
    #[inline]
    fn ready(&mut self, cx: &ElemCtx) {
        self.child.ready(cx);
    }

    #[inline]
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        self.child.set_size(cx, size);
    }

    #[inline]
    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.child.set_position(cx, position);
    }

    #[inline]
    fn metrics(&mut self, cx: &ElemCtx) -> Metrics {
        self.child.metrics(cx)
    }

    #[inline]
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        self.child.render(cx, scene);
    }

    #[inline]
    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        self.child.hit_test(cx, point)
    }

    #[inline]
    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        if let Some(event) = event.downcast() {
            if (self.on_event)(&mut self.child, cx, event).should_stop_propagation() {
                return EventResult::StopPropagation;
            }
        }

        self.child.event(cx, event)
    }
}

/// An element that runs a function on every frame
/// to update the child element in tis rendering logic.
///
/// Useful for animations.
pub struct HookAnimation<F, E: ?Sized> {
    /// If the animation is currently running,
    /// this will be `Some` with the last instant the animation was updated.
    last_instant: Option<std::time::Instant>,

    /// The callback to run.
    callback: F,

    /// The child element.
    pub child: E,
}

impl<F, E> HookAnimation<F, E> {
    /// Creates a new `HookAnimation` element.
    pub fn new(callback: F, child: E) -> Self
    where
        F: FnMut(&mut E, &ElemCtx, f64) -> bool,
    {
        Self {
            last_instant: None,
            callback,
            child,
        }
    }

    /// Starts the animation right now.
    #[inline]
    pub fn start_animation(&mut self, cx: &ElemCtx) {
        if self.last_instant.is_none() {
            self.last_instant = Some(cx.app().now());
            cx.window().request_redraw();
        }
    }
}

impl<F, E> Element for HookAnimation<F, E>
where
    F: FnMut(&mut E, &ElemCtx, f64) -> bool,
    E: ?Sized + Element,
{
    #[inline]
    fn ready(&mut self, cx: &ElemCtx) {
        self.child.ready(cx);
    }

    #[inline]
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        self.child.set_size(cx, size);
    }

    #[inline]
    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.child.set_position(cx, position);
    }

    #[inline]
    fn metrics(&mut self, cx: &ElemCtx) -> Metrics {
        self.child.metrics(cx)
    }

    #[inline]
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        if let Some(last_instant) = self.last_instant {
            let delta = cx.app().now().duration_since(last_instant).as_secs_f64();
            if (self.callback)(&mut self.child, cx, delta) {
                cx.window().request_redraw();
                self.last_instant = Some(cx.app().now());
            } else {
                self.last_instant = None;
            }
        }

        self.child.render(cx, scene);
    }

    #[inline]
    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        self.child.hit_test(cx, point)
    }

    #[inline]
    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        self.child.event(cx, event)
    }
}
