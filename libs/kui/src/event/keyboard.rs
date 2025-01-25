use {std::ops::Deref, winit::event::DeviceId};

/// An event that reports that the state of a keyboard key has changed.
#[derive(Clone, Debug)]
pub struct KeyEvent {
    /// Whether the event has been generated for consistency between platforms.
    pub synthetic: bool,

    /// The ID of the device that generated the event.
    pub device_id: Option<DeviceId>,

    /// The inner key event.
    pub inner: winit::event::KeyEvent,
}

impl Deref for KeyEvent {
    type Target = winit::event::KeyEvent;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
