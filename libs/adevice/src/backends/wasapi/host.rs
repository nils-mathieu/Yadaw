use {
    crate::{
        BackendError, Device, Host, RoleHint,
        backends::wasapi::{
            WasapiHostConfig,
            device::WasapiDevice,
            utility::{backend_error, role_hint_to_wasapi},
        },
    },
    std::rc::Rc,
    windows::Win32::{
        Media::Audio::{
            DEVICE_STATE_ACTIVE, EDataFlow, ERole, IMMDeviceEnumerator, MMDeviceEnumerator, eAll,
            eCapture, eRender,
        },
        System::Com::{CLSCTX_ALL, CoCreateInstance},
    },
};

/// The [`Host`] implementation for the WASAPI backend.
pub struct WasapiHost {
    /// The device enumerator.
    ///
    /// It's used to get the default devices and to list all the available devices.
    device_enumerator: IMMDeviceEnumerator,

    /// The configuration for the WASAPI host.
    config: Rc<WasapiHostConfig>,
}

impl WasapiHost {
    /// Creates a new [`WasapiHost`] instance.
    pub fn new(config: Rc<WasapiHostConfig>) -> Result<Self, BackendError> {
        unsafe {
            let device_enumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|err| backend_error("Failed to create the device enumerator", err))?;

            Ok(Self {
                config,
                device_enumerator,
            })
        }
    }

    /// Returns the default endpoint for the provided flow and role values.
    pub fn get_default_endpoint(
        &self,
        flow: EDataFlow,
        role: ERole,
    ) -> Result<Option<Box<dyn Device>>, BackendError> {
        unsafe {
            let device = self
                .device_enumerator
                .GetDefaultAudioEndpoint(flow, role)
                .map_err(|err| backend_error("Failed to get default device", err))?;
            Ok(Some(Box::new(WasapiDevice::from_wasapi_device(
                self.config.clone(),
                device,
            ))))
        }
    }
}

impl Host for WasapiHost {
    fn devices(&self) -> Result<Vec<Box<dyn Device>>, BackendError> {
        unsafe {
            let collection = self
                .device_enumerator
                .EnumAudioEndpoints(eAll, DEVICE_STATE_ACTIVE)
                .map_err(|err| backend_error("Failed to enumerate audio devices", err))?;

            let count = collection
                .GetCount()
                .map_err(|err| backend_error("Failed to get the number of audio devices", err))?;

            let mut devices: Vec<Box<dyn Device>> = Vec::with_capacity(count as usize);
            for i in 0..count {
                let device = collection
                    .Item(i)
                    .map_err(|err| backend_error("Failed to get audio device", err))?;
                devices.push(Box::new(WasapiDevice::from_wasapi_device(
                    self.config.clone(),
                    device,
                )));
            }

            Ok(devices)
        }
    }

    fn default_input_device(
        &self,
        role: RoleHint,
    ) -> Result<Option<Box<dyn Device>>, BackendError> {
        self.get_default_endpoint(eCapture, role_hint_to_wasapi(role))
    }

    fn default_output_device(
        &self,
        role: RoleHint,
    ) -> Result<Option<Box<dyn Device>>, BackendError> {
        self.get_default_endpoint(eRender, role_hint_to_wasapi(role))
    }
}
