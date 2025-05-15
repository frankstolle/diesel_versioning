use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

mod model;
mod versioned;

///
/// Dervice macro to implement Versioned.
///
#[proc_macro_derive(Versioned, attributes(diesel, version))]
#[proc_macro_error]
pub fn derive_versioned(input: TokenStream) -> TokenStream {
    versioned::derive(parse_macro_input!(input), false)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

///
/// Dervice macro to implement VersionedAsync.
///
#[cfg(feature = "async")]
#[proc_macro_derive(VersionedAsync, attributes(diesel, version))]
#[proc_macro_error]
pub fn derive_versioned_async(input: TokenStream) -> TokenStream {
    versioned::derive(parse_macro_input!(input), true)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
