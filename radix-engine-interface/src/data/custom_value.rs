use sbor::value_kind::*;
use sbor::*;

use crate::api::types::*;
use crate::crypto::*;
use crate::data::types::*;
use crate::data::*;
use crate::math::{Decimal, PreciseDecimal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomValue {
    // RE interpreted types
    PackageAddress(PackageAddress),
    ComponentAddress(ComponentAddress),
    ResourceAddress(ResourceAddress),
    Own(Own),

    // TX interpreted types
    Bucket(ManifestBucket),
    Proof(ManifestProof),
    Expression(ManifestExpression),
    Blob(ManifestBlobRef),

    // Uninterpreted
    Hash(Hash),
    EcdsaSecp256k1PublicKey(EcdsaSecp256k1PublicKey),
    EcdsaSecp256k1Signature(EcdsaSecp256k1Signature),
    EddsaEd25519PublicKey(EddsaEd25519PublicKey),
    EddsaEd25519Signature(EddsaEd25519Signature),
    Decimal(Decimal),
    PreciseDecimal(PreciseDecimal),
    NonFungibleLocalId(NonFungibleLocalId),
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for ScryptoCustomValue {
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            ScryptoCustomValue::PackageAddress(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::PackageAddress))
            }
            ScryptoCustomValue::ComponentAddress(_) => encoder
                .write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::ComponentAddress)),
            ScryptoCustomValue::ResourceAddress(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::ResourceAddress))
            }
            ScryptoCustomValue::Own(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::Own))
            }
            ScryptoCustomValue::Bucket(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::Bucket))
            }
            ScryptoCustomValue::Proof(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::Proof))
            }
            ScryptoCustomValue::Expression(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::Expression))
            }
            ScryptoCustomValue::Blob(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::Blob))
            }
            ScryptoCustomValue::Hash(_) => {
                encoder.write_value_kind(ValueKind::Custom(ScryptoCustomValueKind::Hash))
            }
            ScryptoCustomValue::EcdsaSecp256k1PublicKey(_) => encoder.write_value_kind(
                ValueKind::Custom(ScryptoCustomValueKind::EcdsaSecp256k1PublicKey),
            ),
            ScryptoCustomValue::EcdsaSecp256k1Signature(_) => encoder.write_value_kind(
                ValueKind::Custom(ScryptoCustomValueKind::EcdsaSecp256k1Signature),
            ),
            ScryptoCustomValue::EddsaEd25519PublicKey(_) => encoder.write_value_kind(
                ValueKind::Custom(ScryptoCustomValueKind::EddsaEd25519PublicKey),
            ),
            ScryptoCustomValue::EddsaEd25519Signature(_) => encoder.write_value_kind(
                ValueKind::Custom(ScryptoCustomValueKind::EddsaEd25519Signature),
            ),
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
            ScryptoCustomValue::PackageAddress(v) => v.encode_body(encoder),
            ScryptoCustomValue::ComponentAddress(v) => v.encode_body(encoder),
            ScryptoCustomValue::ResourceAddress(v) => v.encode_body(encoder),
            ScryptoCustomValue::Own(v) => v.encode_body(encoder),
            ScryptoCustomValue::Bucket(v) => v.encode_body(encoder),
            ScryptoCustomValue::Proof(v) => v.encode_body(encoder),
            ScryptoCustomValue::Expression(v) => v.encode_body(encoder),
            ScryptoCustomValue::Blob(v) => v.encode_body(encoder),
            ScryptoCustomValue::Hash(v) => v.encode_body(encoder),
            ScryptoCustomValue::EcdsaSecp256k1PublicKey(v) => v.encode_body(encoder),
            ScryptoCustomValue::EcdsaSecp256k1Signature(v) => v.encode_body(encoder),
            ScryptoCustomValue::EddsaEd25519PublicKey(v) => v.encode_body(encoder),
            ScryptoCustomValue::EddsaEd25519Signature(v) => v.encode_body(encoder),
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
                ScryptoCustomValueKind::PackageAddress => {
                    PackageAddress::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::PackageAddress)
                }
                ScryptoCustomValueKind::ComponentAddress => {
                    ComponentAddress::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::ComponentAddress)
                }
                ScryptoCustomValueKind::ResourceAddress => {
                    ResourceAddress::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::ResourceAddress)
                }
                ScryptoCustomValueKind::Own => {
                    Own::decode_body_with_value_kind(decoder, value_kind).map(Self::Own)
                }
                ScryptoCustomValueKind::Blob => {
                    ManifestBlobRef::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::Blob)
                }
                ScryptoCustomValueKind::Bucket => {
                    ManifestBucket::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::Bucket)
                }
                ScryptoCustomValueKind::Proof => {
                    ManifestProof::decode_body_with_value_kind(decoder, value_kind).map(Self::Proof)
                }
                ScryptoCustomValueKind::Expression => {
                    ManifestExpression::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::Expression)
                }
                ScryptoCustomValueKind::Hash => {
                    Hash::decode_body_with_value_kind(decoder, value_kind).map(Self::Hash)
                }
                ScryptoCustomValueKind::EcdsaSecp256k1PublicKey => {
                    EcdsaSecp256k1PublicKey::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::EcdsaSecp256k1PublicKey)
                }
                ScryptoCustomValueKind::EcdsaSecp256k1Signature => {
                    EcdsaSecp256k1Signature::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::EcdsaSecp256k1Signature)
                }
                ScryptoCustomValueKind::EddsaEd25519PublicKey => {
                    EddsaEd25519PublicKey::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::EddsaEd25519PublicKey)
                }
                ScryptoCustomValueKind::EddsaEd25519Signature => {
                    EddsaEd25519Signature::decode_body_with_value_kind(decoder, value_kind)
                        .map(Self::EddsaEd25519Signature)
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

    #[test]
    fn test_custom_types_group1() {
        let values = (
            PackageAddress::Normal([1u8; 26]),
            ComponentAddress::Normal([2u8; 26]),
            ResourceAddress::Normal([3u8; 26]),
            ComponentAddress::EpochManager([4u8; 26]),
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
                        value: ScryptoCustomValue::PackageAddress(PackageAddress::Normal(
                            [1u8; 26]
                        )),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::ComponentAddress(ComponentAddress::Normal(
                            [2u8; 26]
                        )),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::ResourceAddress(ResourceAddress::Normal(
                            [3u8; 26]
                        )),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::ComponentAddress(
                            ComponentAddress::EpochManager([4u8; 26])
                        ),
                    },
                ]
            }
        );
    }

    #[test]
    fn test_custom_types_group2() {
        let values = (
            Own::Bucket(1),
            Own::Proof(2),
            Own::Vault([3u8; 36]),
            Own::Component([4u8; 36]),
            Own::KeyValueStore([5u8; 36]),
        );
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![
                92, 33, 5, 144, 0, 1, 0, 0, 0, 144, 1, 2, 0, 0, 0, 144, 2, 3, 3, 3, 3, 3, 3, 3, 3,
                3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                144, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 144, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            ]
        );
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&bytes).unwrap(),
            ScryptoValue::Tuple {
                fields: vec![
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Own(Own::Bucket(1)),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Own(Own::Proof(2)),
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
    fn test_custom_types_group3() {
        let values = (
            ManifestBucket(1u32),
            ManifestProof(2u32),
            ManifestExpression::EntireWorktop,
            ManifestBlobRef(Hash([3u8; 32])),
        );
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![
                92, 33, 4, 160, 1, 0, 0, 0, 161, 2, 0, 0, 0, 162, 0, 163, 3, 3, 3, 3, 3, 3, 3, 3,
                3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3
            ]
        );
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&bytes).unwrap(),
            ScryptoValue::Tuple {
                fields: vec![
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Bucket(ManifestBucket(1u32)),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Proof(ManifestProof(2u32)),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Expression(ManifestExpression::EntireWorktop),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Blob(ManifestBlobRef(Hash([3u8; 32]))),
                    },
                ]
            }
        );
    }

    #[test]
    fn test_custom_types_group4() {
        let values = (
            Hash([0u8; 32]),
            EcdsaSecp256k1PublicKey([1u8; 33]),
            EcdsaSecp256k1Signature([2u8; 65]),
            EddsaEd25519PublicKey([3u8; 32]),
            EddsaEd25519Signature([4u8; 64]),
            Decimal::ONE,
            PreciseDecimal::ONE,
            NonFungibleLocalId::integer(1),
            NonFungibleLocalId::bytes(vec![2, 3]).unwrap(),
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
                        value: ScryptoCustomValue::Hash(Hash([0u8; 32])),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::EcdsaSecp256k1PublicKey(
                            EcdsaSecp256k1PublicKey([1u8; 33],)
                        ),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::EcdsaSecp256k1Signature(
                            EcdsaSecp256k1Signature([2u8; 65],)
                        ),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::EddsaEd25519PublicKey(EddsaEd25519PublicKey(
                            [3u8; 32]
                        )),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::EddsaEd25519Signature(EddsaEd25519Signature(
                            [4u8; 64]
                        )),
                    },
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
