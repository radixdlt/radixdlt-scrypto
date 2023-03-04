use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_manifest_sbor(input: TokenStream) -> Result<TokenStream> {
    let context_custom_value_kind =
        Some("radix_engine_common::data::manifest::ManifestCustomValueKind");

    trace!("handle_manifest_sbor() starts");

    let categorize = sbor_derive_common::categorize::handle_categorize(
        input.clone(),
        context_custom_value_kind.clone(),
    )?;
    let encode = sbor_derive_common::encode::handle_encode(
        input.clone(),
        context_custom_value_kind.clone(),
    )?;
    let decode = sbor_derive_common::decode::handle_decode(
        input.clone(),
        context_custom_value_kind.clone(),
    )?;

    let output = quote! {
        #categorize

        #encode

        #decode
    };

    trace!("handle_manifest_sbor() finishes");
    Ok(output)
}
