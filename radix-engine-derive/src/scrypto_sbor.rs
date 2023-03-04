use proc_macro2::TokenStream;
use syn::Result;

pub fn handle_scrypto_sbor(input: TokenStream) -> Result<TokenStream> {
    sbor_derive_common::sbor::handle_sbor(
        input,
        Some("radix_engine_common::data::scrypto::ScryptoCustomValueKind"),
        Some("radix_engine_common::data::scrypto::ScryptoCustomTypeKind"),
    )
}
