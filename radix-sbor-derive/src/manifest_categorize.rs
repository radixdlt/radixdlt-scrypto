use proc_macro2::TokenStream;
use syn::Result;

pub fn handle_manifest_categorize(input: TokenStream) -> Result<TokenStream> {
    sbor_derive_common::categorize::handle_categorize(
        input,
        Some("radix_common::data::manifest::ManifestCustomValueKind"),
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
        let output = handle_manifest_categorize(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl sbor::Categorize<radix_common::data::manifest::ManifestCustomValueKind> for MyStruct {
                    #[inline]
                    fn value_kind() -> sbor::ValueKind<radix_common::data::manifest::ManifestCustomValueKind> {
                        sbor::ValueKind::Tuple
                    }
                }

                impl sbor::SborTuple<radix_common::data::manifest::ManifestCustomValueKind> for MyStruct {
                    fn get_length(&self) -> usize {
                        0usize
                    }
                }
            },
        );
    }

    #[test]
    fn test_categorize_enum() {
        let input = TokenStream::from_str("enum MyEnum<T: Bound> { A { named: T }, B(String), C }")
            .unwrap();
        let output = handle_manifest_categorize(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl<T: Bound> sbor::Categorize<radix_common::data::manifest::ManifestCustomValueKind>
                    for MyEnum<T>
                {
                    #[inline]
                    fn value_kind() -> sbor::ValueKind<radix_common::data::manifest::ManifestCustomValueKind> {
                        sbor::ValueKind::Enum
                    }
                }

                impl<T: Bound> sbor::SborEnum<radix_common::data::manifest::ManifestCustomValueKind> for MyEnum<T> {
                    fn get_discriminator(&self) -> u8 {
                        match self {
                            Self::A { .. } => 0u8,
                            Self::B(_) => 1u8,
                            Self::C => 2u8,
                        }
                    }

                    fn get_length(&self) -> usize {
                        match self {
                            Self::A { .. } => 1usize,
                            Self::B(_) => 1usize,
                            Self::C => 0usize,
                        }
                    }
                }
            },
        );
    }
}
