use sbor::type_id::*;
use sbor::*;

use crate::api::types::*;
use crate::crypto::*;
use crate::data::types::*;
use crate::data::*;
use crate::math::{Decimal, PreciseDecimal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomValue {
    // RE global address types
    PackageAddress(PackageAddress),
    ComponentAddress(ComponentAddress),
    ResourceAddress(ResourceAddress),
    SystemAddress(SystemAddress),

    // RE interpreted types
    Own(Own),
    NonFungibleAddress(NonFungibleAddress),
    Blob(Blob),

    // TX interpreted types (TODO: rename?)
    Bucket(ManifestBucket),
    Proof(ManifestProof),
    Expression(ManifestExpression),

    // Uninterpreted
    Hash(Hash),
    EcdsaSecp256k1PublicKey(EcdsaSecp256k1PublicKey),
    EcdsaSecp256k1Signature(EcdsaSecp256k1Signature),
    EddsaEd25519PublicKey(EddsaEd25519PublicKey),
    EddsaEd25519Signature(EddsaEd25519Signature),
    Decimal(Decimal),
    PreciseDecimal(PreciseDecimal),
    NonFungibleId(NonFungibleId),
}

impl<E: Encoder<ScryptoCustomTypeId>> Encode<ScryptoCustomTypeId, E> for ScryptoCustomValue {
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            ScryptoCustomValue::PackageAddress(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::PackageAddress))
            }
            ScryptoCustomValue::ComponentAddress(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::ComponentAddress))
            }
            ScryptoCustomValue::ResourceAddress(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::ResourceAddress))
            }
            ScryptoCustomValue::SystemAddress(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::SystemAddress))
            }
            ScryptoCustomValue::Own(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Own))
            }
            ScryptoCustomValue::Bucket(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Bucket))
            }
            ScryptoCustomValue::Proof(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Proof))
            }
            ScryptoCustomValue::Expression(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Expression))
            }
            ScryptoCustomValue::Blob(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Blob))
            }
            ScryptoCustomValue::NonFungibleAddress(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::NonFungibleAddress))
            }
            ScryptoCustomValue::Hash(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Hash))
            }
            ScryptoCustomValue::EcdsaSecp256k1PublicKey(_) => encoder.write_type_id(
                SborTypeId::Custom(ScryptoCustomTypeId::EcdsaSecp256k1PublicKey),
            ),
            ScryptoCustomValue::EcdsaSecp256k1Signature(_) => encoder.write_type_id(
                SborTypeId::Custom(ScryptoCustomTypeId::EcdsaSecp256k1Signature),
            ),
            ScryptoCustomValue::EddsaEd25519PublicKey(_) => encoder.write_type_id(
                SborTypeId::Custom(ScryptoCustomTypeId::EddsaEd25519PublicKey),
            ),
            ScryptoCustomValue::EddsaEd25519Signature(_) => encoder.write_type_id(
                SborTypeId::Custom(ScryptoCustomTypeId::EddsaEd25519Signature),
            ),
            ScryptoCustomValue::Decimal(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Decimal))
            }
            ScryptoCustomValue::PreciseDecimal(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::PreciseDecimal))
            }
            ScryptoCustomValue::NonFungibleId(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::NonFungibleId))
            }
        }
    }

    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            // TODO: vector free
            ScryptoCustomValue::PackageAddress(v) => v.encode_body(encoder),
            ScryptoCustomValue::ComponentAddress(v) => v.encode_body(encoder),
            ScryptoCustomValue::ResourceAddress(v) => v.encode_body(encoder),
            ScryptoCustomValue::SystemAddress(v) => v.encode_body(encoder),
            ScryptoCustomValue::Own(v) => v.encode_body(encoder),
            ScryptoCustomValue::Bucket(v) => v.encode_body(encoder),
            ScryptoCustomValue::Proof(v) => v.encode_body(encoder),
            ScryptoCustomValue::Expression(v) => v.encode_body(encoder),
            ScryptoCustomValue::Blob(v) => v.encode_body(encoder),
            ScryptoCustomValue::NonFungibleAddress(v) => v.encode_body(encoder),
            ScryptoCustomValue::Hash(v) => v.encode_body(encoder),
            ScryptoCustomValue::EcdsaSecp256k1PublicKey(v) => v.encode_body(encoder),
            ScryptoCustomValue::EcdsaSecp256k1Signature(v) => v.encode_body(encoder),
            ScryptoCustomValue::EddsaEd25519PublicKey(v) => v.encode_body(encoder),
            ScryptoCustomValue::EddsaEd25519Signature(v) => v.encode_body(encoder),
            ScryptoCustomValue::Decimal(v) => v.encode_body(encoder),
            ScryptoCustomValue::PreciseDecimal(v) => v.encode_body(encoder),
            ScryptoCustomValue::NonFungibleId(v) => v.encode_body(encoder),
        }
    }
}

impl<D: Decoder<ScryptoCustomTypeId>> Decode<ScryptoCustomTypeId, D> for ScryptoCustomValue {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<ScryptoCustomTypeId>,
    ) -> Result<Self, DecodeError> {
        match type_id {
            SborTypeId::Custom(cti) => match cti {
                ScryptoCustomTypeId::PackageAddress => {
                    PackageAddress::decode_body_with_type_id(decoder, type_id)
                        .map(Self::PackageAddress)
                }
                ScryptoCustomTypeId::ComponentAddress => {
                    ComponentAddress::decode_body_with_type_id(decoder, type_id)
                        .map(Self::ComponentAddress)
                }
                ScryptoCustomTypeId::ResourceAddress => {
                    ResourceAddress::decode_body_with_type_id(decoder, type_id)
                        .map(Self::ResourceAddress)
                }
                ScryptoCustomTypeId::SystemAddress => {
                    SystemAddress::decode_body_with_type_id(decoder, type_id)
                        .map(Self::SystemAddress)
                }
                ScryptoCustomTypeId::Own => {
                    Own::decode_body_with_type_id(decoder, type_id).map(Self::Own)
                }
                ScryptoCustomTypeId::NonFungibleAddress => {
                    NonFungibleAddress::decode_body_with_type_id(decoder, type_id)
                        .map(Self::NonFungibleAddress)
                }
                ScryptoCustomTypeId::Blob => {
                    Blob::decode_body_with_type_id(decoder, type_id).map(Self::Blob)
                }
                ScryptoCustomTypeId::Bucket => {
                    ManifestBucket::decode_body_with_type_id(decoder, type_id).map(Self::Bucket)
                }
                ScryptoCustomTypeId::Proof => {
                    ManifestProof::decode_body_with_type_id(decoder, type_id).map(Self::Proof)
                }
                ScryptoCustomTypeId::Expression => {
                    ManifestExpression::decode_body_with_type_id(decoder, type_id)
                        .map(Self::Expression)
                }
                ScryptoCustomTypeId::Hash => {
                    Hash::decode_body_with_type_id(decoder, type_id).map(Self::Hash)
                }
                ScryptoCustomTypeId::EcdsaSecp256k1PublicKey => {
                    EcdsaSecp256k1PublicKey::decode_body_with_type_id(decoder, type_id)
                        .map(Self::EcdsaSecp256k1PublicKey)
                }
                ScryptoCustomTypeId::EcdsaSecp256k1Signature => {
                    EcdsaSecp256k1Signature::decode_body_with_type_id(decoder, type_id)
                        .map(Self::EcdsaSecp256k1Signature)
                }
                ScryptoCustomTypeId::EddsaEd25519PublicKey => {
                    EddsaEd25519PublicKey::decode_body_with_type_id(decoder, type_id)
                        .map(Self::EddsaEd25519PublicKey)
                }
                ScryptoCustomTypeId::EddsaEd25519Signature => {
                    EddsaEd25519Signature::decode_body_with_type_id(decoder, type_id)
                        .map(Self::EddsaEd25519Signature)
                }
                ScryptoCustomTypeId::Decimal => {
                    Decimal::decode_body_with_type_id(decoder, type_id).map(Self::Decimal)
                }
                ScryptoCustomTypeId::PreciseDecimal => {
                    PreciseDecimal::decode_body_with_type_id(decoder, type_id)
                        .map(Self::PreciseDecimal)
                }
                ScryptoCustomTypeId::NonFungibleId => {
                    NonFungibleId::decode_body_with_type_id(decoder, type_id)
                        .map(Self::NonFungibleId)
                }
            },
            _ => Err(DecodeError::UnexpectedCustomTypeId {
                actual: type_id.as_u8(),
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
            SystemAddress::EpochManager([4u8; 26]),
        );
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![
                92, 33, 4, 128, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 129, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 2, 2, 130, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                3, 3, 3, 3, 3, 3, 3, 131, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
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
                        value: ScryptoCustomValue::SystemAddress(SystemAddress::EpochManager(
                            [4u8; 26]
                        )),
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
            Blob(Hash([8u8; 32])),
        );
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![
                92, 33, 7, 148, 0, 1, 0, 0, 0, 148, 1, 2, 0, 0, 0, 148, 2, 3, 3, 3, 3, 3, 3, 3, 3,
                3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                144, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 145, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 162, 0, 6, 6, 6, 6, 6,
                6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0, 7, 0, 0, 0, 161,
                8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
                8, 8, 8, 8
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
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::NonFungibleAddress(NonFungibleAddress {
                            resource_address: ResourceAddress::Normal([6u8; 26]),
                            non_fungible_id: NonFungibleId::U32(7),
                        }),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::Blob(Blob(Hash([8u8; 32]))),
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
        );
        let bytes = scrypto_encode(&values).unwrap();
        assert_eq!(
            bytes,
            vec![92, 33, 3, 146, 1, 0, 0, 0, 147, 2, 0, 0, 0, 160, 0]
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
            NonFungibleId::U32(1),
            NonFungibleId::Bytes(vec![2, 3]),
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
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 183, 0, 1, 0, 0, 0, 183, 3,
                2, 2, 3
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
                        value: ScryptoCustomValue::NonFungibleId(NonFungibleId::U32(1)),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::NonFungibleId(NonFungibleId::Bytes(vec![2, 3])),
                    },
                ]
            }
        );
    }
}
