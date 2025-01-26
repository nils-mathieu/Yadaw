use {
    serde::{Deserialize, Serialize, de::DeserializeOwned},
    serde_inline_default::serde_inline_default,
    std::{path::Path, sync::OnceLock},
};

/// Yadaw settings.
#[serde_inline_default]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Miscellaneous {
    /// Whether the startup sound should be played.
    #[serde_inline_default(true)]
    pub play_startup_sound: bool,
}

impl Default for Miscellaneous {
    fn default() -> Self {
        serde_default()
    }
}

/// Represents the settings for the Yadaw application.
///
/// An instance of this type is loaded from the disk in order to determine what
/// settings the user has chosen.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// The miscellaneous settings.
    #[serde(default, skip_serializing_if = "is_default")]
    pub miscellaneous: Miscellaneous,
}

impl Settings {
    /// An error that might occur when loading settings.
    pub fn load_from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Loads the settings from the provided path.
    ///
    /// If the provided path does not refer to an existing file, the default value is returned
    /// instead.
    pub fn load_from_path(path: &Path) -> Result<Self, SettingsError> {
        match std::fs::read_to_string(path) {
            Ok(s) => Self::load_from_toml(&s).map_err(SettingsError::Toml),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(SettingsError::Io(e)),
        }
    }

    /// Loads the settings from the default path.
    pub fn load() -> Result<Self, SettingsError> {
        Self::load_from_path("settings.toml".as_ref())
    }
}

/// An error that might occur when attempting to load the settings from a file.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("{0}")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
    #[error("{0}")]
    Toml(
        #[from]
        #[source]
        toml::de::Error,
    ),
}

/// Returns whether the provided value is equal to its default value.
fn is_default<T: PartialEq + Default>(t: &T) -> bool {
    t == &T::default()
}

/// Returns the default value of the provided type using serde's default field values.
fn serde_default<T: DeserializeOwned>() -> T {
    let fields = std::iter::empty::<((), ())>();
    let deserializer = serde::de::value::MapDeserializer::<_, serde::de::value::Error>::new(fields);
    T::deserialize(deserializer).unwrap()
}

/// The global settings instance.
static SETTINGS: OnceLock<Settings> = OnceLock::new();

/// Initializes the global settings.
pub fn initialize() {
    debug_assert!(SETTINGS.get().is_none());

    let s = match Settings::load() {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to load the settings file: {e}");
            Settings::default()
        }
    };

    let _ = SETTINGS.set(s);
}

/// Returns a reference to the global settings instance.
#[inline]
pub fn get() -> &'static Settings {
    SETTINGS
        .get()
        .expect("Attempted to use `SETTINGS` before it was initialized")
}
