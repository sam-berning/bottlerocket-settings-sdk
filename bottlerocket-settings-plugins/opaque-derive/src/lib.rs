use darling::{FromDeriveInput, ToTokens};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Opaque)]
pub fn derive_opaque(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let n = Opaque::from_derive_input(&ast).expect("Unable to parse macro arguments");
    quote!(#n).into()
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct Opaque {
    ident: syn::Ident,
}

impl ToTokens for Opaque {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Opaque { ident } = self;
        tokens.extend(quote! {
            impl<'a> SerializeType<'a> for #ident {
                type Interface = OpaqueSettingsInterface;

                fn serialize_impl(&'a self) -> Result<RawValueBox, RBoxError> {
                    serialize_json(self)
                }
            }

            #[sabi_extern_fn]
            fn deserialize_settings(s: RStr<'_>) -> RResult<OpaqueSettings, RBoxError> {
                extern_fn_panic_handling! {
                    deserialize_json::<#ident>(s).map(DynTrait::from_value)
                }
            }

            #[sabi_extern_fn]
            fn default_settings() -> OpaqueSettings {
                extern_fn_panic_handling! {
                    DynTrait::from_value(#ident::default())
                }
            }

            #[export_root_module]
            pub fn get_library() -> OpaqueSettingsPluginRef {
                OpaqueSettingsPlugin {
                    default_settings,
                    deserialize_settings,
                }
                .leak_into_prefix()
            }
        });
    }
}
