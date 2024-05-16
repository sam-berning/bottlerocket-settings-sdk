use darling::{FromDeriveInput, ToTokens};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SettingsPlugin)]
pub fn derive_settings(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let n = SettingsPlugin::from_derive_input(&ast).expect("Unable to parse macro arguments");
    quote!(#n).into()
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct SettingsPlugin {
    ident: syn::Ident,
}

impl ToTokens for SettingsPlugin {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let SettingsPlugin { ident } = self;
        tokens.extend(quote! {
            impl<'a> abi_stable::erased_types::SerializeType<'a> for #ident {
                type Interface = bottlerocket_settings_plugin::BottlerocketSettingsInterface;

                fn serialize_impl(&'a self) -> Result<abi_stable::external_types::RawValueBox, abi_stable::std_types::RBoxError> {
                    bottlerocket_settings_plugin::serialize_json(self)
                }
            }

            #[abi_stable::sabi_extern_fn]
            fn deserialize_settings(s: abi_stable::std_types::RStr<'_>) -> abi_stable::std_types::RResult<bottlerocket_settings_plugin::BottlerocketSettings, abi_stable::std_types::RBoxError> {
                abi_stable::extern_fn_panic_handling! {
                    bottlerocket_settings_plugin::deserialize_json::<#ident>(s).map(abi_stable::DynTrait::from_value)
                }
            }

            #[abi_stable::sabi_extern_fn]
            fn default_settings() -> bottlerocket_settings_plugin::BottlerocketSettings {
                abi_stable::extern_fn_panic_handling! {
                    abi_stable::DynTrait::from_value(#ident::default())
                }
            }

            #[abi_stable::export_root_module]
            fn get_library() -> bottlerocket_settings_plugin::BottlerocketSettingsPluginRef {
                abi_stable::prefix_type::PrefixTypeTrait::leak_into_prefix(
                    bottlerocket_settings_plugin::BottlerocketSettingsPlugin {
                        default_settings,
                        deserialize_settings,
                    })
            }
        });
    }
}
