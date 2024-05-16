#![allow(improper_ctypes_definitions)]

use std::path::PathBuf;

use abi_stable::{
    erased_types::{DeserializeDyn, DynTrait, SerializeProxyType},
    external_types::{RawValueBox, RawValueRef},
    library::{LibraryError, RootModule},
    package_version_strings,
    sabi_types::VersionStrings,
    std_types::{RBox, RBoxError, RErr, ROk, RResult, RStr},
    StableAbi,
};

use lazy_static::lazy_static;

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = BottlerocketSettingsPluginRef)))]
#[sabi(missing_field(panic))]
pub struct BottlerocketSettingsPlugin {
    pub default_settings: extern "C" fn() -> BottlerocketSettings,

    #[sabi(last_prefix_field)]
    pub deserialize_settings:
        for<'a> extern "C" fn(RStr<'a>) -> RResult<BottlerocketSettings, RBoxError>,
}

impl RootModule for BottlerocketSettingsPluginRef {
    const BASE_NAME: &'static str = "settings";
    const NAME: &'static str = "settings";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();

    abi_stable::declare_root_module_statics! {BottlerocketSettingsPluginRef}
}

// FIXME: we could prefer to load "lib{variant}.so" and fall back to "libsettings.so" if not found
// ideally we'd be able to build all the different settings plugins in the current main repo and
// have them not clobber each other
lazy_static! {
    static ref PLUGIN: Result<BottlerocketSettingsPluginRef, LibraryError> =
        BottlerocketSettingsPluginRef::load_from_file(&PathBuf::from(format!(
            "lib{}.so",
            BottlerocketSettingsPluginRef::NAME
        )));
}

// FIXME: error handling is somewhat fraught, we can't really recover from a load error, so
// perhaps a panic is correct
impl BottlerocketSettingsPluginRef {
    pub fn try_load() {
        let _ = *PLUGIN;
        if let Err(e) = PLUGIN.as_ref() {
            eprintln!("{e}");
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
#[allow(dead_code)]
pub struct BottlerocketSettingsPluginLoader;

#[allow(dead_code)]
impl BottlerocketSettingsPluginLoader {
    pub fn load() {
        BottlerocketSettingsPluginRef::try_load()
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(
    Sync,
    Send,
    Default,
    Eq,
    PartialEq,
    Clone,
    Debug,
    Deserialize,
    Serialize
))]
pub struct BottlerocketSettingsInterface;

impl<'a> SerializeProxyType<'a> for BottlerocketSettingsInterface {
    type Proxy = RawValueBox;
}

impl<'a> DeserializeDyn<'a, BottlerocketSettings> for BottlerocketSettingsInterface {
    type Proxy = RawValueRef<'a>;

    fn deserialize_dyn(s: Self::Proxy) -> Result<BottlerocketSettings, RBoxError> {
        BottlerocketSettingsPluginRef::get_module()
            .unwrap()
            .deserialize_settings()(s.get_rstr())
        .into_result()
    }
}

pub type BottlerocketSettings = DynTrait<'static, RBox<()>, BottlerocketSettingsInterface>;

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
pub trait BottlerocketDefaults: Sized {
    fn defaults() -> Self;
}

impl BottlerocketDefaults for BottlerocketSettings {
    fn defaults() -> Self {
        BottlerocketSettingsPluginRef::get_module()
            .unwrap()
            .default_settings()()
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
pub fn deserialize_json<'a, T>(s: RStr<'a>) -> RResult<T, RBoxError>
where
    T: serde::Deserialize<'a>,
{
    match serde_json::from_str::<T>(s.into()) {
        Ok(x) => ROk(x),
        Err(e) => RErr(RBoxError::new(e)),
    }
}

pub fn serialize_json<T>(value: &T) -> Result<RawValueBox, RBoxError>
where
    T: serde::Serialize,
{
    match serde_json::value::to_raw_value::<T>(value) {
        Ok(v) => Ok(v.into()),
        Err(e) => Err(RBoxError::new(e)),
    }
}
