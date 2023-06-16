use sbor::rust::vec::Vec;
use sbor::traversal::VecTraverser;
use sbor::*;

mod custom_extension;
mod custom_formatting;
mod custom_payload_wrappers;
#[cfg(feature = "serde")]
mod custom_serde;
mod custom_traversal;
mod custom_value;
mod custom_value_kind;
mod display_context;

pub mod converter;
mod custom_validation;
pub mod model;
pub use custom_extension::*;
pub use custom_formatting::*;
pub use custom_payload_wrappers::*;
#[cfg(feature = "serde")]
pub use custom_serde::*;
pub use custom_traversal::*;
pub use custom_value::*;
pub use custom_value_kind::*;
pub use display_context::*;

pub use radix_engine_constants::MANIFEST_SBOR_V1_PAYLOAD_PREFIX;
pub const MANIFEST_SBOR_V1_MAX_DEPTH: usize = 24;

pub type ManifestEncoder<'a> = VecEncoder<'a, ManifestCustomValueKind>;
pub type ManifestDecoder<'a> = VecDecoder<'a, ManifestCustomValueKind>;
pub type ManifestValueKind = ValueKind<ManifestCustomValueKind>;
pub type ManifestValue = Value<ManifestCustomValueKind, ManifestCustomValue>;
pub type ManifestEnumVariantValue = EnumVariantValue<ManifestCustomValueKind, ManifestCustomValue>;
pub type ManifestTraverser<'a> = VecTraverser<'a, ManifestCustomTraversal>;

pub trait ManifestCategorize: Categorize<ManifestCustomValueKind> {}
impl<T: Categorize<ManifestCustomValueKind> + ?Sized> ManifestCategorize for T {}

pub trait ManifestSborEnum: SborEnum<ManifestCustomValueKind> {}
impl<T: SborEnum<ManifestCustomValueKind> + ?Sized> ManifestSborEnum for T {}

pub trait ManifestSborTuple: SborTuple<ManifestCustomValueKind> {}
impl<T: SborTuple<ManifestCustomValueKind> + ?Sized> ManifestSborTuple for T {}

pub trait ManifestDecode: for<'a> Decode<ManifestCustomValueKind, ManifestDecoder<'a>> {}
impl<T: for<'a> Decode<ManifestCustomValueKind, ManifestDecoder<'a>>> ManifestDecode for T {}

pub trait ManifestEncode: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>> {}
impl<T: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>> + ?Sized> ManifestEncode
    for T
{
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueConversionError {
    DecodeError(DecodeError),
    EncodeError(EncodeError),
}

pub fn manifest_encode<T: ManifestEncode + ?Sized>(value: &T) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::with_capacity(512);
    let encoder = ManifestEncoder::new(&mut buf, MANIFEST_SBOR_V1_MAX_DEPTH);
    encoder.encode_payload(value, MANIFEST_SBOR_V1_PAYLOAD_PREFIX)?;
    Ok(buf)
}

pub fn manifest_decode<T: ManifestDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    ManifestDecoder::new(buf, MANIFEST_SBOR_V1_MAX_DEPTH)
        .decode_payload(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
}

pub fn to_manifest_value<T: ManifestEncode + ?Sized>(
    value: &T,
) -> Result<ManifestValue, ValueConversionError> {
    let encoded = manifest_encode(value).map_err(ValueConversionError::EncodeError)?;

    manifest_decode(&encoded).map_err(ValueConversionError::DecodeError)
}

pub fn from_manifest_value<T: ManifestDecode>(
    manifest_value: &ManifestValue,
) -> Result<T, ValueConversionError> {
    let encoded = manifest_encode(manifest_value).map_err(ValueConversionError::EncodeError)?;

    manifest_decode(&encoded).map_err(ValueConversionError::DecodeError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_value_conversions() {
        let invalid_value = ManifestValue::Tuple {
            fields: vec![ManifestValue::Array {
                element_value_kind: ValueKind::U8,
                elements: vec![
                    ManifestValue::U8 { value: 1 },
                    ManifestValue::U16 { value: 2 },
                ],
            }],
        };
        assert!(matches!(
            to_manifest_value(&invalid_value),
            Err(ValueConversionError::EncodeError(
                EncodeError::MismatchingArrayElementValueKind { .. }
            ))
        ));

        let invalid_value = ManifestValue::Map {
            key_value_kind: ValueKind::U8,
            value_value_kind: ValueKind::I8,
            entries: vec![(
                ManifestValue::U16 { value: 1 },
                ManifestValue::I8 { value: 1 },
            )],
        };
        assert!(matches!(
            to_manifest_value(&invalid_value),
            Err(ValueConversionError::EncodeError(
                EncodeError::MismatchingMapKeyValueKind { .. }
            ))
        ));

        let invalid_value = ManifestValue::Map {
            key_value_kind: ValueKind::U8,
            value_value_kind: ValueKind::I8,
            entries: vec![(
                ManifestValue::U8 { value: 1 },
                ManifestValue::I16 { value: 1 },
            )],
        };
        assert!(matches!(
            to_manifest_value(&invalid_value),
            Err(ValueConversionError::EncodeError(
                EncodeError::MismatchingMapValueValueKind { .. }
            ))
        ));

        let too_deep_tuple = get_tuple_of_depth(MANIFEST_SBOR_V1_MAX_DEPTH + 1).unwrap();
        assert!(matches!(
            to_manifest_value(&too_deep_tuple),
            Err(ValueConversionError::EncodeError(
                EncodeError::MaxDepthExceeded { .. }
            ))
        ));

        let fine_tuple = get_tuple_of_depth(MANIFEST_SBOR_V1_MAX_DEPTH).unwrap();
        assert!(to_manifest_value(&fine_tuple).is_ok());
    }

    pub fn get_tuple_of_depth(depth: usize) -> Option<ManifestValue> {
        // Minimum tuple depth is 2
        if depth <= 1 {
            None
        } else if depth <= 2 {
            Some(ManifestValue::Tuple {
                fields: vec![ManifestValue::U8 { value: 1 }],
            })
        } else {
            let value = get_tuple_of_depth(depth - 1).unwrap();
            Some(ManifestValue::Tuple {
                fields: vec![value],
            })
        }
    }
}
