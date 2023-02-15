pub mod builder;
pub mod data;
pub mod errors;
pub mod manifest;
pub mod model;
pub mod signing;
pub mod validation;

// Re-export transaction derive.
extern crate transaction_derive;
pub use transaction_derive::{ManifestCategorize, ManifestDecode, ManifestEncode};

extern crate self as transaction;

#[macro_export]
macro_rules! manifest_type {
    // without describe
    ($t:ty, $value_kind:expr, $size: expr) => {
        impl sbor::Categorize<transaction::data::ManifestCustomValueKind> for $t {
            #[inline]
            fn value_kind() -> sbor::ValueKind<transaction::data::ManifestCustomValueKind> {
                sbor::ValueKind::Custom($value_kind)
            }
        }

        impl<E: sbor::Encoder<transaction::data::ManifestCustomValueKind>>
            sbor::Encode<transaction::data::ManifestCustomValueKind, E> for $t
        {
            #[inline]
            fn encode_value_kind(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                encoder.write_value_kind(Self::value_kind())
            }

            #[inline]
            fn encode_body(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                encoder.write_slice(&self.to_vec())
            }
        }

        impl<D: sbor::Decoder<transaction::data::ManifestCustomValueKind>>
            sbor::Decode<transaction::data::ManifestCustomValueKind, D> for $t
        {
            fn decode_body_with_value_kind(
                decoder: &mut D,
                value_kind: sbor::ValueKind<transaction::data::ManifestCustomValueKind>,
            ) -> Result<Self, sbor::DecodeError> {
                decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
                let slice = decoder.read_slice($size)?;
                Self::try_from(slice).map_err(|_| sbor::DecodeError::InvalidCustomValue)
            }
        }
    };
}
