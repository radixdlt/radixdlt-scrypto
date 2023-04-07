use crate::data::scrypto::model::*;
use crate::data::scrypto::*;
use crate::math::{Decimal, PreciseDecimal};
use sbor::value_kind::*;
use sbor::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomValue {
    Reference(Reference),
    Own(Own),
    Decimal(Decimal),
    PreciseDecimal(PreciseDecimal),
    NonFungibleLocalId(NonFungibleLocalId),
}

impl ScryptoCustomValue {
    pub fn get_custom_value_kind(&self) -> ScryptoCustomValueKind {
        match self {
            ScryptoCustomValue::Reference(_) => ScryptoCustomValueKind::Reference,
            ScryptoCustomValue::Own(_) => ScryptoCustomValueKind::Own,
            ScryptoCustomValue::Decimal(_) => ScryptoCustomValueKind::Decimal,
            ScryptoCustomValue::PreciseDecimal(_) => ScryptoCustomValueKind::PreciseDecimal,
            ScryptoCustomValue::NonFungibleLocalId(_) => ScryptoCustomValueKind::NonFungibleLocalId,
        }
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for ScryptoCustomValue {
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(ValueKind::Custom(self.get_custom_value_kind()))
    }

    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            // TODO: vector free
            ScryptoCustomValue::Reference(v) => v.encode_body(encoder),
            ScryptoCustomValue::Own(v) => v.encode_body(encoder),
            ScryptoCustomValue::Decimal(v) => v.encode_body(encoder),
            ScryptoCustomValue::PreciseDecimal(v) => v.encode_body(encoder),
            ScryptoCustomValue::NonFungibleLocalId(v) => v.encode_body(encoder),
        }
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for ScryptoCustomValue {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        match value_kind {
            ValueKind::Custom(cti) => match cti {
                ScryptoCustomValueKind::Reference => {
                    Reference::decode_body_with_value_kind(decoder, value_kind).map(Self::Reference)
                }
                ScryptoCustomValueKind::Own => {
                    Own::decode_body_with_value_kind(decoder, value_kind).map(Self::Own)
                }
                ScryptoCustomValueKind::Decimal => {
                    Decimal::decode_body_with_value_kind(decoder, value_kind).map(Self::Decimal)
                }
                ScryptoCustomValueKind::PreciseDecimal => {
                    PreciseDecimal::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::PreciseDecimal)
                }
                ScryptoCustomValueKind::NonFungibleLocalId => {
                    NonFungibleLocalId::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::NonFungibleLocalId)
                }
            },
            _ => Err(DecodeError::UnexpectedCustomValueKind {
                actual: value_kind.as_u8(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NodeId;

    #[test]
    fn test_custom_types_group1() {
        let values = (Reference(NodeId([1u8; 27])),);
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![
                92, // prefix
                33, // tuple
                1,  // length
                128, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, // reference
            ]
        );
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&bytes).unwrap(),
            ScryptoValue::Tuple {
                fields: vec![ScryptoValue::Custom {
                    value: ScryptoCustomValue::Reference(Reference(NodeId([1u8; 27]))),
                },]
            }
        );
    }

    #[test]
    fn test_custom_types_group2() {
        let values = (Own(NodeId([1u8; NodeId::LENGTH])),);
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![
                92, // prefix
                33, // tuple
                1,  // length
                144, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, // own
            ]
        );
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&bytes).unwrap(),
            ScryptoValue::Tuple {
                fields: vec![ScryptoValue::Custom {
                    value: ScryptoCustomValue::Own(Own(NodeId([1u8; NodeId::LENGTH]))),
                },]
            }
        );
    }

    #[test]
    fn test_custom_types_group4() {
        let values = (
            Decimal::ONE,
            PreciseDecimal::ONE,
            NonFungibleLocalId::integer(1),
            NonFungibleLocalId::bytes(vec![2, 3]).unwrap(),
        );
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![
                92, // prefix
                33, // tuple
                4,  // length
                160, 0, 0, 100, 167, 179, 182, 224, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // decimal
                176, 0, 0, 0, 0, 0, 0, 0, 0, 1, 31, 106, 191, 100, 237, 56, 110, 237, 151, 167,
                218, 244, 249, 63, 233, 3, 79, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, // precise decimal
                192, 1, 0, 0, 0, 0, 0, 0, 0, 1, // non-fungible local id
                192, 2, 2, 2, 3 // non-fungible local id
            ]
        );
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&bytes).unwrap(),
            ScryptoValue::Tuple {
                fields: vec![
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Decimal(Decimal::ONE),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::PreciseDecimal(PreciseDecimal::ONE),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::NonFungibleLocalId(NonFungibleLocalId::integer(
                            1
                        )),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::NonFungibleLocalId(
                            NonFungibleLocalId::bytes(vec![2, 3]).unwrap()
                        ),
                    },
                ]
            }
        );
    }
}
