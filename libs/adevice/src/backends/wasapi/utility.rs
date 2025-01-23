use {
    crate::{BackendError, Error, Format, RoleHint, ShareMode},
    std::mem::ManuallyDrop,
    windows::Win32::Media::{
        Audio::{
            AUDCLNT_E_BUFFER_SIZE_ERROR, AUDCLNT_E_DEVICE_IN_USE, AUDCLNT_E_DEVICE_INVALIDATED,
            AUDCLNT_E_EXCLUSIVE_MODE_NOT_ALLOWED, AUDCLNT_E_EXCLUSIVE_MODE_ONLY,
            AUDCLNT_E_UNSUPPORTED_FORMAT, AUDCLNT_SHAREMODE, AUDCLNT_SHAREMODE_EXCLUSIVE,
            AUDCLNT_SHAREMODE_SHARED, ERole, WAVE_FORMAT_PCM, WAVEFORMATEX, WAVEFORMATEXTENSIBLE,
            eCommunications, eConsole, eMultimedia,
        },
        KernelStreaming::{KSDATAFORMAT_SUBTYPE_PCM, WAVE_FORMAT_EXTENSIBLE},
        Multimedia::{KSDATAFORMAT_SUBTYPE_IEEE_FLOAT, WAVE_FORMAT_IEEE_FLOAT},
    },
};

/// Turns the provided `HRESULT` into a [`BackendError`].
pub fn backend_error(context: &str, err: windows::core::Error) -> BackendError {
    let err_message = err.message();
    if err_message.is_empty() {
        BackendError::new(format!("WASAPI: {}: {}", context, err))
    } else {
        BackendError::new(format!("WASAPI: {}: {} ({})", context, err_message, err))
    }
}

/// Turns the provided `HRESULT` into a [`Error`].
///
/// This function will automatically catch errors indicating that the device is not longer
/// available and return an [`Error::DeviceNotAvailable`] instead.
#[rustfmt::skip]
pub fn device_error(context: &str, err: windows::core::Error) -> Error {
    match err.code() {
        AUDCLNT_E_DEVICE_INVALIDATED => Error::DeviceNotAvailable,
        AUDCLNT_E_DEVICE_IN_USE => Error::DeviceInUse,
        AUDCLNT_E_EXCLUSIVE_MODE_NOT_ALLOWED | AUDCLNT_E_EXCLUSIVE_MODE_ONLY => Error::ShareModeNotSupported,
        AUDCLNT_E_UNSUPPORTED_FORMAT | AUDCLNT_E_BUFFER_SIZE_ERROR => Error::UnsupportedConfiguration,
        _ => Error::Backend(backend_error(context, err)),
    }
}

/// Calls the provided closure and returns a guard that will call the closure when dropped.
pub fn guard(f: impl FnOnce()) -> impl Drop {
    struct Guard<F: FnOnce()>(ManuallyDrop<F>);
    impl<F: FnOnce()> Drop for Guard<F> {
        fn drop(&mut self) {
            unsafe { ManuallyDrop::take(&mut self.0)() }
        }
    }
    Guard(ManuallyDrop::new(f))
}

/// Converts a number of frames to a duration in 100-nanosecond units.
pub fn frames_to_duration(buffer_size: u32, frame_rate: u32) -> u64 {
    let buffer_size = buffer_size as u64;
    let frame_rate = frame_rate as u64;
    (buffer_size * 10_000_000) / frame_rate
}

/// Converts a duration in 100-nanosecond units to a number of frames.
pub fn duration_to_frames(duration: u64, frame_rate: u32) -> u32 {
    let frame_rate = frame_rate as u64;
    ((duration * frame_rate) / 10_000_000) as u32
}

/// The value of the `cbSize` field in a [`WAVEFORMATEXTENSIBLE`] object when the "Extensible"
/// part of the structure is used.
const EXPECTED_EXTENSIBLE_SIZE: u16 =
    (std::mem::size_of::<WAVEFORMATEXTENSIBLE>() - std::mem::size_of::<WAVEFORMATEX>()) as u16;

/// Updates the provided [`WAVEFORMATEXTENSIBLE`] object with the provided parameters.
///
/// # Returns
///
/// This function returns whether the operation was successful.
///
/// If `true` is returned, the `waveformat` object will be updated with the provided parameters.
///
/// If `false` is returned, the `waveformat` object will be left in an unspecified state and it
/// should not be used.
pub fn make_waveformatex(
    channel_count: u16,
    format: Format,
    frame_rate: u32,
    waveformat: &mut WAVEFORMATEX,
) -> bool {
    waveformat.wFormatTag = match format.to_little_endian() {
        Format::U8Le | Format::I16Le | Format::I32Le | Format::I64Le => WAVE_FORMAT_PCM as u16,
        Format::F32Le | Format::F64Le => WAVE_FORMAT_IEEE_FLOAT as u16,
        _ => return false,
    };

    let sample_size = format.size_in_bytes();

    waveformat.nChannels = channel_count;
    waveformat.nSamplesPerSec = frame_rate;
    waveformat.nAvgBytesPerSec = frame_rate * channel_count as u32 * sample_size;
    waveformat.nBlockAlign = channel_count * sample_size as u16;
    waveformat.wBitsPerSample = sample_size as u16 * 8;
    waveformat.cbSize = 0;

    true
}

/// Like [`make_waveformatex`], but fills the "extensible" part instead of the
/// `WAVEFORMATEX` part.
pub fn make_waveformatextensible(
    channel_count: u16,
    format: Format,
    frame_rate: u32,
    waveformat: &mut WAVEFORMATEXTENSIBLE,
) -> bool {
    waveformat.Format.wFormatTag = WAVE_FORMAT_EXTENSIBLE as u16;

    waveformat.SubFormat = match format.to_little_endian() {
        Format::U8Le | Format::I16Le | Format::I32Le | Format::I64Le => KSDATAFORMAT_SUBTYPE_PCM,
        Format::F32Le | Format::F64Le => KSDATAFORMAT_SUBTYPE_IEEE_FLOAT,
        _ => return false,
    };

    if format.is_integer() {
        WAVE_FORMAT_PCM as u16
    } else {
        return false;
    };

    let sample_size = format.size_in_bytes();

    waveformat.Format.nChannels = channel_count;
    waveformat.Format.nSamplesPerSec = frame_rate;
    waveformat.Format.nAvgBytesPerSec = frame_rate * channel_count as u32 * sample_size;
    waveformat.Format.nBlockAlign = channel_count * sample_size as u16;
    waveformat.Format.wBitsPerSample = sample_size as u16 * 8;
    waveformat.Format.cbSize = EXPECTED_EXTENSIBLE_SIZE;

    true
}

/// Breaks down the provided [`WAVEFORMATEXTENSIBLE`] object into its components.
///
/// # Returns
///
/// - The channel count.
///
/// - The sample format.
///
/// - The frame rate.
///
/// If the provided format is not supported or cannot be parsed, this function returns `None`.
///
/// # Remarks
///
/// The functions return the little-endian version of the sample format.
pub fn break_waveformat(waveformat: &WAVEFORMATEXTENSIBLE) -> Option<(u16, Format, u32)> {
    let format = match (
        waveformat.Format.wBitsPerSample,
        waveformat.Format.wFormatTag as u32,
    ) {
        (8, WAVE_FORMAT_PCM) => Format::U8Le,
        (16, WAVE_FORMAT_PCM) => Format::I16Le,
        (32, WAVE_FORMAT_PCM) => Format::I32Le,
        (64, WAVE_FORMAT_PCM) => Format::I64Le,
        (32, WAVE_FORMAT_IEEE_FLOAT) => Format::F32Le,
        (64, WAVE_FORMAT_IEEE_FLOAT) => Format::F64Le,
        (_, WAVE_FORMAT_EXTENSIBLE) if waveformat.Format.cbSize == EXPECTED_EXTENSIBLE_SIZE => {
            let subformat = waveformat.SubFormat;

            if subformat.to_u128() == KSDATAFORMAT_SUBTYPE_PCM.to_u128() {
                match waveformat.Format.wBitsPerSample {
                    8 => Format::U8Le,
                    16 => Format::I16Le,
                    32 => Format::I32Le,
                    64 => Format::I64Le,
                    _ => return None,
                }
            } else if subformat.to_u128() == KSDATAFORMAT_SUBTYPE_IEEE_FLOAT.to_u128() {
                match waveformat.Format.wBitsPerSample {
                    32 => Format::F32Le,
                    64 => Format::F64Le,
                    _ => return None,
                }
            } else {
                return None;
            }
        }
        _ => return None,
    };

    Some((
        waveformat.Format.nChannels,
        format,
        waveformat.Format.nSamplesPerSec,
    ))
}

/// Converts the provided [`RoleHint`] to a WASAPI [`ERole`].
pub fn role_hint_to_wasapi(role: RoleHint) -> ERole {
    match role {
        RoleHint::Communications => eCommunications,
        RoleHint::Multimedia => eMultimedia,
        RoleHint::Notifications => eConsole,
        RoleHint::Games => eConsole,
    }
}

/// Converts the provided [`ShareMode`] to a WASAPI [`AUDCLNT_SHAREMODE`].
pub fn share_mode_to_wasapi(share_mode: ShareMode) -> AUDCLNT_SHAREMODE {
    match share_mode {
        ShareMode::Share => AUDCLNT_SHAREMODE_SHARED,
        ShareMode::Exclusive => AUDCLNT_SHAREMODE_EXCLUSIVE,
    }
}
