mod custom_value;
mod custom_value_kind;
pub mod model;

pub use custom_value::*;
pub use custom_value_kind::*;

use sbor::*;

pub const MANIFEST_SBOR_V1_PAYLOAD_PREFIX: u8 = 77; // [M] ASCII code
pub const MANIFEST_SBOR_V1_MAX_DEPTH: u8 = 16;

pub type ManifestEncoder<'a> = VecEncoder<'a, ManifestCustomValueKind, MANIFEST_SBOR_V1_MAX_DEPTH>;
pub type ManifestDecoder<'a> = VecDecoder<'a, ManifestCustomValueKind, MANIFEST_SBOR_V1_MAX_DEPTH>;
pub type ManifestValueKind = ValueKind<ManifestCustomValueKind>;
pub type ManifestValue = Value<ManifestCustomValueKind, ManifestCustomValue>;

pub trait ManifestCategorize: Categorize<ManifestCustomValueKind> {}
impl<T: Categorize<ManifestCustomValueKind> + ?Sized> ManifestCategorize for T {}

pub trait ManifestDecode: for<'a> Decode<ManifestCustomValueKind, ManifestDecoder<'a>> {}
impl<T: for<'a> Decode<ManifestCustomValueKind, ManifestDecoder<'a>>> ManifestDecode for T {}

pub trait ManifestEncode: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>> {}
impl<T: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>> + ?Sized> ManifestEncode
    for T
{
}

pub fn manifest_encode<T: ManifestEncode + ?Sized>(value: &T) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::with_capacity(512);
    let encoder = ManifestEncoder::new(&mut buf);
    encoder.encode_payload(value, MANIFEST_SBOR_V1_PAYLOAD_PREFIX)?;
    Ok(buf)
}

pub fn manifest_decode<T: ManifestDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    ManifestDecoder::new(buf).decode_payload(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
}

#[macro_export]
macro_rules! manifest_type {
    // without describe
    ($t:ty, $value_kind:expr, $size: expr) => {
        impl sbor::Categorize<transaction_data::ManifestCustomValueKind> for $t {
            #[inline]
            fn value_kind() -> sbor::ValueKind<transaction_data::ManifestCustomValueKind> {
                sbor::ValueKind::Custom($value_kind)
            }
        }

        impl<E: sbor::Encoder<transaction_data::ManifestCustomValueKind>>
            sbor::Encode<transaction_data::ManifestCustomValueKind, E> for $t
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

        impl<D: sbor::Decoder<transaction_data::ManifestCustomValueKind>>
            sbor::Decode<transaction_data::ManifestCustomValueKind, D> for $t
        {
            fn decode_body_with_value_kind(
                decoder: &mut D,
                value_kind: sbor::ValueKind<transaction_data::ManifestCustomValueKind>,
            ) -> Result<Self, sbor::DecodeError> {
                decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
                let slice = decoder.read_slice($size)?;
                Self::try_from(slice).map_err(|_| sbor::DecodeError::InvalidCustomValue)
            }
        }
    };
}

// Re-export transaction derive.
extern crate transaction_derive;
pub use transaction_derive::{ManifestCategorize, ManifestDecode, ManifestEncode};

// Make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
pub extern crate self as transaction_data;

#[macro_export]
macro_rules! count {
    () => {0usize};
    ($a:expr) => {1usize};
    ($a:expr, $($rest:expr),*) => {1usize + $crate::count!($($rest),*)};
}

#[macro_export]
macro_rules! manifest_args {
    ($($args: expr),*) => {{
        use ::sbor::Encoder;
        let mut buf = ::sbor::rust::vec::Vec::new();
        let mut encoder = $crate::ManifestEncoder::new(&mut buf);
        encoder.write_payload_prefix($crate::MANIFEST_SBOR_V1_PAYLOAD_PREFIX).unwrap();
        encoder.write_value_kind($crate::ManifestValueKind::Tuple).unwrap();
        // Hack: stringify to skip ownership move semantics
        encoder.write_size($crate::count!($(stringify!($args)),*)).unwrap();
        $(
            let arg = $args;
            encoder.encode(&arg).unwrap();
        )*
        buf
    }};
}
