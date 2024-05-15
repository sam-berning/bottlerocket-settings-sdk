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
#[sabi(kind(Prefix(prefix_ref = OpaqueSettingsPluginRef)))]
#[sabi(missing_field(panic))]
pub struct OpaqueSettingsPlugin {
    pub default_settings: extern "C" fn() -> OpaqueSettings,
    #[sabi(last_prefix_field)]
    pub deserialize_settings:
        for<'a> extern "C" fn(RStr<'a>) -> RResult<OpaqueSettings, RBoxError>,
}

impl RootModule for OpaqueSettingsPluginRef {
    const BASE_NAME: &'static str = "settings";
    const NAME: &'static str = "settings";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();

    abi_stable::declare_root_module_statics! {OpaqueSettingsPluginRef}
}

// FIXME: we could prefer to load "lib{variant}.so" and fall back to "libsettings.so" if not found
// ideally we'd be able to build all the different settings plugins in the current main repo and
// have them not clobber each other
lazy_static! {
    static ref PLUGIN: Result<OpaqueSettingsPluginRef, LibraryError> =
        OpaqueSettingsPluginRef::load_from_file(&PathBuf::from(format!("lib{}.so", OpaqueSettingsPluginRef::NAME)));
}

// FIXME: error handling is somewhat fraught, we can't really recover from a load error, so
// perhaps a panic is correct
impl OpaqueSettingsPluginRef {
    pub fn try_load() {
        let _ = *PLUGIN;
        if let Err(e) = PLUGIN.as_ref() {
            eprintln!("{e}");
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
#[allow(dead_code)]
pub struct OpaqueSettingsPluginLoader;

#[allow(dead_code)]
impl OpaqueSettingsPluginLoader {
    pub fn load() {
        OpaqueSettingsPluginRef::try_load()
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Sync, Send, Default, Clone, Debug, Deserialize, Serialize, PartialEq))]
pub struct OpaqueSettingsInterface;

impl<'a> SerializeProxyType<'a> for OpaqueSettingsInterface {
    type Proxy = RawValueBox;
}

impl<'a> DeserializeDyn<'a, OpaqueSettings> for OpaqueSettingsInterface {
    type Proxy = RawValueRef<'a>;

    fn deserialize_dyn(s: Self::Proxy) -> Result<OpaqueSettings, RBoxError> {
        OpaqueSettingsPluginRef::get_module()
            .unwrap()
            .deserialize_settings()(s.get_rstr())
        .into_result()
    }
}

pub type OpaqueSettings = DynTrait<'static, RBox<()>, OpaqueSettingsInterface>;

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
pub trait OpaqueDefault: Sized {
    fn opaque_default() -> Self;
}

impl OpaqueDefault for OpaqueSettings {
    fn opaque_default() -> Self {
        OpaqueSettingsPluginRef::get_module()
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
