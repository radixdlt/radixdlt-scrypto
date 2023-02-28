use crate::rust::prelude::*;
use crate::traversal::*;
use crate::typed_traversal::*;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError<E: CustomTypeExtension> {
    TraversalError(TypedTraversalError<E::CustomValueKind>),
    TypeValidationError(TypeValidationError),
    SchemaInconsistency(SchemaInconsistencyError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaInconsistencyError {
    TypeValidationNotFound(LocalTypeIndex),
    TypeValidationMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeValidationError {
    LengthValidationError {
        required: LengthValidation,
        actual: usize,
    },
    I8ValidationError {
        required: NumericValidation<i8>,
        actual: i8,
    },
    I16ValidationError {
        required: NumericValidation<i16>,
        actual: i16,
    },
    I32ValidationError {
        required: NumericValidation<i32>,
        actual: i32,
    },
    I64ValidationError {
        required: NumericValidation<i64>,
        actual: i64,
    },
    I128ValidationError {
        required: NumericValidation<i128>,
        actual: i128,
    },
    U8ValidationError {
        required: NumericValidation<u8>,
        actual: u8,
    },
    U16ValidationError {
        required: NumericValidation<u16>,
        actual: u16,
    },
    U32ValidationError {
        required: NumericValidation<u32>,
        actual: u32,
    },
    U64ValidationError {
        required: NumericValidation<u64>,
        actual: u64,
    },
    U128ValidationError {
        required: NumericValidation<u128>,
        actual: u128,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedValidationError<E: CustomTypeExtension> {
    error: ValidationError<E>,
    location: ErrorLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorLocation {
    start_offset: usize,
    end_offset: usize,
}

pub fn validate<E: CustomTypeExtension>(
    payload: &[u8],
    schema: &Schema<E>,
    index: LocalTypeIndex,
) -> Result<(), LocatedValidationError<E>> {
    let mut traverser = traverse_payload_with_types::<E>(payload, &schema.type_kinds, index);
    // NB - we use loop rather than a while loop partly for borrow-checker reasons
    loop {
        let TypedLocatedTraversalEvent {
            event,
            location: typed_location,
        } = traverser.next_event();
        if matches!(event, TypedTraversalEvent::End) {
            return Ok(());
        }
        validate_event_with_type::<E>(&schema.type_validations, event).map_err(|error| {
            LocatedValidationError {
                error,
                location: ErrorLocation {
                    start_offset: typed_location.location.start_offset,
                    end_offset: typed_location.location.end_offset,
                    // TODO - add context from (location + type_index) + type metadata
                    // This enables a full path to be provided in the error message, which can have a debug such as:
                    // TypeOne["hello"]->Enum::Variant[4]->TypeTwo[0]->Array[4]->Map[Key]->StructWhichErrored
                },
            }
        })?;
    }
}

fn validate_event_with_type<E: CustomTypeExtension>(
    type_validations: &[SchemaTypeValidation<E>],
    event: TypedTraversalEvent<E::CustomTraversal>,
) -> Result<(), ValidationError<E>> {
    match event {
        TypedTraversalEvent::PayloadPrefix => Ok(()),
        TypedTraversalEvent::ContainerStart(type_index, header) => {
            validate_container::<E>(type_validations, header, type_index)
        }
        TypedTraversalEvent::ContainerEnd(_, _) => Ok(()), // Validation already handled at Container Start
        TypedTraversalEvent::TerminalValue(type_index, value_ref) => {
            validate_terminal_value::<E>(type_validations, value_ref, type_index)
        }
        TypedTraversalEvent::TerminalValueBatch(type_index, value_batch_ref) => {
            validate_terminal_value_batch::<E>(type_validations, value_batch_ref, type_index)
        }
        TypedTraversalEvent::End => {
            unreachable!("End should already have been covered in the parent function")
        }
        TypedTraversalEvent::Error(error) => Err(ValidationError::TraversalError(error)),
    }
}

#[macro_export]
macro_rules! return_type_validation_mismatch {
    () => {
        return Err(ValidationError::SchemaInconsistency(
            SchemaInconsistencyError::TypeValidationMismatch,
        ))
    };
}

#[macro_export]
macro_rules! return_type_validation_error {
    ($error: expr) => {
        return Err(ValidationError::TypeValidationError($error));
    };
}

pub fn validate_container<E: CustomTypeExtension>(
    type_validations: &[SchemaTypeValidation<E>],
    header: ContainerHeader<E::CustomTraversal>,
    type_index: LocalTypeIndex,
) -> Result<(), ValidationError<E>> {
    let Some(validation) = resolve_type_validation::<E>(&type_validations, type_index) else {
        return Err(ValidationError::SchemaInconsistency(SchemaInconsistencyError::TypeValidationNotFound(type_index)))
    };
    match validation {
        TypeValidation::None => {}
        TypeValidation::Array { length_validation } => {
            let ContainerHeader::Array(ArrayHeader { length, .. }) = header else {
                return_type_validation_mismatch!()
            };
            if !length_validation.is_valid(length) {
                return_type_validation_error!(TypeValidationError::LengthValidationError {
                    required: *length_validation,
                    actual: length,
                });
            }
        }
        TypeValidation::Map { length_validation } => {
            let ContainerHeader::Map(MapHeader { length, .. }) = header else {
                return_type_validation_mismatch!()
            };
            if !length_validation.is_valid(length) {
                return_type_validation_error!(TypeValidationError::LengthValidationError {
                    required: *length_validation,
                    actual: length,
                });
            }
        }
        TypeValidation::Custom(_) => {
            // TODO - add this in when we have custom validations
            unreachable!("Unreachable at present")
        }
        _ => return_type_validation_mismatch!(),
    }
    Ok(())
}

#[macro_export]
macro_rules! numeric_validation_match {
    ($numeric_validation: ident, $value: expr, $type: ident, $error_type: ident) => {{
        {
            let TerminalValueRef::$type(value) = $value else { return_type_validation_mismatch!() };
            if !$numeric_validation.is_valid(value) {
                return_type_validation_error!(TypeValidationError::$error_type {
                    required: *$numeric_validation,
                    actual: value,
                });
            }
        }
    }};
}

pub fn validate_terminal_value<'de, E: CustomTypeExtension>(
    type_validations: &[SchemaTypeValidation<E>],
    value: TerminalValueRef<'de, E::CustomTraversal>,
    type_index: LocalTypeIndex,
) -> Result<(), ValidationError<E>> {
    let Some(validation) = resolve_type_validation::<E>(&type_validations, type_index) else {
        return Err(ValidationError::SchemaInconsistency(SchemaInconsistencyError::TypeValidationNotFound(type_index)));
    };
    match validation {
        TypeValidation::None => {}
        TypeValidation::I8(x) => {
            numeric_validation_match!(x, value, I8, I8ValidationError)
        }
        TypeValidation::I16(x) => {
            numeric_validation_match!(x, value, I16, I16ValidationError)
        }
        TypeValidation::I32(x) => {
            numeric_validation_match!(x, value, I32, I32ValidationError)
        }
        TypeValidation::I64(x) => {
            numeric_validation_match!(x, value, I64, I64ValidationError)
        }
        TypeValidation::I128(x) => {
            numeric_validation_match!(x, value, I128, I128ValidationError)
        }
        TypeValidation::U8(x) => {
            numeric_validation_match!(x, value, U8, U8ValidationError)
        }
        TypeValidation::U16(x) => {
            numeric_validation_match!(x, value, U16, U16ValidationError)
        }
        TypeValidation::U32(x) => {
            numeric_validation_match!(x, value, U32, U32ValidationError)
        }
        TypeValidation::U64(x) => {
            numeric_validation_match!(x, value, U64, U64ValidationError)
        }
        TypeValidation::U128(x) => {
            numeric_validation_match!(x, value, U128, U128ValidationError)
        }
        TypeValidation::Custom(_) => {
            // TODO - add this in when we have custom validations
            unreachable!("Unreachable at present")
        }
        _ => return_type_validation_mismatch!(),
    }
    Ok(())
}

pub fn validate_terminal_value_batch<'de, E: CustomTypeExtension>(
    type_validations: &[SchemaTypeValidation<E>],
    value_batch: TerminalValueBatchRef<'de>,
    type_index: LocalTypeIndex,
) -> Result<(), ValidationError<E>> {
    let Some(validation) = resolve_type_validation::<E>(&type_validations, type_index) else {
        return Err(ValidationError::SchemaInconsistency(SchemaInconsistencyError::TypeValidationNotFound(type_index)));
    };
    match validation {
        TypeValidation::None => {}
        TypeValidation::U8(numeric_validation) => {
            let TerminalValueBatchRef::U8(value_batch) = value_batch;
            for byte in value_batch.iter() {
                if !numeric_validation.is_valid(*byte) {
                    return_type_validation_error!(TypeValidationError::U8ValidationError {
                        required: *numeric_validation,
                        actual: *byte,
                    });
                }
            }
        }
        TypeValidation::Custom(_) => {
            // TODO - add this in when we have custom validations
            unreachable!("Unreachable at present")
        }
        _ => return_type_validation_mismatch!(),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::LocatedValidationError;
    use crate::{rust::prelude::*, *};

    #[derive(Sbor)]
    struct TestStructArray {
        x: [u8; 16],
    }

    #[derive(Sbor)]
    struct TestStructVec {
        x: Vec<u8>,
    }

    #[test]
    pub fn identical_length_vec_and_array_are_interchangeable() {
        let (type_index, schema) =
            generate_full_schema_from_single_type::<TestStructArray, NoCustomTypeExtension>();
        let payload = basic_encode(&TestStructVec {
            x: Vec::from([0; 16]),
        })
        .unwrap();

        let result = validate(&payload, &schema, type_index);
        assert!(result.is_ok())
    }

    #[test]
    pub fn longer_length_vec_is_not_interchangeable_with_array() {
        let (type_index, schema) =
            generate_full_schema_from_single_type::<TestStructArray, NoCustomTypeExtension>();
        let payload = basic_encode(&TestStructVec {
            x: Vec::from([0; 17]),
        })
        .unwrap();

        let result = validate(&payload, &schema, type_index);
        assert!(matches!(
            result,
            Err(LocatedValidationError {
                error: ValidationError::TypeValidationError(
                    TypeValidationError::LengthValidationError {
                        required: LengthValidation {
                            min: Some(16),
                            max: Some(16)
                        },
                        actual: 17,
                    }
                ),
                ..
            })
        ))
    }

    #[derive(Debug, Clone, Sbor)]
    pub enum SimpleEnum {
        Unit,
        Unnamed(String),
        Named { x: u8, y: u32 },
    }

    #[derive(Debug, Clone, Sbor)]
    pub struct SimpleStruct {
        pub unit: (),
        pub boolean: bool,
        pub u8: u8,
        pub u16: u16,
        pub u32: u32,
        pub u64: u64,
        pub u128: u128,
        pub i8: i8,
        pub i16: i16,
        pub i32: i32,
        pub i64: i64,
        pub i128: i128,
        pub string: String,
        pub enumeration: (SimpleEnum, SimpleEnum, SimpleEnum),
        pub recursive_struct: Option<Box<SimpleStruct>>,
        pub vector: Vec<u16>,
        pub set: HashSet<String>,
        pub map: BTreeMap<String, String>,
    }

    #[test]
    pub fn test_schema() {
        let mut x = SimpleStruct {
            unit: (),
            boolean: true,
            u8: 1,
            u16: 2,
            u32: 3,
            u64: 4,
            u128: 5,
            i8: 6,
            i16: 7,
            i32: 8,
            i64: 9,
            i128: 10,
            string: "String".to_owned(),
            enumeration: (
                SimpleEnum::Unit,
                SimpleEnum::Named { x: 1, y: 2 },
                SimpleEnum::Unnamed("a".to_string()),
            ),
            recursive_struct: None,
            vector: vec![1, 2],
            set: hashset!("a".to_string(), "b".to_string()),
            map: btreemap!("c".to_string() => "d".to_string()),
        };
        x.recursive_struct = Some(Box::new(x.clone()));

        let bytes = basic_encode(&x).unwrap();
        let (type_index, schema) =
            generate_full_schema_from_single_type::<SimpleStruct, NoCustomTypeExtension>();
        let result = validate(&bytes, &schema, type_index);
        assert!(result.is_ok())
    }
}
