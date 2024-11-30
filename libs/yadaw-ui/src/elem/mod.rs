//! Basic implementations of the [`Element`] trait.
//!
//! [`Element`]: crate::element::Element

pub mod shapes;
pub use self::shapes::{ShapeElement, WithBackground};

pub mod text;
pub use self::text::Text;

pub mod linear_layout;
pub use self::linear_layout::LinearLayout;

mod with_margin;
pub use self::with_margin::WithMargin;

mod length;
pub use self::length::Length;

mod lazy_linear_layout;
pub use self::lazy_linear_layout::LazyLinearLayout;

mod with_scroll;
pub use self::with_scroll::WithScroll;

mod events;
pub use self::events::{CatchEvent, HookEvents};

mod with_default_size;
pub use self::with_default_size::WithDefaultSize;

mod with_cursor;
pub use self::with_cursor::WithCursor;

use {
    crate::element::{ElemCtx, Element, Event, EventResult},
    vello::peniko::Brush,
    winit::window::CursorIcon,
};

/// An extension trait for elements.
pub trait ElementExt: Sized + Element {
    /// Hooks a fucntion into the element's event handling.
    fn hook_events<F>(self, f: F) -> HookEvents<F, Self>
    where
        F: FnMut(&mut Self, &ElemCtx, &dyn Event) -> EventResult,
    {
        HookEvents::new(f, self)
    }

    /// Hooks a function into the element's event handling, capturing a specific event type.
    fn catch_event<T, F>(self, f: F) -> CatchEvent<T, F, Self>
    where
        F: FnMut(&mut Self, &ElemCtx, &T) -> EventResult,
    {
        CatchEvent::new(f, self)
    }

    /// Makes sure that the element has a default size.
    fn with_default_size(self, width: Length, height: Length) -> WithDefaultSize<Self> {
        WithDefaultSize::new(self)
            .with_width(width)
            .with_height(height)
    }

    /// Makes sure that the element has a default height.
    fn with_default_height(self, height: Length) -> WithDefaultSize<Self> {
        WithDefaultSize::new(self).with_width(height)
    }

    /// Makes sure that the element has a default width.
    fn with_default_width(self, width: Length) -> WithDefaultSize<Self> {
        WithDefaultSize::new(self).with_width(width)
    }

    /// Make the element scrollable horizontally.
    fn with_scroll_x(self) -> WithScroll<Self> {
        WithScroll::new(self).with_scroll_x(true)
    }

    /// Make the element scrollable vertically.
    fn with_scroll_y(self) -> WithScroll<Self> {
        WithScroll::new(self).with_scroll_y(true)
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
    fn with_background(
        self,
        brush: impl Into<Brush>,
    ) -> WithBackground<shapes::RoundedRectangle, Self> {
        ShapeElement::default().with_child(self).with_brush(brush)
    }

    /// Sets the cursor that should be used when the element is hovered.
    fn with_cursor(self, cursor: CursorIcon) -> WithCursor<Self> {
        WithCursor::new(self).with_cursor(cursor)
    }

    /// Turns the element into a [`Box<dyn Element>`].
    #[inline]
    fn into_dyn_element(self) -> Box<dyn Element>
    where
        Self: 'static,
    {
        Box::new(self)
    }
}

impl<E: Element> ElementExt for E {}
