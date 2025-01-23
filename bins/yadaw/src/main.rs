use {
    adevice::{StreamCallback, StreamConfig},
    kui::{
        elem,
        elements::text::TextResource,
        event::EventResult,
        peniko::Color,
        winit::{dpi::PhysicalSize, window::WindowAttributes},
    },
    std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

/// The glorious entry point of the Yadaw application.
fn main() {
    kui::run(|ctx| {
        let audio_thread_controls = Arc::new(AudioThreadControls {
            paused: AtomicBool::new(true),
            frequency: AtomicU64::new(440),
        });

        initialize_audio_thread(audio_thread_controls.clone());
        initialize_fonts(&ctx).unwrap_or_else(|err| panic!("Failed to register fonts: {err}"));

        let wnd = ctx.create_window(
            WindowAttributes::default()
                .with_title("Yadaw")
                .with_surface_size(PhysicalSize::new(1280, 720)),
        );

        wnd.set_root_element(elem! {
            kui::elements::hook_events {
                kui::elements::anchor {
                    align_center;

                    kui::elements::button {
                        child: make_button();
                        on_click: {
                            let audio_thread_controls = audio_thread_controls.clone();
                            move || {
                                audio_thread_controls.paused.fetch_xor(true, Ordering::Relaxed);
                            }
                        };
                    }
                }

                on_event: {
                    let audio_thread_controls = audio_thread_controls.clone();
                    move |_, _, ev| {
                        if let Some(ev) = ev.downcast_ref::<kui::event::PointerMoved>() {
                            audio_thread_controls.frequency.store(ev.position.x as u64, Ordering::Relaxed);
                        }
                        EventResult::Continue
                    }
                };
            }

        });
    });
}

fn make_button() -> impl kui::elements::interactive::InputAppearance {
    kui::elements::interactive::make_appearance(
        elem! {
            kui::elements::div {
                radius: 8px;
                brush: "#ff0000";
                padding_left: 16px;
                padding_right: 16px;
                padding_top: 8px;
                padding_bottom: 8px;

                kui::elements::label {
                    text: "Click me!";
                    inline: true;
                    brush: "#000";
                    font_stack: "Funnel Sans";
                }
            }
        },
        |elem, cx, state| {
            let div = &mut elem.style;

            if state.active() {
                div.brush = Some(Color::from_rgb8(200, 200, 200).into());
            } else if state.hover() {
                div.brush = Some(Color::from_rgb8(225, 225, 225).into());
            } else {
                div.brush = Some(Color::from_rgb8(255, 255, 255).into());
            }

            cx.window.request_redraw();
        },
    )
}

/// Initializes the fonts for the application.
fn initialize_fonts(ctx: &kui::Ctx) -> std::io::Result<()> {
    const SUPPORTED_EXTENSIONS: &[&[u8]] = &[b"ttf"];

    ctx.with_resource_or_default(|res: &mut TextResource| {
        for entry in std::fs::read_dir("bins/yadaw/assets/fonts")? {
            let entry = entry?;

            if !entry.file_type()?.is_file() {
                continue;
            }

            let path = entry.path();

            let ext = path.extension().unwrap_or_default().as_encoded_bytes();
            if !SUPPORTED_EXTENSIONS.contains(&ext) {
                continue;
            }

            res.register_font(std::fs::read(path)?);
        }
        Ok(())
    })
}

/// Initializes the audio thread for the application.
fn initialize_audio_thread(controls: Arc<AudioThreadControls>) {
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

/// A collection of buffers for the audio stream.
struct AudioBufferMut<'a, T = f32> {
    /// The actual audio data.
    ///
    /// This is a list of `channel_count` buffers, each containing `frame_count` frames.
    data: *const *mut T,
    /// The number of frames in each buffer.
    frame_count: usize,
    /// The number of channels.
    channel_count: usize,

    /// The lifetime of the audio buffer.
    _lifetime: std::marker::PhantomData<&'a ()>,
}

#[allow(dead_code, reason = "TODO: remove this when stuff is used")]
impl<T> AudioBufferMut<'_, T> {
    /// Creates a new [`AudioBuffer<T>`] from the provided raw parts.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `data` pointer references `channel_count`
    /// buffers, each referencing `frame_count` frames.
    ///
    /// The data must remain valid for the lifetime `'a`.
    #[inline]
    pub unsafe fn from_raw_parts(
        data: *const *mut T,
        frame_count: usize,
        channel_count: usize,
    ) -> Self {
        Self {
            data,
            frame_count,
            channel_count,
            _lifetime: std::marker::PhantomData,
        }
    }

    /// Returns the number of channels in the audio buffer.
    #[inline(always)]
    pub fn channel_count(&self) -> usize {
        self.channel_count
    }

    /// Returns the number of frames in each channel of the audio buffer.
    #[inline(always)]
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_unchecked(&self, channel: usize) -> &[T] {
        debug_assert!(channel < self.channel_count);
        unsafe { std::slice::from_raw_parts(*self.data.add(channel), self.frame_count) }
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_unchecked_mut(&mut self, channel: usize) -> &mut [T] {
        debug_assert!(channel < self.channel_count);
        unsafe { std::slice::from_raw_parts_mut(*self.data.add(channel), self.frame_count) }
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Returns
    ///
    /// Returns `None` if the provided `channel` index is out of bounds.
    #[inline]
    pub fn channel(&self, channel: usize) -> Option<&[T]> {
        if channel < self.channel_count {
            Some(unsafe { self.channel_unchecked(channel) })
        } else {
            None
        }
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Returns
    ///
    /// Returns `None` if the provided `channel` index is out of bounds.
    #[inline]
    pub fn channel_mut(&mut self, channel: usize) -> Option<&mut [T]> {
        if channel < self.channel_count {
            Some(unsafe { self.channel_unchecked_mut(channel) })
        } else {
            None
        }
    }

    /// Returns an iterator over the channels of the audio buffer.
    #[inline]
    pub fn channels(&self) -> impl Iterator<Item = &[T]> + '_ {
        (0..self.channel_count).map(move |channel| unsafe { self.channel_unchecked(channel) })
    }

    /// Returns an iterator over the channels of the audio buffer.
    #[inline]
    pub fn channels_mut(&mut self) -> impl Iterator<Item = &mut [T]> + '_ {
        (0..self.channel_count).map(move |channel| unsafe {
            // SAFETY: We can't just use `channel_unchecked_mut` because it borrows self
            // completely.
            std::slice::from_raw_parts_mut(*self.data.add(channel), self.frame_count)
        })
    }
}

/// The controls for the audio thread.
///
/// An instance of this structure is shared between the audio thread and the ui thread.
pub struct AudioThreadControls {
    /// Whether playback is paused or not.
    paused: AtomicBool,
    /// The frequency of the oscillator.
    frequency: AtomicU64,
}

/// The state of the audio thread.
pub struct AudioThread {
    /// The controls of the audio thread.
    controls: Arc<AudioThreadControls>,

    /// The number of frames the audio thread is processing per second.
    sample_rate: f64,

    /// The phase of the oscillator.
    phase: f64,
    /// The amplitude of the oscillator.
    amplitude: f64,
}

impl AudioThread {
    /// Creates a new audio thread.
    pub fn new(frame_rate: f64, controls: Arc<AudioThreadControls>) -> Self {
        Self {
            controls,
            sample_rate: frame_rate,
            phase: 0.0,
            amplitude: 0.0,
        }
    }

    /// The function responsible for filling the audio buffer with data.
    ///
    /// # Remarks
    ///
    /// This function is running on a separate high-priority thread and should *never* block for
    /// any reason.
    ///
    /// This means that any operation that involves the kernel (unless it's specifically a real-time
    /// safe operation) should be avoided at all cost. That includes memory allocations, I/O, etc.
    fn fill_buffer(&mut self, mut buf: AudioBufferMut) {
        let target_amplitude = if self.controls.paused.load(Ordering::Relaxed) {
            0.0
        } else {
            1.0
        };

        let freq = self.controls.frequency.load(Ordering::Relaxed) as f64;

        for frame in 0..buf.frame_count() {
            self.amplitude += (target_amplitude - self.amplitude) * 0.002;

            let sample = self.amplitude * (self.phase * std::f64::consts::TAU).sin();
            self.phase += freq / self.sample_rate;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }

            for channel in buf.channels_mut() {
                channel[frame] = sample as f32;
            }
        }
    }
}
