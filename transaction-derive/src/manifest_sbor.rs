use proc_macro2::TokenStream;
use syn::Result;

pub fn handle_manifest_sbor(input: TokenStream) -> Result<TokenStream> {
    sbor_derive_common::sbor::handle_sbor(
        input,
        Some("transaction_data::ManifestCustomValueKind"),
        Some("transaction_data::ManifestCustomTypeKind<::sbor::GlobalTypeId>"),
    )
}
