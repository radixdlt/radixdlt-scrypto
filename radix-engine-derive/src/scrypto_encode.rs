use proc_macro2::TokenStream;
use syn::Result;

pub fn handle_scrypto_encode(input: TokenStream) -> Result<TokenStream> {
    sbor_derive_common::encode::handle_encode(
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
    fn test_encode_struct() {
        let input = TokenStream::from_str("pub struct MyStruct { }").unwrap();
        let output = handle_scrypto_encode(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl<E: ::sbor::Encoder<radix_engine_common::data::scrypto::ScryptoCustomValueKind> >
                    ::sbor::Encode<radix_engine_common::data::scrypto::ScryptoCustomValueKind, E> for MyStruct
                {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        encoder.write_value_kind(::sbor::ValueKind::Tuple)
                    }
                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        use ::sbor::{self, Encode};
                        encoder.write_size(0)?;
                        Ok(())
                    }
                }
            },
        );
    }

    #[test]
    fn test_encode_enum() {
        let input = TokenStream::from_str("enum MyEnum<T: Bound> { A { named: T }, B(String), C }")
            .unwrap();
        let output = handle_scrypto_encode(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl<
                        T: Bound,
                        E: ::sbor::Encoder<radix_engine_common::data::scrypto::ScryptoCustomValueKind>
                    > ::sbor::Encode<radix_engine_common::data::scrypto::ScryptoCustomValueKind, E> for MyEnum<T>
                where
                    T: ::sbor::Encode<radix_engine_common::data::scrypto::ScryptoCustomValueKind, E>,
                    T: ::sbor::Categorize<radix_engine_common::data::scrypto::ScryptoCustomValueKind>
                {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        encoder.write_value_kind(::sbor::ValueKind::Enum)
                    }
                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        use ::sbor::{self, Encode};
                        match self {
                            Self::A { named, .. } => {
                                encoder.write_discriminator(0u8)?;
                                encoder.write_size(1)?;
                                encoder.encode(named)?;
                            }
                            Self::B(a0) => {
                                encoder.write_discriminator(1u8)?;
                                encoder.write_size(1)?;
                                encoder.encode(a0)?;
                            }
                            Self::C => {
                                encoder.write_discriminator(2u8)?;
                                encoder.write_size(0)?;
                            }
                        }
                        Ok(())
                    }
                }
            },
        );
    }
}
