//! Basic implementations of the [`Element`] trait.
//!
//! [`Element`]: crate::element::Element

pub mod shapes;

mod length;
pub use self::length::*;

mod with_size;
pub use self::with_size::*;
