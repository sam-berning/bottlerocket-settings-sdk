extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// A convenience macro for settings that have only one version and hence require no migrations. 
#[proc_macro_derive(LinearlyMigrateable)]
pub fn derive_no_migration(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = ast.ident;

    quote!(
        impl bottlerocket_settings_sdk::LinearlyMigrateable for #name {
            type ForwardMigrationTarget = bottlerocket_settings_sdk::NoMigration;
            type BackwardMigrationTarget = bottlerocket_settings_sdk::NoMigration;

            fn migrate_forward(&self) -> Result<Self::ForwardMigrationTarget> {
                bottlerocket_settings_sdk::NoMigration::no_defined_migration()
            }

            fn migrate_backward(&self) -> Result<Self::BackwardMigrationTarget> {
                bottlerocket_settings_sdk::NoMigration::no_defined_migration()
            }
        }
    ).into()
}
