use proc_macro2::TokenStream;
use syn::Result;

pub fn handle_categorize(input: TokenStream) -> Result<TokenStream> {
    sbor_derive_common::categorize::handle_categorize(
        input,
        Some("transaction::data::ManifestCustomValueKind"),
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
        let output = handle_categorize(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Categorize<transaction::data::ManifestCustomValueKind> for MyStruct {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<transaction::data::ManifestCustomValueKind> {
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
        let output = handle_categorize(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl<T: Bound> ::sbor::Categorize<transaction::data::ManifestCustomValueKind>
                    for MyEnum<T>
                {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<transaction::data::ManifestCustomValueKind> {
                        ::sbor::ValueKind::Enum
                    }
                }
            },
        );
    }
}
