use crate::data::manifest::model::*;
use crate::data::manifest::*;
use crate::*;
use sbor::value_kind::*;
use sbor::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestCustomValue {
    Address(ManifestAddress),
    Bucket(ManifestBucket),
    Proof(ManifestProof),
    Expression(ManifestExpression),
    Blob(ManifestBlobRef),
    Decimal(ManifestDecimal),
    PreciseDecimal(ManifestPreciseDecimal),
    NonFungibleLocalId(ManifestNonFungibleLocalId),
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for ManifestCustomValue
{
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            ManifestCustomValue::Address(_) => {
                encoder.write_value_kind(ValueKind::Custom(ManifestCustomValueKind::Address))
            }
            ManifestCustomValue::Bucket(_) => {
                encoder.write_value_kind(ValueKind::Custom(ManifestCustomValueKind::Bucket))
            }
            ManifestCustomValue::Proof(_) => {
                encoder.write_value_kind(ValueKind::Custom(ManifestCustomValueKind::Proof))
            }
            ManifestCustomValue::Expression(_) => {
                encoder.write_value_kind(ValueKind::Custom(ManifestCustomValueKind::Expression))
            }
            ManifestCustomValue::Blob(_) => {
                encoder.write_value_kind(ValueKind::Custom(ManifestCustomValueKind::Blob))
            }
            ManifestCustomValue::Decimal(_) => {
                encoder.write_value_kind(ValueKind::Custom(ManifestCustomValueKind::Decimal))
            }
            ManifestCustomValue::PreciseDecimal(_) => {
                encoder.write_value_kind(ValueKind::Custom(ManifestCustomValueKind::PreciseDecimal))
            }
            ManifestCustomValue::NonFungibleLocalId(_) => encoder.write_value_kind(
                ValueKind::Custom(ManifestCustomValueKind::NonFungibleLocalId),
            ),
        }
    }

    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            // TODO: vector free
            ManifestCustomValue::Address(v) => v.encode_body(encoder),
            ManifestCustomValue::Bucket(v) => v.encode_body(encoder),
            ManifestCustomValue::Proof(v) => v.encode_body(encoder),
            ManifestCustomValue::Expression(v) => v.encode_body(encoder),
            ManifestCustomValue::Blob(v) => v.encode_body(encoder),
            ManifestCustomValue::Decimal(v) => v.encode_body(encoder),
            ManifestCustomValue::PreciseDecimal(v) => v.encode_body(encoder),
            ManifestCustomValue::NonFungibleLocalId(v) => v.encode_body(encoder),
        }
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for ManifestCustomValue
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        match value_kind {
            ValueKind::Custom(cti) => match cti {
                ManifestCustomValueKind::Address => {
                    ManifestAddress::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::Address)
                }
                ManifestCustomValueKind::Blob => {
                    ManifestBlobRef::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::Blob)
                }
                ManifestCustomValueKind::Bucket => {
                    ManifestBucket::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::Bucket)
                }
                ManifestCustomValueKind::Proof => {
                    ManifestProof::decode_body_with_value_kind(decoder, value_kind).map(Self::Proof)
                }
                ManifestCustomValueKind::Expression => {
                    ManifestExpression::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::Expression)
                }
                ManifestCustomValueKind::Decimal => {
                    ManifestDecimal::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::Decimal)
                }
                ManifestCustomValueKind::PreciseDecimal => {
                    ManifestPreciseDecimal::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::PreciseDecimal)
                }
                ManifestCustomValueKind::NonFungibleLocalId => {
                    ManifestNonFungibleLocalId::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::NonFungibleLocalId)
                }
            },
            _ => Err(DecodeError::UnexpectedCustomValueKind {
                actual: value_kind.as_u8(),
            }),
        }
    }
}
