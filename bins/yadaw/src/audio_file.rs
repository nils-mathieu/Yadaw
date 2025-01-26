use {
    crate::audio_thread::{AudioBufferMut, AudioBufferOwned, AudioBufferRef, OneShot},
    std::{path::PathBuf, sync::Arc},
    symphonia::core::{
        audio::Audio,
        codecs::audio::AudioDecoderOptions,
        formats::{FormatOptions, TrackType, probe::Hint},
        io::{MediaSource, MediaSourceStream},
        meta::MetadataOptions,
    },
};

/// An error that might occur when loading a file as an [`AudioFile`].
#[derive(Debug)]
pub enum AudioFileError {
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
pub struct AudioFile {
    /// The samples of the audio file.
    data: AudioBufferOwned,
    /// The frame rate of the audio file.
    frame_rate: f64,
}

impl AudioFile {
    /// Creates a new audio file with
    pub fn load(file: PathBuf) -> Result<Self, AudioFileError> {
        Self::load_from_source(Box::new(std::fs::File::open(&file)?))
    }

    /// Loads an [`AudioFile`] from an arbitrary media source.
    pub fn load_from_source(source: Box<dyn MediaSource>) -> Result<Self, AudioFileError> {
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

        Ok(Self { frame_rate, data })
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

    /// Creates a new [`AudioFilePlayer`] instance that plays this audio file.
    pub fn player(self: &Arc<Self>, volume: f32) -> AudioFilePlayer {
        AudioFilePlayer::new(self.clone(), volume)
    }

    /// Plays the audio file.
    pub fn play(self: &Arc<Self>, volume: f32) {
        crate::audio_thread::one_shot_controls().play(self.player(volume));
    }
}

impl std::fmt::Debug for AudioFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioFile")
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
pub struct AudioFilePlayer {
    /// The file to play.
    file: Arc<AudioFile>,
    /// The current frame index.
    next_index: usize,
    /// The volume at which to play the file.
    volume: f32,
}

impl AudioFilePlayer {
    /// Creates a new [`PlayAudioFile`] instance.
    #[inline]
    pub fn new(file: Arc<AudioFile>, volume: f32) -> Self {
        Self {
            file,
            next_index: 0,
            volume,
        }
    }
}

impl OneShot for AudioFilePlayer {
    fn fill_buffer(&mut self, _frame_rate: f64, mut buf: AudioBufferMut) -> bool {
        assert_eq!(buf.channel_count(), self.file.data().channel_count());

        for (dst_channel, src) in buf.channels_mut().zip(self.file.data().channels()) {
            for (dst, sample) in dst_channel.iter_mut().zip(src.iter().skip(self.next_index)) {
                *dst += *sample * self.volume;
            }
        }

        self.next_index += buf.frame_count();
        self.next_index < self.file.data().frame_count()
    }
}
