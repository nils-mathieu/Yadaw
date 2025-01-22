use {
    vello::kurbo::Point,
    winit::event::{ButtonSource, DeviceId, ElementState, PointerKind, PointerSource},
};

/// Indicates that the pointer has moved over the window.
#[derive(Clone, Debug)]
pub struct PointerMoved {
    /// The ID of the device that generated the event.
    pub device_id: Option<DeviceId>,
    /// The new position of the pointer.
    pub position: Point,
    /// Whether the pointer is the primary pointer.
    ///
    /// This is notably the case for mouses, and for the first touch point on touch screens.
    pub primary: bool,
    /// The source of the pointer event.
    ///
    /// This can be used to differentiate between different kinds of pointers, like touchpads,
    /// mouses, or touchscreens.
    pub source: PointerSource,
}

/// A pointer button has been pressed or released.
#[derive(Clone, Debug)]
pub struct PointerButton {
    /// The ID of the device that generated the event.
    pub device_id: Option<DeviceId>,
    /// The position of the pointer at the time of the event.
    pub position: Point,
    /// Whether the button was pressed or released.
    pub state: ElementState,
    /// Whether the pointer is the primary pointer.
    ///
    /// This is notably the case for mouses, and for the first touch point on touch screens.
    pub primary: bool,
    /// The button that was pressed or released.
    pub button: ButtonSource,
}

/// An event that indicates that the pointer has left or entered the window.
#[derive(Clone, Debug)]
pub struct PointerEnetered {
    /// The ID of the device that generated the event.
    pub device_id: Option<DeviceId>,
    /// The position of the pointer at the time of the event.
    pub position: Point,
    /// Whether the pointer is the primary pointer.
    ///
    /// This is notably the case for mouses, and for the first touch point on touch screens.
    pub primary: bool,
    /// The kind of the pointer.
    pub kind: PointerKind,
}

/// An event that indicates that the pointer has left or entered the window.
#[derive(Clone, Debug)]
pub struct PointerLeft {
    /// The ID of the device that generated the event.
    pub device_id: Option<DeviceId>,
    /// Whether the pointer is the primary pointer.
    ///
    /// This is notably the case for mouses, and for the first touch point on touch screens.
    pub primary: bool,
    /// The kind of the pointer.
    pub kind: PointerKind,
}
