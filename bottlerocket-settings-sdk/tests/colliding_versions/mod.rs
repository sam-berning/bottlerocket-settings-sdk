use bottlerocket_settings_sdk::{
    BottlerocketSetting, GenerateResult, LinearMigratorExtensionBuilder, SettingsModel,
};
use bottlerocket_settings_sdk::settings_derive::LinearlyMigrateable;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

#[test]
fn test_no_colliding_model_versions() {
    // Given two models with the same version string,
    // When an extension is built with those models,
    // The extension will fail to build.

    assert!(
        LinearMigratorExtensionBuilder::with_name("colliding-versions")
            .with_models(vec![
                BottlerocketSetting::<ModelA>::model(),
                BottlerocketSetting::<ModelB>::model(),
            ])
            .build()
            .is_err()
    );
}

#[derive(Debug, Snafu)]
pub struct MyError;

type Result<T> = std::result::Result<T, MyError>;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, LinearlyMigrateable)]
pub struct ModelA;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, LinearlyMigrateable)]
pub struct ModelB;

impl SettingsModel for ModelA {
    type PartialKind = Self;
    type ErrorKind = MyError;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_: Option<Self>, _: Self) -> Result<()> {
        unimplemented!()
    }

    fn generate(
        _: Option<Self::PartialKind>,
        _: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        unimplemented!()
    }

    fn validate(_: Self, _: Option<serde_json::Value>) -> Result<()> {
        unimplemented!()
    }
}

impl SettingsModel for ModelB {
    type PartialKind = Self;
    type ErrorKind = MyError;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_: Option<Self>, _: Self) -> Result<()> {
        unimplemented!()
    }

    fn generate(
        _: Option<Self::PartialKind>,
        _: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        unimplemented!()
    }

    fn validate(_: Self, _: Option<serde_json::Value>) -> Result<()> {
        unimplemented!()
    }
}
