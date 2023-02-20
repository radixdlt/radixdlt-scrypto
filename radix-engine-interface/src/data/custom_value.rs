use sbor::value_kind::*;
use sbor::*;

use crate::blueprints::resource::*;
use crate::data::model::*;
use crate::data::*;
use crate::math::{Decimal, PreciseDecimal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomValue {
    Address(Address),
    Own(Own),
    Decimal(Decimal),
    PreciseDecimal(PreciseDecimal),
    NonFungibleLocalId(NonFungibleLocalId),
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for ScryptoCustomValue {
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            ScryptoCustomValue::Address(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::Address))
            }
            ScryptoCustomValue::Own(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::Own))
            }
            ScryptoCustomValue::Decimal(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::Decimal))
            }
            ScryptoCustomValue::PreciseDecimal(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::PreciseDecimal))
            }
            ScryptoCustomValue::NonFungibleLocalId(_) => encoder.write_value_kind(
                ValueKind::Custom(ScryptoCustomValueKind::NonFungibleLocalId),
            ),
        }
    }

    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            // TODO: vector free
            ScryptoCustomValue::Address(v) => v.encode_body(encoder),
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
                ScryptoCustomValueKind::Address => {
                    Address::decode_body_with_value_kind(decoder, value_kind).map(Self::Address)
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
    use crate::api::types::*;

    #[test]
    fn test_custom_types_group1() {
        let values = (
            Address::Package(PackageAddress::Normal([1u8; 26])),
            Address::Component(ComponentAddress::Normal([2u8; 26])),
            Address::Resource(ResourceAddress::Normal([3u8; 26])),
            Address::Component(ComponentAddress::EpochManager([4u8; 26])),
        );
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![
                92, // prefix
                33, // tuple
                4,  // length
                128, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, // address
                128, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, // address
                128, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                3, // address
                128, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4 // address
            ]
        );
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&bytes).unwrap(),
            ScryptoValue::Tuple {
                fields: vec![
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Address(Address::Package(
                            PackageAddress::Normal([1u8; 26])
                        )),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Address(Address::Component(
                            ComponentAddress::Normal([2u8; 26])
                        )),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Address(Address::Resource(
                            ResourceAddress::Normal([3u8; 26])
                        )),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Address(Address::Component(
                            ComponentAddress::EpochManager([4u8; 26])
                        )),
                    },
                ]
            }
        );
    }

    #[test]
    fn test_custom_types_group2() {
        let values = (
            Own::Bucket([1u8; 36]),
            Own::Proof([2u8; 36]),
            Own::Vault([3u8; 36]),
            Own::Component([4u8; 36]),
            Own::KeyValueStore([5u8; 36]),
        );
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![
                92, // prefix
                33, // tuple
                5,  // length
                129, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // own
                129, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // own
                129, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // own
                129, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, // own
                129, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5 // own
            ]
        );
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&bytes).unwrap(),
            ScryptoValue::Tuple {
                fields: vec![
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Own(Own::Bucket([1u8; 36])),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Own(Own::Proof([2u8; 36])),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Own(Own::Vault([3u8; 36])),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Own(Own::Component([4u8; 36])),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Own(Own::KeyValueStore([5u8; 36])),
                    },
                ]
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
                133, 0, 0, 100, 167, 179, 182, 224, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // decimal
                134, 0, 0, 0, 0, 0, 0, 0, 0, 1, 31, 106, 191, 100, 237, 56, 110, 237, 151, 167,
                218, 244, 249, 63, 233, 3, 79, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, // precise decimal
                135, 1, 0, 0, 0, 0, 0, 0, 0, 1, // non-fungible local id
                135, 2, 2, 2, 3 // non-fungible local id
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
