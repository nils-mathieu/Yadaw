//! Basic implementations of the [`Element`] trait.
//!
//! [`Element`]: crate::element::Element

pub mod utils;

pub mod shapes;
pub use self::shapes::{ClipChild, ShapeElement, SolidShape, WithBackground};

pub mod text;
pub use self::text::Text;

pub mod linear_layout;
pub use self::linear_layout::LinearLayout;

pub mod elements;
pub use self::elements::Elements;

mod with_margin;
pub use self::with_margin::WithMargin;

mod length;
pub use self::length::Length;

mod lazy_linear_layout;
pub use self::lazy_linear_layout::LazyLinearLayout;

mod with_scroll;
pub use self::with_scroll::WithScroll;

mod events;
pub use self::events::{CatchEvent, HookAnimation, HookEvents, HookReady};

mod with_default_size;
pub use self::with_default_size::WithDefaultSize;

mod with_cursor;
pub use self::with_cursor::WithCursor;

mod empty;
pub use self::empty::Empty;

mod with_data;
pub use self::with_data::WithData;

mod transform;
pub use self::transform::Translate;

use {
    crate::element::{ElemCtx, Element, Event, EventResult},
    std::{cell::RefCell, rc::Rc},
    vello::{kurbo::Vec2, peniko::Brush},
    winit::window::CursorIcon,
};

/// An extension trait for elements.
pub trait ElementExt: Sized + Element {
    /// Hooks a fucntion into the element's event handling.
    fn on_any_event<F>(self, f: F) -> HookEvents<F, Self>
    where
        F: FnMut(&mut Self, &ElemCtx, &dyn Event) -> EventResult,
    {
        HookEvents::new(f, self)
    }

    /// Hooks a function into the element's event handling, capturing a specific event type.
    fn on_event<T, F>(self, f: F) -> CatchEvent<T, F, Self>
    where
        F: FnMut(&mut Self, &ElemCtx, &T) -> EventResult,
    {
        CatchEvent::new(f, self)
    }

    /// Hooks a function in the rendering logic of the element, allowing for custom animations
    /// effects.
    ///
    /// The animation must be initiated by calling [`start_animation`](HookAnimation::start_animation).
    fn on_animation<F>(self, f: F) -> HookAnimation<F, Self>
    where
        F: FnMut(&mut Self, &ElemCtx, f64) -> bool,
    {
        HookAnimation::new(f, self)
    }

    /// Hooks a function into the element's ready logic.
    fn on_ready<F>(self, f: F) -> HookReady<F, Self>
    where
        F: FnMut(&mut Self, &ElemCtx),
    {
        HookReady::new(f, self)
    }

    /// Makes sure that the element has a default size.
    fn with_default_size(self, width: Length, height: Length) -> WithDefaultSize<Self> {
        WithDefaultSize::new(self)
            .with_default_width(width)
            .with_default_height(height)
    }

    /// Makes sure that the element has a default height.
    fn with_default_height(self, height: Length) -> WithDefaultSize<Self> {
        WithDefaultSize::new(self).with_default_height(height)
    }

    /// Makes sure that the element has a default width.
    fn with_default_width(self, width: Length) -> WithDefaultSize<Self> {
        WithDefaultSize::new(self).with_default_width(width)
    }

    /// Make the element scrollable horizontally.
    fn with_scroll_x(self) -> WithScroll<Self> {
        WithScroll::new(self).with_scroll_x()
    }

    /// Make the element scrollable vertically.
    fn with_scroll_y(self) -> WithScroll<Self> {
        WithScroll::new(self).with_scroll_y()
    }

    /// Adds a margin around the element.
    fn with_margin(self, margin: Length) -> WithMargin<Self> {
        WithMargin::new(self).with_margin(margin)
    }

    /// Adds a margin around the element.
    fn with_margin_top(self, top: Length) -> WithMargin<Self> {
        WithMargin::new(self).with_margin_top(top)
    }

    /// Adds a margin around the element.
    fn with_margin_right(self, right: Length) -> WithMargin<Self> {
        WithMargin::new(self).with_margin_right(right)
    }

    /// Adds a margin around the element.
    fn with_margin_bottom(self, bottom: Length) -> WithMargin<Self> {
        WithMargin::new(self).with_margin_bottom(bottom)
    }

    /// Adds a margin around the element.
    fn with_margin_left(self, left: Length) -> WithMargin<Self> {
        WithMargin::new(self).with_margin_left(left)
    }

    /// Adds a background shape to the element.
    fn with_background(self, brush: impl Into<Brush>) -> WithBackground<shapes::RoundedRect, Self> {
        ShapeElement::new()
            .with_child(self)
            .with_fill_shape()
            .with_brush(brush)
    }

    /// Sets the cursor that should be used when the element is hovered.
    fn with_cursor(self, cursor: CursorIcon) -> WithCursor<Self> {
        WithCursor::new(self).with_cursor(cursor)
    }

    /// Wraps the element in a clip shape.
    fn with_clip_rect(self) -> ClipChild<shapes::RoundedRect, Self> {
        ShapeElement::new().with_clip_shape().with_child(self)
    }

    /// Turns the element into a [`Box<dyn Element>`].
    fn into_dyn_element(self) -> Box<dyn Element>
    where
        Self: 'static,
    {
        Box::new(self)
    }

    /// Turns this [`Element`] into a [`linear_layout::Child`] with the provided grow factor.
    fn with_grow(self, grow: f64) -> linear_layout::Child<Self> {
        linear_layout::Child::new(self).with_grow(grow)
    }

    /// Creates a reference-counted [`RefCell`] containing the element.
    fn into_ref(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    /// Associates some data with the element.
    fn with_data<T>(self, data: T) -> WithData<T, Self> {
        WithData { data, child: self }
    }

    /// Translates the element by the provided vector.
    fn with_translation(self, translation: impl Into<Vec2>) -> Translate<Self> {
        Translate {
            translation: translation.into(),
            child: self,
        }
    }
}

impl<E: Element> ElementExt for E {}
