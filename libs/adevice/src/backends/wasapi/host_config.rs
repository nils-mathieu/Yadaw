use {crate::Format, std::borrow::Cow};

/// The WASAPI-specific host configuration.
#[derive(Debug, Clone)]
pub struct WasapiHostConfig {
    /// The list of channel counts to try when trying to determine the formats available
    /// on a device.
    pub tried_channel_counts: Cow<'static, [u16]>,
    /// The list of formats to try when trying to determine the formats available on a device.
    ///
    /// The formats set in this list are automatically converted to the correct endianness. Passing
    /// both the little-endian and big-endian variants of a format is not necessary.
    pub tried_formats: Cow<'static, [Format]>,
    /// The list of sample rates to try when trying to determine the formats available on a device.
    pub tried_frame_rates: Cow<'static, [u32]>,
}

impl Default for WasapiHostConfig {
    fn default() -> Self {
        #[rustfmt::skip]
        const TRIED_CHANNEL_COUNTS: [u16; 6] = [1, 2, 4, 6, 8, 10];
        #[rustfmt::skip]
        const TRIED_FORMATS: [Format; 7] = [Format::F32Le, Format::F64Le, Format::U8Le, Format::I16Le, Format::I24Le, Format::I32Le, Format::I64Le];
        #[rustfmt::skip]
        const TRIED_FRAME_RATES: [u32; 7] = [8000, 16000, 44100, 48000, 88000, 96000, 192000];

        Self {
            tried_channel_counts: Cow::Borrowed(&TRIED_CHANNEL_COUNTS),
            tried_formats: Cow::Borrowed(&TRIED_FORMATS),
            tried_frame_rates: Cow::Borrowed(&TRIED_FRAME_RATES),
        }
    }
}
