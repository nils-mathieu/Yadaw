use {
    crate::audio_thread::{
        AudioBufferMut, AudioBufferOwned, AudioBufferRef, AudioThreadEvent,
        one_shot_player::OneShot,
    },
    kui::{
        elem,
        elements::text::TextResource,
        event::EventResult,
        peniko::Color,
        winit::{dpi::PhysicalSize, window::WindowAttributes},
    },
    std::{path::PathBuf, sync::Arc},
    symphonia::core::{
        audio::Audio,
        codecs::audio::AudioDecoderOptions,
        formats::{FormatOptions, TrackType, probe::Hint},
        io::{MediaSource, MediaSourceStream},
        meta::MetadataOptions,
    },
};

mod audio_thread;

struct SinePluck {
    frequency: f64,
    amplitude: f64,
    phase: f64,
}

impl SinePluck {
    pub fn new(freq: f64) -> Self {
        Self {
            frequency: freq,
            amplitude: 1.0,
            phase: 0.0,
        }
    }
}

impl self::audio_thread::one_shot_player::OneShot for SinePluck {
    fn fill_buffer(&mut self, frame_rate: f64, mut buf: AudioBufferMut) -> bool {
        for frame_index in 0..buf.frame_count() {
            let val = self.amplitude * (self.phase * std::f64::consts::TAU).sin();
            self.phase += self.frequency / frame_rate;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
            self.amplitude -= self.amplitude * 0.0001;

            for channel in buf.channels_mut() {
                channel[frame_index] += val as f32;
            }
        }

        self.amplitude > 0.001
    }
}

/// The glorious entry point of the Yadaw application.
fn main() {
    let file = Arc::new(AudioFile::load("bins/yadaw/assets/sfx/welcome.mp3".into()).unwrap());

    kui::run(|ctx| {
        initialize_fonts(&ctx).unwrap_or_else(|err| panic!("Failed to register fonts: {err}"));

        let wnd = ctx.create_window(
            WindowAttributes::default()
                .with_title("Yadaw")
                .with_surface_size(PhysicalSize::new(1280, 720)),
        );

        let audio_thread_controls = Arc::new(self::audio_thread::AudioThreadControls::new(
            wnd.make_proxy(),
        ));
        self::audio_thread::initialize_audio_thread(audio_thread_controls.clone());

        wnd.set_root_element(elem! {
            kui::elements::flex {
                vertical;
                align_center;
                justify_center;
                gap: 8px;

                child: make_button("Play Pluck", {
                    let atc = audio_thread_controls.clone();
                    move || atc.one_shot.play(SinePluck::new(440.0))
                });
                child: make_button("Play welcome", {
                    let atc = audio_thread_controls.clone();
                    let file = file.clone();
                    move || atc.one_shot.play(PlayAudioFile::new(file.clone()))
                });
                child: make_button("Stop", {
                    let atc = audio_thread_controls.clone();
                    move || atc.one_shot.clear()
                });
                kui::elements::hook_events {
                    kui::elements::label {
                        text: "Running one shots: 0";
                        brush: "#fff";
                        inline: true;
                        font_stack: "Funnel Sans";
                    }
                    on_event: |elem, cx, ev| {
                        if let Some(AudioThreadEvent::OneShotCountChanged(val)) = ev.downcast_ref::<AudioThreadEvent>() {
                            elem.set_text(format!("Running one shots: {val}"));
                            cx.window.request_relayout();
                        }
                        EventResult::Continue
                    };
                }
            }
        });
    });
}

/// Creates a new button element.
fn make_button(text: impl Into<String>, mut on_click: impl FnMut()) -> impl kui::Element {
    elem! {
        kui::elements::button {
            child: kui::elements::interactive::make_appearance(
                elem! {
                    kui::elements::div {
                        radius: 8px;
                        brush: "#ff0000";
                        padding_left: 16px;
                        padding_right: 16px;
                        padding_top: 8px;
                        padding_bottom: 8px;

                        kui::elements::label {
                            text: text;
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
            );

            act_on_press: true;
            on_click: move |_, _| on_click();
        }
    }
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

/// An error that might occur when loading a file as an [`AudioFile`].
#[derive(Debug)]
enum AudioFileError {
    /// The file could not be opened in the first place.
    Io(std::io::Error),

    /// A loading error occured.
    Loading(symphonia::core::errors::Error),

    /// No audio track was found in the file.
    NoAudioTrack,
    /// A track was found, but it cannot be played because no codec could be found to decode it.
    CodecNotFound,
}

impl From<symphonia::core::errors::Error> for AudioFileError {
    #[inline]
    fn from(err: symphonia::core::errors::Error) -> Self {
        Self::Loading(err)
    }
}

impl From<std::io::Error> for AudioFileError {
    #[inline]
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl std::fmt::Display for AudioFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => std::fmt::Display::fmt(err, f),
            Self::Loading(err) => std::fmt::Debug::fmt(err, f),
            Self::NoAudioTrack => write!(f, "No audio track found in the file"),
            Self::CodecNotFound => write!(f, "No codec found to decode the audio track"),
        }
    }
}

impl std::error::Error for AudioFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Loading(err) => Some(err),
            Self::NoAudioTrack => None,
            Self::CodecNotFound => None,
        }
    }
}

/// Contains information about a potentially loading audio file.
struct AudioFile {
    /// The path to the file (if applicable).
    path: Option<PathBuf>,
    /// The samples of the audio file.
    data: AudioBufferOwned,
    /// The frame rate of the audio file.
    frame_rate: f64,
}

impl AudioFile {
    /// Creates a new audio file with
    pub fn load(file: PathBuf) -> Result<Self, AudioFileError> {
        Self::load_from_source(Box::new(std::fs::File::open(&file)?), Some(file))
    }

    /// Loads an [`AudioFile`] from an arbitrary media source.
    pub fn load_from_source(
        source: Box<dyn MediaSource>,
        path: Option<PathBuf>,
    ) -> Result<Self, AudioFileError> {
        //
        // Probe the input media source for the file format that we're dealing with.
        //

        let mut format = symphonia::default::get_probe().probe(
            Hint::new()
                .with_extension("wav")
                .with_extension("flac")
                .with_extension("ogg")
                .with_extension("mp3"),
            MediaSourceStream::new(source, Default::default()),
            FormatOptions::default(),
            MetadataOptions::default(),
        )?;

        // TODO: Remove this. It just want to be notified when a file has multiple tracks to decide
        // how to handle that case.
        debug_assert_eq!(format.tracks().len(), 1);

        let track = format
            .default_track(TrackType::Audio)
            .ok_or(AudioFileError::NoAudioTrack)?;

        let track_id = track.id;

        //
        // Create a decoder for the track.
        //

        let audio_codec_params = track
            .codec_params
            .as_ref()
            .ok_or(AudioFileError::CodecNotFound)?
            .audio()
            .ok_or(AudioFileError::CodecNotFound)?;

        // TODO: Determine in which case those informations are not available.
        let channel_count = audio_codec_params.channels.as_ref().unwrap().count();
        let frame_rate = audio_codec_params.sample_rate.unwrap() as f64;

        let mut decoder = symphonia::default::get_codecs()
            .make_audio_decoder(audio_codec_params, &AudioDecoderOptions::default())?;

        let mut data = AudioBufferOwned::new(channel_count);

        while let Some(packet) = format.next_packet()? {
            // If the packet does not belong to the audio track we're interested in, skip it.
            if packet.track_id() != track_id {
                continue;
            }

            // Decode the packet into audio samples.
            let buf = decoder.decode(&packet)?;

            use symphonia::core::audio::GenericAudioBufferRef;
            match buf {
                GenericAudioBufferRef::F32(buf) => extend_copy(buf, &mut data),
                GenericAudioBufferRef::F64(buf) => extend_convert(buf, &mut data),
                GenericAudioBufferRef::U8(buf) => extend_convert(buf, &mut data),
                GenericAudioBufferRef::U16(buf) => extend_convert(buf, &mut data),
                GenericAudioBufferRef::U24(buf) => extend_convert(buf, &mut data),
                GenericAudioBufferRef::U32(buf) => extend_convert(buf, &mut data),
                GenericAudioBufferRef::S8(buf) => extend_convert(buf, &mut data),
                GenericAudioBufferRef::S16(buf) => extend_convert(buf, &mut data),
                GenericAudioBufferRef::S24(buf) => extend_convert(buf, &mut data),
                GenericAudioBufferRef::S32(buf) => extend_convert(buf, &mut data),
            }
        }

        Ok(Self {
            frame_rate,
            path,
            data,
        })
    }

    /// Returns the data of the audio file.
    #[inline]
    pub fn data(&self) -> AudioBufferRef {
        self.data.as_audio_buffer_ref()
    }

    /// Returns the frame rate of the audio file.
    #[inline]
    pub fn frame_rate(&self) -> f64 {
        self.frame_rate
    }
}

impl std::fmt::Debug for AudioFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioFile")
            .field("path", &self.path)
            .field("frame_rate", &self.frame_rate)
            .finish_non_exhaustive()
    }
}

/// Extends the provided audio buffer with the data from the symphonia audio buffer.
fn extend_convert<T, U>(
    buf: &symphonia::core::audio::AudioBuffer<T>,
    dest: &mut AudioBufferOwned<U>,
) where
    T: symphonia::core::audio::sample::Sample + symphonia::core::audio::conv::IntoSample<U>,
{
    unsafe {
        assert_eq!(buf.spec().channels().count(), dest.channel_count());
        dest.extend_unchecked_by_sample(buf.frames(), |chan, i| {
            buf.plane(chan)
                .unwrap_unchecked()
                .get_unchecked(i)
                .into_sample()
        });
    }
}

/// Extends the provided audio buffer with the data from the symphonia audio buffer.
fn extend_copy<T>(buf: &symphonia::core::audio::AudioBuffer<T>, dest: &mut AudioBufferOwned<T>)
where
    T: symphonia::core::audio::sample::Sample,
{
    unsafe {
        assert_eq!(buf.spec().channels().count(), dest.channel_count());
        let amount = buf.frames();
        dest.extend_unchecked_by_channel(amount, |chan, dst| {
            let src = buf.plane(chan).unwrap_unchecked().as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, amount);
        });
    }
}

/// An audio file that is playing.
struct PlayAudioFile {
    /// The file to play.
    file: Arc<AudioFile>,
    /// The current frame index.
    next_index: usize,
}

impl PlayAudioFile {
    /// Creates a new [`PlayAudioFile`] instance.
    #[inline]
    pub fn new(file: Arc<AudioFile>) -> Self {
        Self {
            file,
            next_index: 0,
        }
    }
}

impl OneShot for PlayAudioFile {
    fn fill_buffer(&mut self, _frame_rate: f64, mut buf: AudioBufferMut) -> bool {
        assert_eq!(buf.channel_count(), self.file.data().channel_count());

        for (dst_channel, src) in buf.channels_mut().zip(self.file.data().channels()) {
            for (dst, sample) in dst_channel.iter_mut().zip(src.iter().skip(self.next_index)) {
                *dst += *sample;
            }
        }

        self.next_index += buf.frame_count();
        self.next_index < self.file.data().frame_count()
    }
}
