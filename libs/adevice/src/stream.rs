use crate::Error;

/// Stores the actual data that the stream is rendering or capturing.
#[derive(Clone, Copy)]
pub union StreamData {
    /// This pointer is initialized when the `channel_encoding` field of the stream configuration
    /// is set to [`ChannelLayout::Interleaved`].
    ///
    /// It references exactly `frame_count * channel_count * sample_size` bytes of memory.
    pub interleaved: *mut u8,

    /// This pointer is initialized when the `channel_encoding` field of the stream configuration
    /// is set to [`ChannelLayout::Separate`].
    ///
    /// It references exactly `channel_count` pointers, each pointing to
    /// `frame_count * sample_size` bytes of memory.
    pub separate: *const *mut u8,
}

/// The information passed from the audio device to the user-defined callback function responsible
/// for rendering or capturing audio data.
pub struct StreamCallback {
    /// The buffer that the user should write audio data to or read audio data from.
    pub(crate) data: StreamData,
    /// The number of frames that the buffer references.
    ///
    /// A frame is a single sample for each channel.
    pub(crate) frame_count: usize,
}

impl StreamCallback {
    /// Returns the output buffer as an interleaved slice of samples.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// - The [`StreamCallback`] instance comes from an output stream.
    ///
    /// - The provided `channel_count` is the same as the channel count of the stream.
    ///
    /// - The provided type `T` is the same as the sample format of the stream.
    ///
    /// - The channel layout of the stream is [`Interleaved`](crate::ChannelLayout::Interleaved).
    #[inline]
    pub unsafe fn get_interleaved_output_buffer<T>(&mut self, channel_count: u32) -> &mut [T] {
        unsafe {
            let data: *mut T = self.data.interleaved.cast();
            std::slice::from_raw_parts_mut(data, self.frame_count * channel_count as usize)
        }
    }

    /// Returns the input buffer as an interleaved slice of samples.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// - The [`StreamCallback`] instance comes from an input stream.
    ///
    /// - The provided `channel_count` is the same as the channel count of the stream.
    ///
    /// - The provided type `T` is the same as the sample format of the stream.
    ///
    /// - The channel layout of the stream is [`Interleaved`](crate::ChannelLayout::Interleaved).
    #[inline]
    pub unsafe fn get_interleaved_input_buffer<T>(&self, channel_count: u32) -> &[T] {
        unsafe {
            let data: *const T = self.data.interleaved.cast();
            std::slice::from_raw_parts(data, self.frame_count * channel_count as usize)
        }
    }

    /// Returns the output buffer as a slice of pointers to separate channels.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// - The [`StreamCallback`] instance comes from an output stream.
    ///
    /// - The provided `channel_count` is the same as the channel count of the stream.
    ///
    /// - The provided type `T` is the same as the sample format of the stream.
    ///
    /// - The channel layout of the stream is [`Separate`](crate::ChannelLayout::Separate).
    ///
    /// # Remarks
    ///
    /// The pointers referenced by the returned slice are all of the length `frame_count`.
    #[inline]
    pub unsafe fn get_separate_output_buffer<T>(&self, channel_count: u32) -> &[*mut T] {
        unsafe {
            let data: *const *mut T = self.data.separate.cast();
            std::slice::from_raw_parts(data, channel_count as usize)
        }
    }

    /// Returns the input buffer as a slice of pointers to separate channels.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// - The [`StreamCallback`] instance comes from an input stream.
    ///
    /// - The provided `channel_count` is the same as the channel count of the stream.
    ///
    /// - The provided type `T` is the same as the sample format of the stream.
    ///
    /// - The channel layout of the stream is [`Separate`](crate::ChannelLayout::Separate).
    ///
    /// # Remarks
    ///
    /// The pointers referenced by the returned slice are all of the length `frame_count`.
    #[inline]
    pub unsafe fn get_separate_input_buffer<T>(&self, channel_count: u32) -> &[*const T] {
        unsafe {
            let data: *const *const T = self.data.separate.cast();
            std::slice::from_raw_parts(data, channel_count as usize)
        }
    }

    /// Returns the data associated with the stream callback.
    #[inline]
    pub fn data(&self) -> StreamData {
        self.data
    }

    /// Returns the number of frames that the buffer references.
    #[inline]
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }
}

/// Represents an open stream of audio data.
pub trait Stream {
    /// Starts the stream.
    ///
    /// This function is non-blocking and will return immediately, often before the stream is
    /// actually started.
    ///
    /// If the stream was already running, this function has no effect.
    fn start(&self) -> Result<(), Error>;

    /// Stops the stream.
    ///
    /// This function is non-blocking and will return immediately, often before the stream is
    /// actually stopped.
    ///
    /// If the stream was already paused, this function has no effect.
    fn stop(&self) -> Result<(), Error>;

    /// If the stream has encountered an error, this function returns the error. In that case, the
    /// high-priority thread driving the audio stream has already returned internally and the
    /// stream is likely unusable.
    fn check_error(&self) -> Result<(), Error>;
}
