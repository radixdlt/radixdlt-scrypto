use proc_macro2::TokenStream;
use syn::Result;

pub fn handle_scrypto_categorize(input: TokenStream) -> Result<TokenStream> {
    sbor_derive_common::categorize::handle_categorize(
        input,
        Some("radix_engine_common::data::scrypto::ScryptoCustomValueKind"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;
    use std::str::FromStr;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_categorize_struct() {
        let input = TokenStream::from_str("pub struct MyStruct { }").unwrap();
        let output = handle_scrypto_categorize(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Categorize<radix_engine_common::data::scrypto::ScryptoCustomValueKind> for MyStruct {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<radix_engine_common::data::scrypto::ScryptoCustomValueKind> {
                        ::sbor::ValueKind::Tuple
                    }
                }
            },
        );
    }

    #[test]
    fn test_categorize_enum() {
        let input = TokenStream::from_str("enum MyEnum<T: Bound> { A { named: T }, B(String), C }")
            .unwrap();
        let output = handle_scrypto_categorize(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl<T: Bound> ::sbor::Categorize<radix_engine_common::data::scrypto::ScryptoCustomValueKind>
                    for MyEnum<T>
                {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<radix_engine_common::data::scrypto::ScryptoCustomValueKind> {
                        ::sbor::ValueKind::Enum
                    }
                }
            },
        );
    }
}
