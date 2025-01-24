use {crate::ShareMode, bitflags::bitflags, std::num::NonZero};

bitflags::bitflags! {
    /// A set of sample formats supported by an audio device.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct Formats: u32 {
        /// Signed 8-bit integer format.
        const I8 = 1 << 0;
        /// Unsigned 8-bit integer format.
        const U8 = 1 << 1;
        /// Signed 16-bit integer format.
        const I16 = 1 << 2;
        /// Unsigned 16-bit integer format.
        const U16 = 1 << 3;
        /// Signed 24-bit integer format.
        const I24 = 1 << 4;
        /// Unsigned 24-bit integer format.
        const U24 = 1 << 5;
        /// Signed 32-bit integer format.
        const I32 = 1 << 6;
        /// Unsigned 32-bit integer format.
        const U32 = 1 << 7;
        /// Signed 64-bit integer format.
        const I64 = 1 << 8;
        /// Unsigned 64-bit integer format.
        const U64 = 1 << 9;
        /// 32-bit floating point format.
        const F32 = 1 << 10;
        /// 64-bit floating point format.
        const F64 = 1 << 11;
    }
}

/// The format that an audio device should be initialized with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Signed 8-bit integer format.
    I8,
    /// Unsigned 8-bit integer format.
    U8,
    /// Signed 16-bit integer format.
    I16,
    /// Unsigned 16-bit integer format.
    U16,
    /// Signed 24-bit integer format.
    I24,
    /// Unsigned 24-bit integer format.
    U24,
    /// Signed 32-bit integer format.
    I32,
    /// Unsigned 32-bit integer format.
    U32,
    /// 32-bit floating point format.
    F32,
    /// 64-bit floating point format.
    F64,
}

impl Format {
    /// Returns the size, in bytes, of a single sample encoded in this format.
    #[rustfmt::skip]
    pub fn size_in_bytes(self) -> u32 {
        match self {
            Format::I8 | Format::U8 => 1,
            Format::I16 | Format::U16 => 2,
            Format::I24 | Format::U24 => 3,
            Format::I32 | Format::U32 | Format::F32 => 4,
            Format::F64 => 8,
        }
    }
}

impl From<Format> for Formats {
    fn from(value: Format) -> Self {
        match value {
            Format::I8 => Formats::I8,
            Format::U8 => Formats::U8,
            Format::I16 => Formats::I16,
            Format::U16 => Formats::U16,
            Format::I24 => Formats::I24,
            Format::U24 => Formats::U24,
            Format::I32 => Formats::I32,
            Format::U32 => Formats::U32,
            Format::F32 => Formats::F32,
            Format::F64 => Formats::F64,
        }
    }
}

bitflags! {
    /// A set of channel layouts supported by an audio device.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct ChannelLayouts: u8 {
        /// See [`ChannelLayout::Interleaved`].
        const INTERLEAVED = 1 << 0;
        /// See [`ChannelLayout::Planar`].
        const PLANAR = 1 << 1;
    }
}

impl From<ChannelLayout> for ChannelLayouts {
    fn from(value: ChannelLayout) -> Self {
        match value {
            ChannelLayout::Interleaved => ChannelLayouts::INTERLEAVED,
            ChannelLayout::Planar => ChannelLayouts::PLANAR,
        }
    }
}

/// The layout that individual channels of audio data are encoded in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelLayout {
    /// One sample for each channel is stored contiguously in memory.
    ///
    /// # Representation
    ///
    /// Example for a stereo stream
    ///
    /// ```
    /// [L0, R0, L1, R1, L2, R2, ...]
    /// ```
    Interleaved,

    /// The samples for each channel are stored in separate arrays.
    ///
    /// # Representation
    ///
    /// Example for a stereo stream
    ///
    /// ```
    /// [L0, L1, L2, ...]
    /// [R0, R1, R2, ...]
    /// ```
    Planar,
}

/// The formats that are supported by a device.
///
/// # Remarks
///
/// [`advice`] will attempt to represent the available formats in a way that models the actual
/// underlying device as closely as possible. However, because of the fundamental differences
/// between audio APIs, some information may be lost or become inaccurate.
///
/// It's possible (though unliekly, and it should be considered a bug), that a format reported as
/// supported here will not actually work when used to create a stream.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceFormats {
    /// The maximum number of channels avaiable for the device.
    pub max_channel_count: u16,
    /// A list of frame rates that are supported by the device.
    ///
    /// At least one frame rate is present in this set.
    pub frame_rates: Vec<f64>,
    /// The formats supported by the device.
    ///
    /// At least one format is present in this set.
    pub formats: Formats,
    /// The minimum buffer size that the device supports.
    ///
    /// This is a number of *frames* that the device can process during a single call to the
    /// stream's callback.
    pub min_buffer_size: u32,
    /// The maximum buffer size that the device supports.
    ///
    /// This is a number of *frames* that the device can process during a single call to the
    /// stream's callback.
    ///
    /// # Remarks
    ///
    /// This might be `u32::MAX` if the device does not have a maximum buffer size. Attempting to
    /// actually create a stream with a buffer size that large will likely result in an error
    /// anyway.
    pub max_buffer_size: u32,
    /// The layouts that the device supports for encoding individual channels of audio data.
    ///
    /// At least one format is present in this set.
    pub channel_layouts: ChannelLayouts,
}

impl DeviceFormats {
    /// A dummy [`DeviceFormats`] that represents a device that does not support anything.
    pub const DUMMY: Self = Self {
        max_channel_count: 0,
        frame_rates: vec![],
        formats: Formats::empty(),
        min_buffer_size: 0,
        max_buffer_size: 0,
        channel_layouts: ChannelLayouts::empty(),
    };

    /// Creates a [`StreamConfig`] from the provided preferred parameters.
    ///
    /// # Parameters
    ///
    /// - `preferred_channel_count`: The preferred number of channels that is preferred by the
    ///   user.
    ///
    /// - `preferred_formats`: A list of formats that are preferred by the user. The first supported
    ///   format will be used. If none of the provided formats are available, another format will
    ///   be selected.
    ///
    /// - `preferred_layout`: The preferred channel layout that is preferred by the user. If the
    ///   layout is not supported by the device, another supported layout will be selected.
    ///
    /// - `preferred_buffer_size`: The preferred buffer size that is preferred by the user. If the
    ///   buffer size is not supported by the device, the closest supported buffer size will be
    ///   used instead.
    ///
    /// - `preferred_frame_rate`: The preferred frame rate that is preferred by the user. If the
    ///   frame rate is not supported by the device, the closest supported frame rate will be used
    ///   instead.
    ///
    /// # Returns
    ///
    /// A [`StreamConfig`] that represents the configuration that should be used for the stream.
    pub fn to_stream_config(
        &self,
        share_mode: ShareMode,
        preferred_channel_count: u16,
        preferred_formats: &[Format],
        preferred_layout: ChannelLayout,
        preferred_buffer_size: u32,
        preferred_frame_rate: f64,
    ) -> StreamConfig {
        #[rustfmt::skip]
        const FALLBACK_FORMATS: [Format; 10] = [Format::F32, Format::I24, Format::U24, Format::I16, Format::U16, Format::F64, Format::I32, Format::U32, Format::I8, Format::U8];
        #[rustfmt::skip]
        const FALLBACK_CHANNEL_LAYOUTS: [ChannelLayout; 2] = [ChannelLayout::Planar, ChannelLayout::Interleaved];

        StreamConfig {
            share_mode,

            channel_count: preferred_channel_count.min(self.max_channel_count),

            format: *preferred_formats
                .iter()
                .chain(FALLBACK_FORMATS.iter())
                .find(|&&f| self.formats.contains(f.into()))
                .unwrap(),

            buffer_size: NonZero::new(
                preferred_buffer_size.clamp(self.min_buffer_size, self.max_buffer_size),
            ),

            channel_layout: if self.channel_layouts.contains(preferred_layout.into()) {
                preferred_layout
            } else {
                *FALLBACK_CHANNEL_LAYOUTS
                    .iter()
                    .find(|&&l| self.channel_layouts.contains(l.into()))
                    .unwrap()
            },

            frame_rate: *self
                .frame_rates
                .iter()
                .min_by(|&&a, &&b| {
                    f64::total_cmp(
                        &(a - preferred_frame_rate).abs(),
                        &(b - preferred_frame_rate).abs(),
                    )
                })
                .unwrap(),
        }
    }

    /// Returns whether the structure contains invalid fields (e.g. an empty set of formats).
    pub(crate) fn validate(&self) -> bool {
        if self.formats.is_empty() {
            return false;
        }

        if self.channel_layouts.is_empty() {
            return false;
        }

        if self.max_channel_count == 0 {
            return false;
        }

        if self.frame_rates.is_empty() {
            return false;
        }

        if self.max_buffer_size == 0 {
            return false;
        }

        true
    }
}

/// Represents the configuration of an audio stream.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Whether the stream should be used in shared or exclusive mode.
    ///
    /// Note that this is a hint. On platforms that do not support that, it is possible for the
    /// backend implementation to ignore this value.
    pub share_mode: ShareMode,
    /// The number of channels that should be used in the stream.
    ///
    /// This must equal the number of channels supported by the device.
    pub channel_count: u16,
    /// The format that should be used in the stream.
    pub format: Format,
    /// The frame rate that should be used in the created stream.
    ///
    /// The frame rate is the number of frames that should be processed each second by the stream.
    pub frame_rate: f64,
    /// The size of the buffer that should be used in the stream.
    ///
    /// # Default
    ///
    /// If the provided value is `None`, the backend will choose an unspecified default value. Note
    /// that most implementations will *not* choose buffer sizes suitable for real-time audio
    /// applications. High latency is to be expected when no buffer size is specified.
    ///
    /// # Hint
    ///
    /// This field is a *hint* for the backend. The backend may choose to ignore this value and
    /// choose a different buffer size based on the device's capabilities and constraints. Even,
    /// it's possible for the backend to change buffer sizes during the lifetime of the stream. For
    /// this reason, one should not rely on the buffer size being constant or equal to the
    /// requested value.
    pub buffer_size: Option<NonZero<u32>>,
    /// The layout used by the stream to encode individual channels of audio data.
    pub channel_layout: ChannelLayout,
}
