use {
    crate::audio_thread::{AudioBufferMut, AudioThread, AudioThreadControls},
    adevice::{StreamCallback, StreamConfig},
    std::sync::Arc,
};

/// Initializes the audio thread for the application.
pub fn initialize_audio_thread(controls: Arc<AudioThreadControls>) {
    let host = adevice::default_host()
        .unwrap_or_else(|err| panic!("Failed to initialize the audio host: {err}"))
        .unwrap_or_else(|| panic!("No audio backend available"));

    let output_device = host
        .default_output_device(adevice::RoleHint::Games)
        .unwrap_or_else(|err| panic!("Failed to get the default output device: {err}"))
        .unwrap_or_else(|| panic!("No default output device available"));

    let config = output_device
        .output_formats(adevice::ShareMode::Share)
        .unwrap_or_else(|err| panic!("Failed to get the available output formats: {err}"))
        .unwrap_or_else(|| panic!("No output formats available for the output device"))
        .to_stream_config(
            adevice::ShareMode::Share,
            2,
            &[],
            adevice::ChannelLayout::Planar,
            256,
            44100.0,
        );

    let handler = unsafe { make_stream_handler(controls, &config) };

    let stream = output_device
        .open_output_stream(config, handler)
        .unwrap_or_else(|err| panic!("Failed to build the output stream: {err}"));
    stream
        .start()
        .unwrap_or_else(|err| panic!("Failed to start the output stream: {err}"));

    // Leak the stream so that it is not closed when the function returns.
    std::mem::forget(stream);
}

/// Makes the output stream handler for the provided parameters.
///
/// # Safety
///
/// The caller must make sure that the returned handler is only used with a stream created with the
/// same configuration.
unsafe fn make_stream_handler(
    controls: Arc<AudioThreadControls>,
    config: &StreamConfig,
) -> Box<dyn Send + FnMut(StreamCallback)> {
    /// A trait for converting a sample from `f32` to the desired sample format.
    trait FromF32Sample {
        /// Performs the conversion from `f32` to the desired sample format.
        fn from_f32_sample(sample: f32) -> Self;
    }

    impl FromF32Sample for f32 {
        #[inline(always)]
        fn from_f32_sample(sample: f32) -> Self {
            sample
        }
    }

    impl FromF32Sample for i16 {
        #[inline(always)]
        fn from_f32_sample(sample: f32) -> Self {
            (sample * i16::MAX as f32) as i16
        }
    }

    /// Contains the state required to convert the `f32` planar data to the desired sample format
    /// and layout.
    struct StreamConverter {
        /// The buffer that holds the planar data.
        buffer: Vec<f32>,
        /// A buffer of pointers to the individual channels.
        pointers: Vec<*mut f32>,
    }

    // SAFETY: The raw pointers in `pointers` point into the structure itself. Mutation of those
    // values are done within the regular XOR pattern of Rust's ownership model.
    unsafe impl Send for StreamConverter {}
    unsafe impl Sync for StreamConverter {}

    impl StreamConverter {
        /// Creates a new stream converter.
        pub fn new(channel_count: usize) -> Self {
            Self {
                buffer: Vec::new(),
                pointers: vec![std::ptr::null_mut(); channel_count],
            }
        }

        /// Fills the internal buffer with the provided callback data.
        pub fn prepare_buffer(&mut self, frame_count: usize) -> AudioBufferMut {
            let channel_count = self.pointers.len();

            // FIXME: Remove this allocation? It's technically not real-time safe but it should
            // only occur once at the very beginning.
            self.buffer.resize(frame_count * channel_count, 0.0);
            for (i, ptr) in self.pointers.iter_mut().enumerate() {
                unsafe { *ptr = self.buffer.as_mut_ptr().add(i * frame_count) };
            }

            unsafe {
                AudioBufferMut::from_raw_parts(self.pointers.as_ptr(), frame_count, channel_count)
            }
        }

        /// Copies the internal buffer to the provided planar buffer.
        ///
        /// # Safety
        ///
        /// `dest` must have the same number of channels and frames as the internal buffer.
        pub unsafe fn copy_to_planar<T: FromF32Sample>(&self, mut dest: AudioBufferMut<T>) {
            debug_assert_eq!(dest.channel_count(), self.pointers.len());
            debug_assert_eq!(dest.frame_count(), self.buffer.len() / self.pointers.len());

            let channel_count = self.pointers.len();
            let frame_count = dest.frame_count();

            for channel in 0..channel_count {
                unsafe {
                    let src = self.buffer.as_ptr().add(channel * frame_count);
                    let dst = dest.channel_unchecked_mut(channel).as_mut_ptr();
                    for i in 0..frame_count {
                        *dst.add(i) = T::from_f32_sample(*src.add(i));
                    }
                }
            }
        }

        /// Copies the internal buffer to the provided interleaved buffer.
        ///
        /// # Safety
        ///
        /// `dest` must be large enough to hold `channels * frame_count` samples.
        pub unsafe fn copy_to_interleaved<T: FromF32Sample>(
            &self,
            dest: &mut [T],
            frame_count: usize,
        ) {
            debug_assert!(dest.len() == self.buffer.len());
            debug_assert!(frame_count == self.buffer.len() / self.pointers.len());

            let channel_count = self.pointers.len();

            for channel in 0..channel_count {
                unsafe {
                    let src = self.buffer.as_ptr().add(channel * frame_count);
                    let dst = dest.as_mut_ptr().add(channel);
                    for i in 0..frame_count {
                        *dst.add(i * channel_count) = T::from_f32_sample(*src.add(i));
                    }
                }
            }
        }
    }

    unsafe fn make_stream_handler_interleaved<T: FromF32Sample>(
        controls: Arc<AudioThreadControls>,
        config: &StreamConfig,
    ) -> Box<dyn Send + FnMut(StreamCallback)> {
        let mut converter = StreamConverter::new(config.channel_count as usize);
        let mut audio_thread = AudioThread::new(config.frame_rate, controls);
        let channel_count = config.channel_count as usize;
        Box::new(move |callback| unsafe {
            audio_thread.fill_buffer(converter.prepare_buffer(callback.frame_count()));
            converter.copy_to_interleaved(
                std::slice::from_raw_parts_mut(
                    callback.data().interleaved as *mut T,
                    callback.frame_count() * channel_count,
                ),
                callback.frame_count(),
            );
        })
    }

    unsafe fn make_stream_handler_planar<T: FromF32Sample>(
        controls: Arc<AudioThreadControls>,
        config: &StreamConfig,
    ) -> Box<dyn Send + FnMut(StreamCallback)> {
        let mut converter = StreamConverter::new(config.channel_count as usize);
        let mut audio_thread = AudioThread::new(config.frame_rate, controls);
        let channel_count = config.channel_count as usize;
        Box::new(move |callback| unsafe {
            audio_thread.fill_buffer(converter.prepare_buffer(callback.frame_count()));
            converter.copy_to_planar(AudioBufferMut::from_raw_parts(
                callback.data().planar as *const *mut T,
                callback.frame_count(),
                channel_count,
            ));
        })
    }

    unsafe fn make_stream_handler_planar_f32(
        controls: Arc<AudioThreadControls>,
        config: &StreamConfig,
    ) -> Box<dyn Send + FnMut(StreamCallback)> {
        let channel_count = config.channel_count;
        let mut audio_thread = AudioThread::new(config.frame_rate, controls);
        Box::new(move |callback| unsafe {
            audio_thread.fill_buffer(AudioBufferMut::from_raw_parts(
                callback.data().planar as *const *mut f32,
                callback.frame_count(),
                channel_count as usize,
            ));
        })
    }

    unsafe {
        use adevice::{ChannelLayout::*, Format::*};
        match (config.channel_layout, config.format) {
            (Interleaved, F32) => make_stream_handler_interleaved::<f32>(controls, config),
            (Interleaved, I16) => make_stream_handler_interleaved::<i16>(controls, config),
            (Planar, F32) => make_stream_handler_planar_f32(controls, config),
            (Planar, I16) => make_stream_handler_planar::<i16>(controls, config),
            (channel_layout, sample_format) => panic!(
                "Unsupported channel layout and format combination: {channel_layout:?}, {sample_format:?}"
            ),
        }
    }
}
