use {
    crate::{BackendError, Error, Format},
    coreaudio_sys::{
        AudioStreamBasicDescription, CFRange, CFStringGetBytes, CFStringGetCStringPtr,
        CFStringGetLength, CFStringRef, OSStatus, kAudioFormatFlagIsFloat,
        kAudioFormatFlagIsPacked, kAudioFormatLinearPCM, kCFStringEncodingUTF8,
    },
    std::{borrow::Cow, ffi::CStr, mem::ManuallyDrop},
};

/// Converts the provided `OSStatus` to a crate-specific error.
pub fn backend_error(ctx: &str, err: OSStatus) -> BackendError {
    BackendError::new(format!("{}: 0x{:x}", ctx, err))
}

/// Converts the provided `OSStatus` to a crate-specific error.
pub fn device_error(ctx: &str, err: OSStatus) -> Error {
    match err {
        0xffffd58f => Error::UnsupportedConfiguration,
        _ => backend_error(ctx, err).into(),
    }
}

/// Extracts the content of a `CFStringRef` as a UTF-8 string.
///
/// # Safety
///
/// The caller must make sure that the provided string is valid.
pub unsafe fn extract_cfstring(s: CFStringRef) -> Cow<'static, str> {
    unsafe {
        // Attempt to read the string as UTF-8.
        let maybe_cstring = CFStringGetCStringPtr(s, kCFStringEncodingUTF8);

        if !maybe_cstring.is_null() {
            let s = CStr::from_ptr(maybe_cstring);
            return Cow::Borrowed(std::str::from_utf8_unchecked(s.to_bytes()));
        }

        // Query  the length of the string in UTF16 code points.
        let utf16_len = CFStringGetLength(s);

        // Query the length of the string in bytes.
        let mut buffer_size = 0;
        CFStringGetBytes(
            s,
            CFRange {
                location: 0,
                length: utf16_len,
            },
            kCFStringEncodingUTF8,
            0,
            0,
            std::ptr::null_mut(),
            0,
            &mut buffer_size,
        );

        let mut buf: Vec<u8> = Vec::with_capacity(buffer_size as usize);

        // Encode the string into the buffer.
        let mut bytes_encoded = 0;
        CFStringGetBytes(
            s,
            CFRange {
                location: 0,
                length: utf16_len,
            },
            kCFStringEncodingUTF8,
            0,
            0,
            buf.as_mut_ptr(),
            buf.len() as i64,
            &mut bytes_encoded,
        );

        buf.set_len(bytes_encoded as usize);
        Cow::Owned(String::from_utf8_unchecked(buf))
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

/// Extracts and parses the content of a [`AudioStreamBasicDescription`] into:
///
/// - The format of the stream.
///
/// - The sample rate of the stream.
///
/// - The number of channels of the stream.
pub fn extract_basic_desc(desc: &AudioStreamBasicDescription) -> Option<(Format, f64, u16)> {
    if desc.mFormatID != kAudioFormatLinearPCM {
        return None;
    }
    if desc.mFormatFlags & kAudioFormatFlagIsPacked == 0 {
        return None;
    }

    let float = desc.mFormatFlags & kAudioFormatFlagIsFloat != 0;
    let format = match (desc.mBitsPerChannel, float) {
        (8, false) => Format::U8,
        (16, false) => Format::I16,
        (24, false) => Format::I24,
        (32, false) => Format::I32,
        (32, true) => Format::F32,
        (64, true) => Format::F64,
        _ => return None,
    };

    let channels: u16 = desc.mChannelsPerFrame.try_into().ok()?;

    Some((format, desc.mSampleRate, channels))
}

/// Creates an audio stream basic description from the provided format, frame rate, and number of
/// channels.
pub fn make_basic_desc(
    format: Format,
    frame_rate: f64,
    channels: u16,
) -> AudioStreamBasicDescription {
    let flags = match format {
        Format::F32 | Format::F64 => kAudioFormatFlagIsFloat | kAudioFormatFlagIsPacked,
        _ => kAudioFormatFlagIsPacked,
    };

    const FRAMES_PER_PACKET: u32 = 1;

    let bytes_per_frame = format.size_in_bytes() * channels as u32;

    AudioStreamBasicDescription {
        mFormatID: kAudioFormatLinearPCM,
        mFormatFlags: flags,
        mSampleRate: frame_rate,
        mBitsPerChannel: format.size_in_bytes() * 8,
        mChannelsPerFrame: channels as u32,
        mBytesPerFrame: bytes_per_frame,
        mBytesPerPacket: bytes_per_frame * FRAMES_PER_PACKET,
        mFramesPerPacket: FRAMES_PER_PACKET,
        ..Default::default()
    }
}
