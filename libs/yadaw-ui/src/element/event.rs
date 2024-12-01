use std::any::TypeId;

/// Information about an event that occurred on an element.
pub trait Event: 'static {
    /// Returns the [`TypeId`] of the event.
    fn type_id(&self) -> TypeId;
}

impl dyn Event {
    /// Returns whether the event is of type `T`.
    #[inline]
    pub fn is<T: Event>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    /// Downcasts the event to type `T` without checking whether the event is of the correct type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check whether the event is of the correct type.
    /// The caller must ensure that the event is of the correct type.
    #[inline]
    pub unsafe fn downcast_unchecked<T: Event>(&self) -> &T {
        unsafe { &*(self as *const dyn Event as *const T) }
    }

    /// Downcasts the event to the type `T` if the event is of that type.
    #[inline]
    pub fn downcast<T: Event>(&self) -> Option<&T> {
        if self.is::<T>() {
            Some(unsafe { self.downcast_unchecked() })
        } else {
            None
        }
    }
}

impl<T: 'static> Event for T {
    #[inline]
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

/// The result of handling an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventResult {
    /// Indicate sthat the event should stop propagating down the element tree.
    StopPropagation,
    /// The event was not handled, or at least, propagation should continue.
    Continue,
}

impl EventResult {
    /// Returns whether the [`EventResult`] indicates that the event should stop
    /// propagating further down the element tree.
    #[inline]
    pub const fn should_stop_propagation(self) -> bool {
        matches!(self, Self::StopPropagation)
    }

    /// Returns whether the [`EventResult`] should continue propagating.
    #[inline]
    pub const fn should_continue(self) -> bool {
        matches!(self, Self::Continue)
    }
}
