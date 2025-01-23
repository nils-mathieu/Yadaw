use {crate::ShareMode, bitflags::bitflags, std::num::NonZero};

bitflags::bitflags! {
    /// A set of sample formats supported by an audio device.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct Formats: u32 {
        /// Little endian signed 8-bit integer format.
        const I8_LE = 1 << 0;
        /// Little endian unsigned 8-bit integer format.
        const U8_LE = 1 << 1;
        /// Little endian signed 16-bit integer format.
        const I16_LE = 1 << 2;
        /// Little endian unsigned 16-bit integer format.
        const U16_LE = 1 << 3;
        /// Little endian signed 24-bit integer format.
        const I24_LE = 1 << 4;
        /// Little endian unsigned 24-bit integer format.
        const U24_LE = 1 << 5;
        /// Little endian signed 32-bit integer format.
        const I32_LE = 1 << 6;
        /// Little endian unsigned 32-bit integer format.
        const U32_LE = 1 << 7;
        /// Little endian signed 64-bit integer format.
        const I64_LE = 1 << 8;
        /// Little endian unsigned 64-bit integer format.
        const U64_LE = 1 << 9;
        /// Little endian 32-bit floating point format.
        const F32_LE = 1 << 10;
        /// Little endian 64-bit floating point format.
        const F64_LE = 1 << 11;
        /// Big endian signed 8-bit integer format.
        const I8_BE = 1 << 12;
        /// Big endian unsigned 8-bit integer format.
        const U8_BE = 1 << 13;
        /// Big endian signed 16-bit integer format.
        const I16_BE = 1 << 14;
        /// Big endian unsigned 16-bit integer format.
        const U16_BE = 1 << 15;
        /// Big endian signed 24-bit integer format.
        const I24_BE = 1 << 16;
        /// Big endian unsigned 24-bit integer format.
        const U24_BE = 1 << 17;
        /// Big endian signed 32-bit integer format.
        const I32_BE = 1 << 18;
        /// Big endian unsigned 32-bit integer format.
        const U32_BE = 1 << 19;
        /// Big endian signed 64-bit integer format.
        const I64_BE = 1 << 20;
        /// Big endian unsigned 64-bit integer format.
        const U64_BE = 1 << 21;
        /// Big endian 32-bit floating point format.
        const F32_BE = 1 << 22;
        /// Big endian 64-bit floating point format.
        const F64_BE = 1 << 23;
    }
}

/// The format that an audio device should be initialized with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Little endian signed 8-bit integer format.
    I8Le,
    /// Little endian unsigned 8-bit integer format.
    U8Le,
    /// Little endian signed 16-bit integer format.
    I16Le,
    /// Little endian unsigned 16-bit integer format.
    U16Le,
    /// Little endian signed 24-bit integer format.
    I24Le,
    /// Little endian unsigned 24-bit integer format.
    U24Le,
    /// Little endian signed 32-bit integer format.
    I32Le,
    /// Little endian unsigned 32-bit integer format.
    U32Le,
    /// Little endian signed 64-bit integer format.
    I64Le,
    /// Little endian unsigned 64-bit integer format.
    U64Le,
    /// Little endian 32-bit floating point format.
    F32Le,
    /// Little endian 64-bit floating point format.
    F64Le,
    /// Big endian signed 8-bit integer format.
    I8Be,
    /// Big endian unsigned 8-bit integer format.
    U8Be,
    /// Big endian signed 16-bit integer format.
    I16Be,
    /// Big endian unsigned 16-bit integer format.
    U16Be,
    /// Big endian signed 24-bit integer format.
    I24Be,
    /// Big endian unsigned 24-bit integer format.
    U24Be,
    /// Big endian signed 32-bit integer format.
    I32Be,
    /// Big endian unsigned 32-bit integer format.
    U32Be,
    /// Big endian signed 64-bit integer format.
    I64Be,
    /// Big endian unsigned 64-bit integer format.
    U64Be,
    /// Big endian 32-bit floating point format.
    F32Be,
    /// Big endian 64-bit floating point format.
    F64Be,
}

impl Format {
    /// Returns the size, in bytes, of a single sample encoded in this format.
    #[rustfmt::skip]
    pub fn size_in_bytes(self) -> u32 {
        match self {
            Format::I8Le
            | Format::U8Le
            | Format::I8Be
            | Format::U8Be => 1,
            Format::I16Le
            | Format::U16Le
            | Format::I16Be
            | Format::U16Be => 2,
            Format::I24Le
            | Format::U24Le
            | Format::I24Be
            | Format::U24Be => 3,
            Format::I32Le
            | Format::U32Le
            | Format::I32Be
            | Format::U32Be
            | Format::F32Le
            | Format::F32Be => 4,
            Format::I64Le
            | Format::U64Le
            | Format::I64Be
            | Format::U64Be
            | Format::F64Le
            | Format::F64Be => 8,
        }
    }

    /// Returns whether the samples encoded in this format are little endian.
    pub fn is_little_endian(self) -> bool {
        match self {
            Format::I8Le
            | Format::U8Le
            | Format::I16Le
            | Format::U16Le
            | Format::I24Le
            | Format::U24Le
            | Format::I32Le
            | Format::U32Le
            | Format::I64Le
            | Format::U64Le
            | Format::F32Le
            | Format::F64Le => true,
            Format::I8Be
            | Format::U8Be
            | Format::I16Be
            | Format::U16Be
            | Format::I24Be
            | Format::U24Be
            | Format::I32Be
            | Format::U32Be
            | Format::I64Be
            | Format::U64Be
            | Format::F32Be
            | Format::F64Be => false,
        }
    }

    /// Returns whether the samples encoded in this format are big endian.
    #[inline]
    pub fn is_big_endian(self) -> bool {
        !self.is_little_endian()
    }

    /// Returns whether the samples encoded in this format are in the native endian of the
    /// compilation target.
    #[inline]
    pub fn is_native_endian(self) -> bool {
        if cfg!(target_endian = "little") {
            self.is_little_endian()
        } else {
            self.is_big_endian()
        }
    }

    /// Converts this [`Sample`] variant to little endian.
    ///
    /// If the sample is already in little endian, this will return `self`. Otherwise, this will
    /// return the equivalent sample in little endian.
    pub fn to_little_endian(self) -> Self {
        match self {
            Format::I8Le
            | Format::U8Le
            | Format::I16Le
            | Format::U16Le
            | Format::I24Le
            | Format::U24Le
            | Format::I32Le
            | Format::U32Le
            | Format::I64Le
            | Format::U64Le
            | Format::F32Le
            | Format::F64Le => self,
            Format::I8Be => Format::I8Le,
            Format::U8Be => Format::U8Le,
            Format::I16Be => Format::I16Le,
            Format::U16Be => Format::U16Le,
            Format::I24Be => Format::I24Le,
            Format::U24Be => Format::U24Le,
            Format::I32Be => Format::I32Le,
            Format::U32Be => Format::U32Le,
            Format::I64Be => Format::I64Le,
            Format::U64Be => Format::U64Le,
            Format::F32Be => Format::F32Le,
            Format::F64Be => Format::F64Le,
        }
    }

    /// Converts this [`Sample`] variant to big endian.
    ///
    /// If the sample is already in big endian, this will return `self`. Otherwise, this will
    /// return the equivalent sample in big endian.
    pub fn to_big_endian(self) -> Self {
        match self {
            Format::I8Be
            | Format::U8Be
            | Format::I16Be
            | Format::U16Be
            | Format::I24Be
            | Format::U24Be
            | Format::I32Be
            | Format::U32Be
            | Format::I64Be
            | Format::U64Be
            | Format::F32Be
            | Format::F64Be => self,
            Format::I8Le => Format::I8Be,
            Format::U8Le => Format::U8Be,
            Format::I16Le => Format::I16Be,
            Format::U16Le => Format::U16Be,
            Format::I24Le => Format::I24Be,
            Format::U24Le => Format::U24Be,
            Format::I32Le => Format::I32Be,
            Format::U32Le => Format::U32Be,
            Format::I64Le => Format::I64Be,
            Format::U64Le => Format::U64Be,
            Format::F32Le => Format::F32Be,
            Format::F64Le => Format::F64Be,
        }
    }

    /// Whether the format is signed.
    ///
    /// Note that this function returns `true` for floating point formats.
    pub fn is_signed(self) -> bool {
        match self {
            Format::I8Le
            | Format::I16Le
            | Format::I24Le
            | Format::I32Le
            | Format::I64Le
            | Format::F32Le
            | Format::F64Le
            | Format::I8Be
            | Format::I16Be
            | Format::I24Be
            | Format::I32Be
            | Format::I64Be
            | Format::F32Be
            | Format::F64Be => true,
            Format::U8Le
            | Format::U16Le
            | Format::U24Le
            | Format::U32Le
            | Format::U64Le
            | Format::U8Be
            | Format::U16Be
            | Format::U24Be
            | Format::U32Be
            | Format::U64Be => false,
        }
    }

    /// Returns whether the format is an integer format.
    pub fn is_integer(self) -> bool {
        match self {
            Format::I8Le
            | Format::U8Le
            | Format::I16Le
            | Format::U16Le
            | Format::I24Le
            | Format::U24Le
            | Format::I32Le
            | Format::U32Le
            | Format::I64Le
            | Format::U64Le
            | Format::I8Be
            | Format::U8Be
            | Format::I16Be
            | Format::U16Be
            | Format::I24Be
            | Format::U24Be
            | Format::I32Be
            | Format::U32Be
            | Format::I64Be
            | Format::U64Be => true,
            Format::F32Le | Format::F64Le | Format::F32Be | Format::F64Be => false,
        }
    }

    /// Returns whether the format is a floating point format.
    #[inline]
    pub fn is_floating_point(self) -> bool {
        !self.is_integer()
    }

    /// Converts this [`Sample`] variant to the native endian of the compilation target.
    ///
    /// If the sample is already in the correct format, this will return `self`. Otherwise, this
    /// will return the equivalent sample in the native endian format.
    #[inline]
    pub fn to_native_endian(self) -> Self {
        if cfg!(target_endian = "little") {
            self.to_little_endian()
        } else {
            self.to_big_endian()
        }
    }
}

impl From<Format> for Formats {
    fn from(value: Format) -> Self {
        match value {
            Format::U8Le => Formats::U8_LE,
            Format::I8Le => Formats::I8_LE,
            Format::U16Le => Formats::U16_LE,
            Format::I16Le => Formats::I16_LE,
            Format::U24Le => Formats::U24_LE,
            Format::I24Le => Formats::I24_LE,
            Format::U32Le => Formats::U32_LE,
            Format::I32Le => Formats::I32_LE,
            Format::U64Le => Formats::U64_LE,
            Format::I64Le => Formats::I64_LE,
            Format::F32Le => Formats::F32_LE,
            Format::F64Le => Formats::F64_LE,
            Format::U8Be => Formats::U8_BE,
            Format::I8Be => Formats::I8_BE,
            Format::U16Be => Formats::U16_BE,
            Format::I16Be => Formats::I16_BE,
            Format::U24Be => Formats::U24_BE,
            Format::I24Be => Formats::I24_BE,
            Format::U32Be => Formats::U32_BE,
            Format::I32Be => Formats::I32_BE,
            Format::U64Be => Formats::U64_BE,
            Format::I64Be => Formats::I64_BE,
            Format::F32Be => Formats::F32_BE,
            Format::F64Be => Formats::F64_BE,
        }
    }
}

bitflags! {
    /// A set of channel layouts supported by an audio device.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct ChannelLayouts: u8 {
        /// See [`ChannelLayout::Interleaved`].
        const INTERLEAVED = 1 << 0;
        /// See [`ChannelLayout::Separate`].
        const SEPARATE = 1 << 1;
    }
}

impl From<ChannelLayout> for ChannelLayouts {
    fn from(value: ChannelLayout) -> Self {
        match value {
            ChannelLayout::Interleaved => ChannelLayouts::INTERLEAVED,
            ChannelLayout::Separate => ChannelLayouts::SEPARATE,
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
    Separate,
}

/// The formats that are supported by a device.
///
/// # Remarks
///
/// [`adevice`] will attempt to represent the available formats in a way that models the actual
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
        const FALLBACK_FORMATS: [Format; 24] = [Format::F32Le, Format::F32Be, Format::I16Le, Format::I16Be, Format::U16Le, Format::U16Be, Format::I24Le, Format::I24Be, Format::U24Le, Format::U24Be, Format::F64Le, Format::F64Be, Format::I32Le, Format::I32Be, Format::U32Le, Format::U32Be, Format::I64Le, Format::I64Be, Format::U64Le, Format::U64Be, Format::I8Le, Format::I8Be, Format::U8Le, Format::U8Be];
        #[rustfmt::skip]
        const FALLBACK_CHANNEL_LAYOUTS: [ChannelLayout; 2] = [ChannelLayout::Separate, ChannelLayout::Interleaved];

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

            channel_encoding: if self.channel_layouts.contains(preferred_layout.into()) {
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
    pub channel_encoding: ChannelLayout,
}
