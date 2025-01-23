use {
    crate::{
        ChannelLayouts, Device, DeviceFormats, Error, Format, ShareMode, Stream, StreamCallback,
        StreamConfig,
        backends::wasapi::{
            host_config::WasapiHostConfig,
            stream::WasapiStream,
            utility::{
                break_waveformat, device_error, duration_to_frames, guard, make_waveformatex,
                make_waveformatextensible, share_mode_to_wasapi,
            },
        },
    },
    std::{
        cell::{RefCell, RefMut},
        ops::Deref,
        ptr::NonNull,
        rc::Rc,
    },
    windows::{
        Win32::{
            Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
            Foundation::{PROPERTYKEY, S_FALSE, S_OK},
            Media::Audio::{
                AUDCLNT_E_UNSUPPORTED_FORMAT, AUDCLNT_SHAREMODE, AUDCLNT_SHAREMODE_EXCLUSIVE,
                AUDCLNT_SHAREMODE_SHARED, EDataFlow, IAudioClient, IAudioClient2, IMMDevice,
                IMMEndpoint, WAVEFORMATEXTENSIBLE, eCapture, eRender,
            },
            System::Com::{
                CLSCTX_ALL, CoTaskMemFree, STGM_READ, StructuredStorage::PropVariantToStringAlloc,
            },
        },
        core::Interface,
    },
};

/// An allocated `WAVEFORMATEXTENSIBLE` object.
///
/// When dropped, this type takes care of releasing the memory allocated for the
/// object.
struct WaveformatObject(NonNull<WAVEFORMATEXTENSIBLE>);

impl WaveformatObject {
    /// Creates a new `WaveformatObject` from the provided `WAVEFORMATEXTENSIBLE` object.
    ///
    /// # Safety
    ///
    /// The caller must make sure that it is safe to free the allocated object when the
    /// `WaveformatObject` is dropped.
    #[inline]
    pub unsafe fn new(waveformat: *mut WAVEFORMATEXTENSIBLE) -> Self {
        unsafe { Self(NonNull::new_unchecked(waveformat)) }
    }
}

impl Deref for WaveformatObject {
    type Target = WAVEFORMATEXTENSIBLE;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl Drop for WaveformatObject {
    #[inline]
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0.as_ptr() as *mut _)) };
    }
}

/// A device object for the WASAPI backend.
pub struct WasapiDevice {
    /// The inne WASAPI device object.
    inner: IMMDevice,

    /// An audio client that has been opened for the device.
    ///
    /// This is lazily initialized when needed and taken out of the `RefCell` when a stream is
    /// created for it.
    audio_client: RefCell<Option<IAudioClient>>,

    /// The WASAPI host configuration passed to the device.
    config: Rc<WasapiHostConfig>,
}

impl WasapiDevice {
    /// Creates an [`WasapiDevice`] from the provided WASAPI device object.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided object is still valid and can be used on the
    /// current thread.
    pub unsafe fn from_wasapi_device(config: Rc<WasapiHostConfig>, dev: IMMDevice) -> Self {
        Self {
            inner: dev,
            audio_client: RefCell::new(None),
            config,
        }
    }

    /// Returns the data-flow associated with this device.
    fn data_flow(&self) -> Result<EDataFlow, Error> {
        unsafe {
            self.inner
                .cast::<IMMEndpoint>()
                .map_err(|err| device_error("Failed to cast the device to an endpoint", err))?
                .GetDataFlow()
                .map_err(|err| device_error("Failed to get the data flow of the device", err))
        }
    }

    /// Attempts to read a device property as a string.
    fn get_property_as_string(&self, key: &PROPERTYKEY) -> Result<Option<String>, Error> {
        unsafe {
            // Access the property store for the given key.
            let property_store = self
                .inner
                .OpenPropertyStore(STGM_READ)
                .map_err(|err| device_error("Failed to open the device property store", err))?;
            let val = property_store
                .GetValue(key)
                .map_err(|err| device_error("Failed to get the device property value", err))?;

            if val.is_empty() {
                return Ok(None);
            }

            let utf16_ptr = PropVariantToStringAlloc(&val).map_err(|err| {
                device_error("Extracted device property value is not a string", err)
            })?;
            let _guard = guard(|| CoTaskMemFree(Some(utf16_ptr.as_ptr() as *mut _)));

            // Convert the UTF-16 string to a Rust string.
            let result = String::from_utf16_lossy(utf16_ptr.as_wide());
            Ok(Some(result))
        }
    }

    /// Creates a new [`IAudioClient`] for the device.
    fn create_new_audio_client(&self) -> Result<IAudioClient, Error> {
        unsafe {
            self.inner.Activate(CLSCTX_ALL, None).map_err(|err| {
                device_error("Failed to activate the audio client for the device", err)
            })
        }
    }

    /// Gets the audio client associated with the device.
    ///
    /// If the audio client has not been opened yet, it will be opened.
    ///
    /// # Panics
    ///
    /// This function panics if the audio client's cell is already borrowed in any way.
    fn get_audio_client(&self) -> Result<RefMut<IAudioClient>, Error> {
        let lock = match RefMut::filter_map(self.audio_client.borrow_mut(), |cli| cli.as_mut()) {
            Ok(cli) => return Ok(cli),
            Err(lock) => lock,
        };

        // The audio client has not been opened yet, so we need to open it.

        let audio_client = self.create_new_audio_client()?;
        Ok(RefMut::map(lock, move |cell| cell.insert(audio_client)))
    }

    /// Takes the audio client associated with the device.
    fn take_audio_client(&self) -> Result<IAudioClient, Error> {
        if let Some(audio_client) = self.audio_client.take() {
            Ok(audio_client)
        } else {
            self.create_new_audio_client()
        }
    }

    /// Gets the mix format of the device when used in shared mode.
    fn get_shared_mix_format(&self) -> Result<WaveformatObject, Error> {
        unsafe {
            self.get_audio_client()?
                .GetMixFormat()
                .map_err(|err| device_error("Failed to get the device mix format", err))
                .map(|ptr| WaveformatObject::new(ptr as _))
        }
    }

    /// Checks whether the provided format is supported by the device.
    fn is_format_supported(
        &self,
        share_mode: AUDCLNT_SHAREMODE,
        format: &WAVEFORMATEXTENSIBLE,
    ) -> Result<(bool, Option<WaveformatObject>), Error> {
        unsafe {
            let mut closest_match: *mut WAVEFORMATEXTENSIBLE = std::ptr::null_mut();
            let result = self.get_audio_client()?.IsFormatSupported(
                share_mode,
                format as *const _ as _,
                Some(&mut closest_match as *mut _ as _),
            );

            let closest_match = if closest_match.is_null() {
                None
            } else {
                Some(WaveformatObject::new(closest_match))
            };

            match result {
                S_OK => Ok((true, closest_match)),
                S_FALSE | AUDCLNT_E_UNSUPPORTED_FORMAT => Ok((false, closest_match)),
                err => Err(device_error(
                    "Failed to check if the format is supported",
                    err.into(),
                )),
            }
        }
    }

    /// Attempts a bunch of formats to determine what is supported by the underlying device.
    fn query_supported_formats(
        &self,
        share_mode: AUDCLNT_SHAREMODE,
    ) -> Result<DeviceFormats, Error> {
        let mut formats = DeviceFormats::DUMMY;
        let mut waveformat = WAVEFORMATEXTENSIBLE::default();

        formats.channel_layouts.insert(ChannelLayouts::INTERLEAVED);
        formats.max_buffer_size = u32::MAX;

        /// Pushes an item to a vector if it is not already present.
        fn push_unique<T: PartialEq>(v: &mut Vec<T>, item: T) {
            if !v.contains(&item) {
                v.push(item);
            }
        }

        /// Registers a set of values to the device formats
        /// if they are not already present.
        fn insert_values(
            formats: &mut DeviceFormats,
            channel_count: u32,
            format: Format,
            frame_rate: f64,
        ) {
            formats.max_channel_count = formats.max_channel_count.max(channel_count as u16);
            formats.formats.insert(format.into());
            push_unique(&mut formats.frame_rates, frame_rate);
        }

        /// Checks if the provided format is supported by the device.
        ///
        /// If it's supported, the format is inserted into the supported device formats structure.
        ///
        /// If it's not supported, the closest match is checked and inserted if possible.
        fn try_format(
            device: &WasapiDevice,
            share_mode: AUDCLNT_SHAREMODE,
            formats: &mut DeviceFormats,
            channel_count: u16,
            format: Format,
            frame_rate: u32,
            waveformat: &WAVEFORMATEXTENSIBLE,
        ) -> Result<bool, Error> {
            let (supported, closest_match) = device.is_format_supported(share_mode, waveformat)?;

            let validated_format;

            if supported {
                insert_values(formats, channel_count as u32, format, frame_rate as f64);
                validated_format = waveformat;
            } else if let Some(closest_match) = closest_match.as_ref() {
                if let Some((channel_count, format, frame_rate)) = break_waveformat(closest_match) {
                    insert_values(formats, channel_count as u32, format, frame_rate as f64);
                    validated_format = closest_match;
                } else {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            // Compute the buffer size of this format.

            if let Ok(audio_client2) = device.get_audio_client()?.cast::<IAudioClient2>() {
                let mut min_duration: i64 = 0;
                let mut max_duration: i64 = 0;

                unsafe {
                    if audio_client2
                        .GetBufferSizeLimits(
                            &validated_format.Format,
                            true,
                            &mut min_duration,
                            &mut max_duration,
                        )
                        .is_ok()
                    {
                        let min_frames = duration_to_frames(min_duration as u64, frame_rate);
                        let max_frames = duration_to_frames(max_duration as u64, frame_rate);
                        formats.min_buffer_size = formats.min_buffer_size.min(min_frames);
                        formats.max_buffer_size = formats.max_buffer_size.max(max_frames);
                    }
                }
            }

            Ok(true)
        }

        if share_mode == AUDCLNT_SHAREMODE_SHARED {
            let waveformat = self.get_shared_mix_format()?;
            if let Some((channel_count, format, frame_rate)) = break_waveformat(&waveformat) {
                insert_values(
                    &mut formats,
                    channel_count as u32,
                    format,
                    frame_rate as f64,
                );
            }
        }

        for &channel_count in self.config.tried_channel_counts.as_ref() {
            for &format in self.config.tried_formats.as_ref() {
                for &frame_rate in self.config.tried_frame_rates.as_ref() {
                    if !make_waveformatex(channel_count, format, frame_rate, &mut waveformat.Format)
                    {
                        continue;
                    }

                    if try_format(
                        self,
                        share_mode,
                        &mut formats,
                        channel_count,
                        format,
                        frame_rate,
                        &waveformat,
                    )? {
                        continue;
                    }

                    if share_mode != AUDCLNT_SHAREMODE_EXCLUSIVE {
                        continue;
                    }

                    // In exclusive mode, the device might only respond to the waveformatex or
                    // waveformatextensible structure but not the other.
                    // It's useful to check the same format configuration in case the other
                    // ones was not recognized.

                    if !make_waveformatextensible(
                        channel_count,
                        format,
                        frame_rate,
                        &mut waveformat,
                    ) {
                        continue;
                    }

                    try_format(
                        self,
                        share_mode,
                        &mut formats,
                        channel_count,
                        format,
                        frame_rate,
                        &waveformat,
                    )?;
                }
            }
        }

        Ok(formats)
    }
}

impl Device for WasapiDevice {
    #[inline]
    fn name(&self) -> Result<Option<String>, Error> {
        self.get_property_as_string(&PKEY_Device_FriendlyName)
    }

    fn output_formats(&self, share: ShareMode) -> Result<Option<DeviceFormats>, Error> {
        if self.data_flow()? == eRender {
            let share = share_mode_to_wasapi(share);
            Ok(Some(self.query_supported_formats(share)?))
        } else {
            Ok(None)
        }
    }

    fn input_formats(&self, share: ShareMode) -> Result<Option<DeviceFormats>, Error> {
        if self.data_flow()? == eCapture {
            let share = share_mode_to_wasapi(share);
            Ok(Some(self.query_supported_formats(share)?))
        } else {
            Ok(None)
        }
    }

    fn open_output_stream(
        &self,
        config: StreamConfig,
        callback: Box<dyn Send + FnMut(StreamCallback)>,
    ) -> Result<Box<dyn Stream>, Error> {
        let stream = WasapiStream::new(self.take_audio_client()?, config, callback)?;
        Ok(Box::new(stream))
    }

    fn open_input_stream(
        &self,
        _config: StreamConfig,
        _callback: Box<dyn Send + FnMut(StreamCallback)>,
    ) -> Result<Box<dyn Stream>, Error> {
        unimplemented!()
    }
}
