/// A macro for implementing sbor traits.
#[macro_export]
macro_rules! scrypto_type {
    // static size
    ($t:ty, $type_id:expr, $schema_type: expr, $size: expr) => {
        impl sbor::TypeId<crate::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn type_id() -> sbor::SborTypeId<crate::data::ScryptoCustomTypeId> {
                sbor::SborTypeId::Custom($type_id)
            }
        }

        impl sbor::Encode<crate::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn encode_type_id(encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>) {
                encoder.write_type_id(Self::type_id());
            }
            #[inline]
            fn encode_value(&self, encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>) {
                encoder.write_slice(&self.to_vec());
            }
        }

        impl sbor::Decode<crate::data::ScryptoCustomTypeId> for $t {
            fn check_type_id(
                decoder: &mut sbor::Decoder<crate::data::ScryptoCustomTypeId>,
            ) -> Result<(), sbor::DecodeError> {
                decoder.check_type_id(Self::type_id())
            }

            fn decode_value(
                decoder: &mut sbor::Decoder<crate::data::ScryptoCustomTypeId>,
            ) -> Result<Self, sbor::DecodeError> {
                let slice = decoder.read_slice($size)?;
                Self::try_from(slice).map_err(|_| sbor::DecodeError::InvalidCustomValue)
            }
        }

        impl scrypto_abi::Describe for $t {
            fn describe() -> scrypto_abi::Type {
                $schema_type
            }
        }
    };

    // dynamic size
    ($t:ty, $type_id:expr, $schema_type: expr) => {
        impl sbor::TypeId<crate::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn type_id() -> sbor::SborTypeId<crate::data::ScryptoCustomTypeId> {
                sbor::SborTypeId::Custom($type_id)
            }
        }

        impl sbor::Encode<crate::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn encode_type_id(encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>) {
                encoder.write_type_id(Self::type_id());
            }
            #[inline]
            fn encode_value(&self, encoder: &mut sbor::Encoder<crate::data::ScryptoCustomTypeId>) {
                let bytes = self.to_vec();
                encoder.write_size(bytes.len());
                encoder.write_slice(&bytes);
            }
        }

        impl sbor::Decode<crate::data::ScryptoCustomTypeId> for $t {
            fn check_type_id(
                decoder: &mut sbor::Decoder<crate::data::ScryptoCustomTypeId>,
            ) -> Result<(), sbor::DecodeError> {
                decoder.check_type_id(Self::type_id())
            }

            fn decode_value(
                decoder: &mut sbor::Decoder<crate::data::ScryptoCustomTypeId>,
            ) -> Result<Self, sbor::DecodeError> {
                let len = decoder.read_size()?;
                let slice = decoder.read_slice(len)?;
                Self::try_from(slice).map_err(|_| sbor::DecodeError::InvalidCustomValue)
            }
        }

        impl scrypto_abi::Describe for $t {
            fn describe() -> scrypto_abi::Type {
                $schema_type
            }
        }
    };
}
