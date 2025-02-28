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
        }
    };

    TokenStream::from(expanded)
}
