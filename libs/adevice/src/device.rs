use crate::{DeviceFormats, Error, Stream, StreamCallback, StreamConfig};

/// Represents the mode in which the audio device is shared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareMode {
    /// The audio device is shared between multiple applications.
    Share,

    /// The audio device is used exclusively by the application, preventing other
    /// processes from using it.
    Exclusive,
}

/// Represents a device responsible for managing audio the input and output streams of an audio
/// device.
pub trait Device {
    /// Returns the name of the device, if one is available.
    fn name(&self) -> Result<Option<String>, Error>;

    /// Returns the configuration of the device, when used as an output device.
    ///
    /// If the device is not an output device, this function returns `None`.
    fn output_formats(&self, share: ShareMode) -> Result<Option<DeviceFormats>, Error>;

    /// Returns the configuration of the device, when sued as an input device.
    ///
    /// If the device is not an input device, this function returns `None`.
    fn input_formats(&self, share: ShareMode) -> Result<Option<DeviceFormats>, Error>;

    /// Opens an output stream with the specified configuration.
    ///
    /// Internally, the stream is driven by a high-priority thread that is responsible for
    /// rendering the audio data. The provided callback will be called whenever the stream
    /// needs more data to play.
    fn open_output_stream(
        &self,
        config: StreamConfig,
        callback: Box<dyn Send + FnMut(StreamCallback)>,
    ) -> Result<Box<dyn Stream>, Error>;

    /// Opens an input stream with the specified configuration.
    ///
    /// Internally, the stream is driven by a high-priority thread that is responsible for
    /// capturing the audio data. The provided callback will be called whenever the stream
    /// has captured more data.
    fn open_input_stream(
        &self,
        config: StreamConfig,
        callback: Box<dyn Send + FnMut(StreamCallback)>,
    ) -> Result<Box<dyn Stream>, Error>;
}
