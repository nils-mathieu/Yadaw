use {
    crate::{BackendError, backends::wasapi::utility::backend_error},
    windows::Win32::{
        Foundation::{RPC_E_CHANGED_MODE, S_FALSE, S_OK},
        System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx},
    },
};

/// Ensures that the COM interface has been initialized.
///
/// # Panics
///
/// This function will panic if the COM interface cannot be initialized.
pub fn ensure_com_initialized() -> Result<(), BackendError> {
    unsafe {
        let ret = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        // Codes:
        // - S_OK: The COM interface was initialized successfully.
        // - S_FALSE: The COM interface was already initialized.
        // - RPC_E_CHANGED_MODE: The COM interface was initialized in a different mode, but we don't care as long as it's initialized.
        match ret {
            S_OK | S_FALSE | RPC_E_CHANGED_MODE => Ok(()),
            _ => Err(backend_error(
                "Failed to initialize the COM interface",
                ret.into(),
            )),
        }
    }
}
