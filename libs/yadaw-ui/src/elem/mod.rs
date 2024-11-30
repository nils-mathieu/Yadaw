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

mod with_size;
pub use self::with_size::WithSize;

mod with_cursor;
pub use self::with_cursor::WithCursor;

use crate::element::Element;

/// An extension trait for elements.
pub trait ElementExt: Sized + Element {
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
