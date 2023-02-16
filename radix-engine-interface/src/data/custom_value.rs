use sbor::value_kind::*;
use sbor::*;

use crate::blueprints::resource::*;
use crate::crypto::*;
use crate::data::model::*;
use crate::data::*;
use crate::math::{Decimal, PreciseDecimal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomValue {
    // RE interpreted types
    Address(Address),
    Own(Own),

    // Uninterpreted
    Decimal(Decimal),
    PreciseDecimal(PreciseDecimal),
    NonFungibleLocalId(NonFungibleLocalId),
    PublicKey(PublicKey),
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
            ScryptoCustomValue::PublicKey(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::PublicKey))
            }
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
            ScryptoCustomValue::PublicKey(v) => v.encode_body(encoder),
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
                ScryptoCustomValueKind::PublicKey => {
                    PublicKey::decode_body_with_value_kind(decoder, value_kind).map(Self::PublicKey)
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
            Address::ResourceManager(ResourceAddress::Normal([3u8; 26])),
            Address::Component(ComponentAddress::EpochManager([4u8; 26])),
        );
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![
                92, 33, 4, 128, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 129, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 2, 2, 130, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                3, 3, 3, 3, 3, 3, 3, 129, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4, 4, 4, 4, 4, 4, 4
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
                        value: ScryptoCustomValue::Address(Address::ResourceManager(
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
                92, 33, 5, 144, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 144, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 144,
                2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                3, 3, 3, 3, 3, 3, 3, 3, 3, 144, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 144, 4, 5, 5, 5, 5, 5,
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
                5, 5, 5
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
            EcdsaSecp256k1PublicKey([1u8; 33]),
            EddsaEd25519PublicKey([3u8; 32]),
        );
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![
                92, 33, 9, 176, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 177, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 178, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                179, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                3, 3, 3, 3, 3, 3, 180, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 181, 0, 0, 100, 167, 179, 182, 224,
                13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 182, 0,
                0, 0, 0, 0, 0, 0, 0, 1, 31, 106, 191, 100, 237, 56, 110, 237, 151, 167, 218, 244,
                249, 63, 233, 3, 79, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 183, 1, 0, 0, 0, 0, 0, 0, 0,
                1, 183, 2, 2, 2, 3
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
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::PublicKey(PublicKey::EcdsaSecp256k1(
                            EcdsaSecp256k1PublicKey([1u8; 33],)
                        )),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::PublicKey(PublicKey::EddsaEd25519(
                            EddsaEd25519PublicKey([3u8; 32])
                        )),
                    },
                ]
            }
        );
    }
}
