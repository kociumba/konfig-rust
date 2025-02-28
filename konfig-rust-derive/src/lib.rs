extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(KonfigSection)]
pub fn konfig_section_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl KonfigSection for #name {
            fn name(&self) -> std::borrow::Cow<'_, str> {
                std::borrow::Cow::Borrowed(stringify!(#name))
            }
            fn validate(&self) -> Result<(), KonfigError> { Ok(()) }
            fn on_load(&self) -> Result<(), KonfigError> { Ok(()) }
            // fn to_bytes(&self, format: &FormatHandlerEnum) -> Result<Vec<u8>, KonfigError> {
            //     format.marshal(self)
            // }
            // fn update_from_bytes(&mut self, bytes: &[u8], format: &FormatHandlerEnum) -> Result<(), KonfigError> {
            //     let new_instance: #name = match format {
            //         FormatHandlerEnum::JSON(_) => {
            //             serde_json::from_slice(bytes)
            //                 .map_err(|err| KonfigError::UnmarshalError(err.to_string()))?
            //         },
            //         FormatHandlerEnum::YAML(_) => {
            //             serde_yaml::from_slice(bytes)
            //                 .map_err(|err| KonfigError::UnmarshalError(err.to_string()))?
            //         },
            //         FormatHandlerEnum::TOML(_) => {
            //             let s = std::str::from_utf8(bytes)
            //                 .map_err(|err| KonfigError::UnmarshalError(err.to_string()))?;
            //             toml::from_str(s)
            //                 .map_err(|err| KonfigError::UnmarshalError(err.to_string()))?
            //         },
            //     };
            //
            //     *self = new_instance;
            //     Ok(())
            // }
        }
    };

    TokenStream::from(expanded)
}
