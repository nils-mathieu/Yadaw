use {
    crate::{BackendError, Error},
    coreaudio_sys::{
        CFRange, CFStringGetBytes, CFStringGetCStringPtr, CFStringGetLength, CFStringRef, OSStatus,
        kCFStringEncodingUTF8,
    },
    std::{borrow::Cow, ffi::CStr, mem::ManuallyDrop},
};

/// Converts the provided `OSStatus` to a crate-specific error.
pub fn backend_error(ctx: &str, err: OSStatus) -> BackendError {
    BackendError::new(format!("{}: 0x{:x}", ctx, err))
}

/// Converts the provided `OSStatus` to a crate-specific error.
pub fn device_error(ctx: &str, err: OSStatus) -> Error {
    backend_error(ctx, err).into()
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
