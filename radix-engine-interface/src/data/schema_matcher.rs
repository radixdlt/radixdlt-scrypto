use crate::data::*;
use sbor::*;
use scrypto_abi::{Fields, Type};

use super::types::Own;

pub fn get_value_kind(ty: &Type) -> Option<ScryptoValueKind> {
    match ty {
        Type::Bool => Some(ValueKind::Bool),
        Type::I8 => Some(ValueKind::I8),
        Type::I16 => Some(ValueKind::I16),
        Type::I32 => Some(ValueKind::I32),
        Type::I64 => Some(ValueKind::I64),
        Type::I128 => Some(ValueKind::I128),
        Type::U8 => Some(ValueKind::U8),
        Type::U16 => Some(ValueKind::U16),
        Type::U32 => Some(ValueKind::U32),
        Type::U64 => Some(ValueKind::U64),
        Type::U128 => Some(ValueKind::U128),
        Type::String => Some(ValueKind::String),

        Type::Array { .. } => Some(ValueKind::Array),
        Type::Vec { .. } => Some(ValueKind::Array),
        Type::HashSet { .. } => Some(ValueKind::Array),
        Type::TreeSet { .. } => Some(ValueKind::Array),
        Type::HashMap { .. } => Some(ValueKind::Array),
        Type::TreeMap { .. } => Some(ValueKind::Array),

        Type::Tuple { .. } => Some(ValueKind::Tuple),
        Type::Struct { .. } => Some(ValueKind::Tuple),
        Type::NonFungibleGlobalId { .. } => Some(ValueKind::Tuple),

        Type::Enum { .. } => Some(ValueKind::Enum),
        Type::Option { .. } => Some(ValueKind::Enum),
        Type::Result { .. } => Some(ValueKind::Enum),

        Type::PackageAddress => Some(ValueKind::Custom(ScryptoCustomValueKind::PackageAddress)),
        Type::ComponentAddress => Some(ValueKind::Custom(ScryptoCustomValueKind::ComponentAddress)),
        Type::ResourceAddress => Some(ValueKind::Custom(ScryptoCustomValueKind::ResourceAddress)),

        Type::Own
        | Type::Bucket
        | Type::Proof
        | Type::Vault
        | Type::Component
        | Type::KeyValueStore { .. } => Some(ValueKind::Custom(ScryptoCustomValueKind::Own)),

        Type::Hash => Some(ValueKind::Custom(ScryptoCustomValueKind::Hash)),
        Type::EcdsaSecp256k1PublicKey => Some(ValueKind::Custom(
            ScryptoCustomValueKind::EcdsaSecp256k1PublicKey,
        )),
        Type::EcdsaSecp256k1Signature => Some(ValueKind::Custom(
            ScryptoCustomValueKind::EcdsaSecp256k1Signature,
        )),
        Type::EddsaEd25519PublicKey => Some(ValueKind::Custom(
            ScryptoCustomValueKind::EddsaEd25519PublicKey,
        )),
        Type::EddsaEd25519Signature => Some(ValueKind::Custom(
            ScryptoCustomValueKind::EddsaEd25519Signature,
        )),
        Type::Decimal => Some(ValueKind::Custom(ScryptoCustomValueKind::Decimal)),
        Type::PreciseDecimal => Some(ValueKind::Custom(ScryptoCustomValueKind::PreciseDecimal)),
        Type::NonFungibleLocalId => Some(ValueKind::Custom(
            ScryptoCustomValueKind::NonFungibleLocalId,
        )),

        Type::Any => None,
    }
}

pub fn match_schema_with_value(ty: &Type, value: &ScryptoValue) -> bool {
    match ty {
        Type::Bool => matches!(value, Value::Bool { .. }),
        Type::I8 => matches!(value, Value::I8 { .. }),
        Type::I16 => matches!(value, Value::I16 { .. }),
        Type::I32 => matches!(value, Value::I32 { .. }),
        Type::I64 => matches!(value, Value::I64 { .. }),
        Type::I128 => matches!(value, Value::I128 { .. }),
        Type::U8 => matches!(value, Value::U8 { .. }),
        Type::U16 => matches!(value, Value::U16 { .. }),
        Type::U32 => matches!(value, Value::U32 { .. }),
        Type::U64 => matches!(value, Value::U64 { .. }),
        Type::U128 => matches!(value, Value::U128 { .. }),
        Type::String => matches!(value, Value::String { .. }),

        // array
        Type::Array {
            element_type,
            length,
        } => {
            if let Value::Array {
                element_value_kind,
                elements,
            } = value
            {
                let element_type_matches = match get_value_kind(element_type) {
                    Some(id) => id == *element_value_kind,
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
            if let Value::Array {
                element_value_kind,
                elements,
            } = value
            {
                let element_type_matches = match get_value_kind(element_type) {
                    Some(id) => id == *element_value_kind,
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
            if let Value::Map {
                key_value_kind,
                value_value_kind,
                entries,
            } = value
            {
                let key_type_matches = match get_value_kind(key_type) {
                    Some(id) => id == *key_value_kind,
                    None => true,
                };
                let value_type_matches = match get_value_kind(value_type) {
                    Some(id) => id == *value_value_kind,
                    None => true,
                };
                key_type_matches
                    && value_type_matches
                    && entries.iter().all(|e| {
                        match_schema_with_value(key_type, &e.0)
                            && match_schema_with_value(value_type, &e.1)
                    })
            } else {
                false
            }
        }

        // tuple
        Type::Tuple { element_types } => {
            if let Value::Tuple { fields } = value {
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
            if let Value::Tuple { fields } = value {
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
        Type::NonFungibleGlobalId => {
            if let Value::Tuple { fields } = value {
                fields.len() == 2
                    && match_schema_with_value(&Type::ResourceAddress, fields.get(0).unwrap())
                    && match_schema_with_value(&Type::NonFungibleLocalId, fields.get(1).unwrap())
            } else {
                false
            }
        }

        // enum
        Type::Enum {
            name: _,
            variants: type_variants,
        } => {
            if let Value::Enum {
                discriminator,
                fields,
            } = value
            {
                if let Some(variant) = type_variants.get(*discriminator as usize) {
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
                false
            } else {
                false
            }
        }
        Type::Option { some_type } => {
            if let Value::Enum {
                discriminator,
                fields,
            } = value
            {
                match *discriminator {
                    OPTION_VARIANT_NONE => fields.len() == 0,
                    OPTION_VARIANT_SOME => {
                        fields.len() == 1 && match_schema_with_value(some_type, &fields[0])
                    }
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
            if let Value::Enum {
                discriminator,
                fields,
            } = value
            {
                match *discriminator {
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
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::PackageAddress(_))
            } else {
                false
            }
        }
        Type::ComponentAddress => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::ComponentAddress(_))
            } else {
                false
            }
        }
        Type::ResourceAddress => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::ResourceAddress(_))
            } else {
                false
            }
        }

        Type::Own => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(_))
            } else {
                false
            }
        }
        Type::Bucket => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(Own::Bucket(_)))
            } else {
                false
            }
        }
        Type::Proof => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(Own::Proof(_)))
            } else {
                false
            }
        }
        Type::Vault => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(Own::Vault(_)))
            } else {
                false
            }
        }
        Type::Component => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(Own::Component(_)))
            } else {
                false
            }
        }
        Type::KeyValueStore { .. } => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Own(Own::KeyValueStore(_)))
            } else {
                false
            }
        }
        Type::Hash => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Hash(_))
            } else {
                false
            }
        }
        Type::EcdsaSecp256k1PublicKey => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::EcdsaSecp256k1PublicKey(_))
            } else {
                false
            }
        }
        Type::EcdsaSecp256k1Signature => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::EcdsaSecp256k1Signature(_))
            } else {
                false
            }
        }
        Type::EddsaEd25519PublicKey => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::EddsaEd25519PublicKey(_))
            } else {
                false
            }
        }
        Type::EddsaEd25519Signature => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::EddsaEd25519Signature(_))
            } else {
                false
            }
        }
        Type::Decimal => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::Decimal(_))
            } else {
                false
            }
        }
        Type::PreciseDecimal => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::PreciseDecimal(_))
            } else {
                false
            }
        }
        Type::NonFungibleLocalId => {
            if let Value::Custom { value } = value {
                matches!(value, ScryptoCustomValue::NonFungibleLocalId(_))
            } else {
                false
            }
        }

        Type::Any => true,
    }
}
