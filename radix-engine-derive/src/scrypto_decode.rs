use proc_macro2::TokenStream;
use syn::Result;

pub fn handle_scrypto_decode(input: TokenStream) -> Result<TokenStream> {
    sbor_derive_common::decode::handle_decode(
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
    fn test_decode_struct() {
        let input = TokenStream::from_str("pub struct MyStruct { }").unwrap();
        let output = handle_scrypto_decode(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl<D: ::sbor::Decoder<radix_engine_common::data::scrypto::ScryptoCustomValueKind> >
                    ::sbor::Decode<radix_engine_common::data::scrypto::ScryptoCustomValueKind, D> for MyStruct
                {
                    #[inline]
                    fn decode_body_with_value_kind(
                        decoder: &mut D,
                        value_kind: ::sbor::ValueKind<radix_engine_common::data::scrypto::ScryptoCustomValueKind>
                    ) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Tuple)?;
                        decoder.read_and_check_size(0)?;
                        Ok(Self {})
                    }
                }
            },
        );
    }

    #[test]
    fn test_decode_enum() {
        let input = TokenStream::from_str("enum MyEnum<T: Bound> { A { named: T }, B(String), C }")
            .unwrap();
        let output = handle_scrypto_decode(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl<
                        T: Bound,
                        D: ::sbor::Decoder<radix_engine_common::data::scrypto::ScryptoCustomValueKind>
                    > ::sbor::Decode<radix_engine_common::data::scrypto::ScryptoCustomValueKind, D> for MyEnum<T>
                where
                    T: ::sbor::Decode<radix_engine_common::data::scrypto::ScryptoCustomValueKind, D>,
                    T: ::sbor::Categorize<radix_engine_common::data::scrypto::ScryptoCustomValueKind>
                {
                    #[inline]
                    fn decode_body_with_value_kind(
                        decoder: &mut D,
                        value_kind: ::sbor::ValueKind<radix_engine_common::data::scrypto::ScryptoCustomValueKind>
                    ) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Enum)?;
                        let discriminator = decoder.read_discriminator()?;
                        match discriminator {
                            0u8 => {
                                decoder.read_and_check_size(1)?;
                                Ok(Self::A {
                                    named: decoder.decode::<T>()?,
                                })
                            },
                            1u8 => {
                                decoder.read_and_check_size(1)?;
                                Ok(Self::B(decoder.decode::<String>()?,))
                            },
                            2u8 => {
                                decoder.read_and_check_size(0)?;
                                Ok(Self::C)
                            },
                            _ => Err(::sbor::DecodeError::UnknownDiscriminator(discriminator))
                        }
                    }
                }
            },
        );
    }
}
