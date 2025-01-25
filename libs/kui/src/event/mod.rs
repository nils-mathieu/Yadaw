use std::any::TypeId;

mod pointer;
pub use self::pointer::*;

mod keyboard;
pub use self::keyboard::*;

/// The result of an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventResult {
    /// The event should continue to be propagated.
    Continue,
    /// The event has been handled and should not be propagated further.
    Handled,
}

impl EventResult {
    /// Whether the event has been handled.
    #[inline]
    pub fn is_handled(self) -> bool {
        self == EventResult::Handled
    }
}

/// The event trait.
pub trait Event: 'static {
    /// The type ID of the event.
    fn type_id(&self) -> TypeId;
}

impl<T: 'static> Event for T {
    #[inline]
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl dyn Event {
    /// Returns whether the event is of type `T`.
    #[inline]
    pub fn is<T: Event>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    /// Downcasts the event into `T`.
    ///
    /// # Safety
    ///
    /// The caller is responsible for making sure that the event really is of type `T`.
    #[inline(always)]
    pub unsafe fn downcast_ref_unchecked<T: Event>(&self) -> &T {
        unsafe { &*(self as *const dyn Event as *const T) }
    }

    /// Downcasts the event into `T`.
    ///
    /// # Safety
    ///
    /// The caller is responsible for making sure that the event really is of type `T`.
    #[inline(always)]
    pub unsafe fn downcast_mut_unchecked<T: Event>(&mut self) -> &mut T {
        unsafe { &mut *(self as *mut dyn Event as *mut T) }
    }

    /// Downcasts the event into `T`.
    ///
    /// # Returns
    ///
    /// Returns `None` if the event is not of type `T`.
    #[inline]
    pub fn downcast_ref<T: Event>(&self) -> Option<&T> {
        if self.is::<T>() {
            Some(unsafe { self.downcast_ref_unchecked() })
        } else {
            None
        }
    }

    /// Downcasts the event into `T`.
    ///
    /// # Returns
    ///
    /// Returns `None` if the event is not of type `T`.
    #[inline]
    pub fn downcast_mut<T: Event>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            Some(unsafe { self.downcast_mut_unchecked() })
        } else {
            None
        }
    }
}
