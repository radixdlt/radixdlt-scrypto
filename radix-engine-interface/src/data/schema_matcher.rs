use crate::data::*;
use sbor::*;
use scrypto_abi::{Fields, Type};

use super::types::Own;

pub fn sbor_type_id(ty: &Type) -> Option<ScryptoSborTypeId> {
    match ty {
        Type::Unit => Some(SborTypeId::Unit),
        Type::Bool => Some(SborTypeId::Bool),
        Type::I8 => Some(SborTypeId::I8),
        Type::I16 => Some(SborTypeId::I16),
        Type::I32 => Some(SborTypeId::I32),
        Type::I64 => Some(SborTypeId::I64),
        Type::I128 => Some(SborTypeId::I128),
        Type::U8 => Some(SborTypeId::U8),
        Type::U16 => Some(SborTypeId::U16),
        Type::U32 => Some(SborTypeId::U32),
        Type::U64 => Some(SborTypeId::U64),
        Type::U128 => Some(SborTypeId::U128),
        Type::String => Some(SborTypeId::String),

        Type::Array { .. } => Some(SborTypeId::Array),
        Type::Vec { .. } => Some(SborTypeId::Array),
        Type::HashSet { .. } => Some(SborTypeId::Array),
        Type::TreeSet { .. } => Some(SborTypeId::Array),
        Type::HashMap { .. } => Some(SborTypeId::Array),
        Type::TreeMap { .. } => Some(SborTypeId::Array),

        Type::Tuple { .. } => Some(SborTypeId::Tuple),
        Type::Struct { .. } => Some(SborTypeId::Tuple),
        Type::NonFungibleAddress { .. } => Some(SborTypeId::Tuple),

        Type::Enum { .. } => Some(SborTypeId::Enum),
        Type::Option { .. } => Some(SborTypeId::Enum),
        Type::Result { .. } => Some(SborTypeId::Enum),

        Type::PackageAddress => Some(SborTypeId::Custom(ScryptoCustomTypeId::PackageAddress)),
        Type::ComponentAddress => Some(SborTypeId::Custom(ScryptoCustomTypeId::ComponentAddress)),
        Type::ResourceAddress => Some(SborTypeId::Custom(ScryptoCustomTypeId::ResourceAddress)),
        Type::SystemAddress => Some(SborTypeId::Custom(ScryptoCustomTypeId::SystemAddress)),

        Type::Own
        | Type::Bucket
        | Type::Proof
        | Type::Vault
        | Type::Component
        | Type::KeyValueStore { .. } => Some(SborTypeId::Custom(ScryptoCustomTypeId::Own)),
        Type::Blob => Some(SborTypeId::Custom(ScryptoCustomTypeId::Blob)),

        Type::ManifestBucket => Some(SborTypeId::Custom(ScryptoCustomTypeId::Bucket)),
        Type::ManifestProof => Some(SborTypeId::Custom(ScryptoCustomTypeId::Proof)),
        Type::ManifestExpression => Some(SborTypeId::Custom(ScryptoCustomTypeId::Expression)),

        Type::Hash => Some(SborTypeId::Custom(ScryptoCustomTypeId::Hash)),
        Type::EcdsaSecp256k1PublicKey => Some(SborTypeId::Custom(
            ScryptoCustomTypeId::EcdsaSecp256k1PublicKey,
        )),
        Type::EcdsaSecp256k1Signature => Some(SborTypeId::Custom(
            ScryptoCustomTypeId::EcdsaSecp256k1Signature,
        )),
        Type::EddsaEd25519PublicKey => Some(SborTypeId::Custom(
            ScryptoCustomTypeId::EddsaEd25519PublicKey,
        )),
        Type::EddsaEd25519Signature => Some(SborTypeId::Custom(
            ScryptoCustomTypeId::EddsaEd25519Signature,
        )),
        Type::Decimal => Some(SborTypeId::Custom(ScryptoCustomTypeId::Decimal)),
        Type::PreciseDecimal => Some(SborTypeId::Custom(ScryptoCustomTypeId::PreciseDecimal)),
        Type::NonFungibleId => Some(SborTypeId::Custom(ScryptoCustomTypeId::NonFungibleId)),

        Type::Any => None,
    }
}

pub fn match_schema_with_value(ty: &Type, value: &ScryptoValue) -> bool {
    match ty {
        Type::Unit => matches!(value, SborValue::Unit),
        Type::Bool => matches!(value, SborValue::Bool { .. }),
        Type::I8 => matches!(value, SborValue::I8 { .. }),
        Type::I16 => matches!(value, SborValue::I16 { .. }),
        Type::I32 => matches!(value, SborValue::I32 { .. }),
        Type::I64 => matches!(value, SborValue::I64 { .. }),
        Type::I128 => matches!(value, SborValue::I128 { .. }),
        Type::U8 => matches!(value, SborValue::U8 { .. }),
        Type::U16 => matches!(value, SborValue::U16 { .. }),
        Type::U32 => matches!(value, SborValue::U32 { .. }),
        Type::U64 => matches!(value, SborValue::U64 { .. }),
        Type::U128 => matches!(value, SborValue::U128 { .. }),
        Type::String => matches!(value, SborValue::String { .. }),

        // array
        Type::Array {
            element_type,
            length,
        } => {
            if let SborValue::Array {
                element_type_id,
                elements,
            } = value
            {
                let element_type_matches = match sbor_type_id(element_type) {
                    Some(id) => id == *element_type_id,
                    None => true,
                };
                element_type_matches
                    && usize::from(*length) == elements.len()
                    && elements
                        .iter()
                        .all(|v| match_schema_with_value(element_type, v))
            } else {
                false
            }
        }
        Type::Vec { element_type }
        | Type::HashSet { element_type }
        | Type::TreeSet { element_type } => {
            if let SborValue::Array {
                element_type_id,
                elements,
            } = value
            {
                let element_type_matches = match sbor_type_id(element_type) {
                    Some(id) => id == *element_type_id,
                    None => true,
                };
                element_type_matches
                    && elements
                        .iter()
                        .all(|v| match_schema_with_value(element_type, v))
            } else {
                false
            }
        }
        Type::TreeMap {
            key_type,
            value_type,
        }
        | Type::HashMap {
            key_type,
            value_type,
        } => {
            if let SborValue::Array {
                element_type_id,
                elements,
            } = value
            {
                *element_type_id == SborTypeId::Tuple
                    && elements.iter().all(|e| {
                        if let SborValue::Tuple { fields } = e {
                            fields.len() == 2
                                && match_schema_with_value(key_type, &fields[0])
                                && match_schema_with_value(value_type, &fields[1])
                        } else {
                            false
                        }
                    })
            } else {
                false
            }
        }

        // tuple
        Type::Tuple { element_types } => {
            if let SborValue::Tuple { fields } = value {
                element_types.len() == fields.len()
                    && element_types
                        .iter()
                        .enumerate()
                        .all(|(i, e)| match_schema_with_value(e, fields.get(i).unwrap()))
            } else {
                false
            }
        }
        Type::Struct {
            name: _,
            fields: type_fields,
        } => {
            if let SborValue::Tuple { fields } = value {
                match type_fields {
                    Fields::Unit => fields.is_empty(),
                    Fields::Unnamed { unnamed } => {
                        unnamed.len() == fields.len()
                            && unnamed
                                .iter()
                                .enumerate()
                                .all(|(i, e)| match_schema_with_value(e, fields.get(i).unwrap()))
                    }
                    Fields::Named { named } => {
                        named.len() == fields.len()
                            && named.iter().enumerate().all(|(i, (_, e))| {
                                match_schema_with_value(e, fields.get(i).unwrap())
                            })
                    }
                }
            } else {
                false
            }
        }
        Type::NonFungibleAddress => {
            if let SborValue::Tuple { fields } = value {
                fields.len() == 2
                    && match_schema_with_value(&Type::ResourceAddress, fields.get(0).unwrap())
                    && match_schema_with_value(&Type::NonFungibleId, fields.get(1).unwrap())
            } else {
                false
            }
        }

        // enum
        Type::Enum {
            name: _,
            variants: type_variants,
        } => {
            if let SborValue::Enum {
                discriminator,
                fields,
            } = value
            {
                for variant in type_variants {
                    if variant.name.eq(discriminator) {
                        return match &variant.fields {
                            Fields::Unit => fields.is_empty(),
                            Fields::Unnamed { unnamed } => {
                                unnamed.len() == fields.len()
                                    && unnamed.iter().enumerate().all(|(i, e)| {
                                        match_schema_with_value(e, fields.get(i).unwrap())
                                    })
                            }
                            Fields::Named { named } => {
                                named.len() == fields.len()
                                    && named.iter().enumerate().all(|(i, (_, e))| {
                                        match_schema_with_value(e, fields.get(i).unwrap())
                                    })
                            }
                        };
                    }
                }
                false
            } else {
                false
            }
        }
        Type::Option { some_type } => {
            if let SborValue::Enum {
                discriminator,
                fields,
            } = value
            {
                match discriminator.as_str() {
                    OPTION_VARIANT_SOME => {
                        fields.len() == 1 && match_schema_with_value(some_type, &fields[0])
                    }
                    OPTION_VARIANT_NONE => fields.len() == 0,
                    _ => false,
                }
            } else {
                false
            }
        }
        Type::Result {
            okay_type,
            err_type,
        } => {
            if let SborValue::Enum {
                discriminator,
                fields,
            } = value
            {
                match discriminator.as_str() {
                    RESULT_VARIANT_OK => {
                        fields.len() == 1 && match_schema_with_value(okay_type, &fields[0])
                    }
                    RESULT_VARIANT_ERR => {
                        fields.len() == 1 && match_schema_with_value(err_type, &fields[0])
                    }
                    _ => false,
                }
            } else {
                false
            }
        }

        // custom
        Type::PackageAddress => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::PackageAddress(_))
            } else {
                false
            }
        }
        Type::ComponentAddress => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::ComponentAddress(_))
            } else {
                false
            }
        }
        Type::ResourceAddress => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::ResourceAddress(_))
            } else {
                false
            }
        }
        Type::SystemAddress => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::SystemAddress(_))
            } else {
                false
            }
        }

        Type::Own => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(_))
            } else {
                false
            }
        }
        Type::Bucket => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(Own::Bucket(_)))
            } else {
                false
            }
        }
        Type::Proof => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(Own::Proof(_)))
            } else {
                false
            }
        }
        Type::Vault => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(Own::Vault(_)))
            } else {
                false
            }
        }
        Type::Component => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(Own::Component(_)))
            } else {
                false
            }
        }
        Type::KeyValueStore { .. } => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(Own::KeyValueStore(_)))
            } else {
                false
            }
        }
        Type::Blob => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Blob(_))
            } else {
                false
            }
        }

        Type::ManifestBucket => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Bucket(_))
            } else {
                false
            }
        }
        Type::ManifestProof => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Proof(_))
            } else {
                false
            }
        }
        Type::ManifestExpression => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Expression(_))
            } else {
                false
            }
        }

        Type::Hash => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Hash(_))
            } else {
                false
            }
        }
        Type::EcdsaSecp256k1PublicKey => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::EcdsaSecp256k1PublicKey(_))
            } else {
                false
            }
        }
        Type::EcdsaSecp256k1Signature => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::EcdsaSecp256k1Signature(_))
            } else {
                false
            }
        }
        Type::EddsaEd25519PublicKey => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::EddsaEd25519PublicKey(_))
            } else {
                false
            }
        }
        Type::EddsaEd25519Signature => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::EddsaEd25519Signature(_))
            } else {
                false
            }
        }
        Type::Decimal => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Decimal(_))
            } else {
                false
            }
        }
        Type::PreciseDecimal => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::PreciseDecimal(_))
            } else {
                false
            }
        }
        Type::NonFungibleId => {
            if let SborValue::Custom { value } = value {
                matches!(value, ScryptoCustomValue::NonFungibleId(_))
            } else {
                false
            }
        }

        Type::Any => true,
    }
}
