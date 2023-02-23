use crate::traversal::*;
use crate::typed_traversal::*;
use crate::*;

pub enum ValidationError<X: CustomValueKind> {
    TraversalError(TypedTraversalError<X>),
    TypeValidationError(TypeValidationError),
    SchemaInconsistency(SchemaInconsistencyError),
}

pub enum SchemaInconsistencyError {
    TypeValidationNotFound(LocalTypeIndex),
    TypeValidationMismatch,
}

pub enum TypeValidationError {
    LengthValidationError {
        required: LengthValidation,
        actual: u32,
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

pub type FullValidationError<X> = LocatedError<ValidationError<X>>;

pub fn validate<E: CustomTypeExtension>(
    payload: &[u8],
    schema: &Schema<E>,
    index: LocalTypeIndex,
) -> Result<(), FullValidationError<E::CustomValueKind>> {
    let mut traverser = traverse_payload_with_types::<E>(payload, &schema.type_kinds, index);
    // NB - we use loop rather than a while loop partly for borrow-checker reasons
    loop {
        let event = traverser.next_event();
        if matches!(event, TypedTraversalEvent::End(_)) {
            return Ok(());
        }
        validate_event::<E>(&schema.type_validations, event)?;
    }
}

fn validate_event<E: CustomTypeExtension>(
    type_validations: &[SchemaTypeValidation<E>],
    event: TypedTraversalEvent<E::CustomTraversal>,
) -> Result<(), FullValidationError<E::CustomValueKind>> {
    match event {
        TypedTraversalEvent::PayloadPrefix(_) => Ok(()),
        TypedTraversalEvent::ContainerStart(event) => {
            validate_container::<E>(type_validations, event)
        }
        TypedTraversalEvent::ContainerEnd(_) => Ok(()), // Validation already handled at Container Start
        TypedTraversalEvent::TerminalValue(event) => {
            validate_terminal_value::<E>(type_validations, event)
        }
        TypedTraversalEvent::TerminalValueBatch(event) => {
            validate_terminal_value_batch::<E>(type_validations, event)
        }
        TypedTraversalEvent::End(_) => {
            unreachable!("End should already have been covered in the parent function")
        }
        TypedTraversalEvent::Error(located_error) => Err(FullValidationError {
            error: ValidationError::TraversalError(located_error.error),
            location: located_error.location,
        }),
    }
}

#[macro_export]
macro_rules! return_type_validation_mismatch {
    ($location: expr) => {
        return Err(FullValidationError {
            error: ValidationError::SchemaInconsistency(
                SchemaInconsistencyError::TypeValidationMismatch,
            ),
            location: $location,
        })
    };
}

#[macro_export]
macro_rules! return_type_validation_error {
    ($location: expr, $error: expr) => {
        return Err(FullValidationError {
            error: ValidationError::TypeValidationError($error),
            location: $location,
        })
    };
}

pub fn validate_container<E: CustomTypeExtension>(
    type_validations: &[SchemaTypeValidation<E>],
    event: TypedLocatedDecoding<ContainerHeader<E::CustomTraversal>, E::CustomTraversal>,
) -> Result<(), FullValidationError<E::CustomValueKind>> {
    let TypedLocatedDecoding {
        inner: header,
        type_index,
        location,
        ..
    } = event;
    let Some(validation) = resolve_type_validation::<E>(&type_validations, type_index) else {
        return Err(FullValidationError {
            error: ValidationError::SchemaInconsistency(SchemaInconsistencyError::TypeValidationNotFound(type_index)),
            location,
        })
    };
    match validation {
        TypeValidation::None => {}
        TypeValidation::Array { length_validation } => {
            let ContainerHeader::Array(ArrayHeader { length, .. }) = header else {
                return_type_validation_mismatch!(location)
            };
            if !length_validation.is_valid(length) {
                return_type_validation_error!(
                    location,
                    TypeValidationError::LengthValidationError {
                        required: *length_validation,
                        actual: length,
                    }
                );
            }
        }
        TypeValidation::Map { length_validation } => {
            let ContainerHeader::Map(MapHeader { length, .. }) = header else {
                return_type_validation_mismatch!(location)
            };
            if !length_validation.is_valid(length) {
                return_type_validation_error!(
                    location,
                    TypeValidationError::LengthValidationError {
                        required: *length_validation,
                        actual: length,
                    }
                );
            }
        }
        TypeValidation::Custom(_) => {
            // TODO - add this in when we have custom validations
            unreachable!("Unreachable at present")
        }
        _ => return_type_validation_mismatch!(location),
    }
    Ok(())
}

#[macro_export]
macro_rules! numeric_validation_match {
    ($numeric_validation: ident, $value: expr, $location: expr, $type: ident, $error_type: ident) => {{
        {
            let TerminalValueRef::$type(value) = $value else { return_type_validation_mismatch!($location) };
            if !$numeric_validation.is_valid(value) {
                return_type_validation_error!(
                    $location,
                    TypeValidationError::$error_type {
                        required: *$numeric_validation,
                        actual: value,
                    }
                );
            }
        }
    }};
}

pub fn validate_terminal_value<'de, E: CustomTypeExtension>(
    type_validations: &[SchemaTypeValidation<E>],
    event: TypedLocatedDecoding<TerminalValueRef<'de, E::CustomTraversal>, E::CustomTraversal>,
) -> Result<(), FullValidationError<E::CustomValueKind>> {
    let TypedLocatedDecoding {
        inner: value,
        type_index,
        location,
        ..
    } = event;
    let Some(validation) = resolve_type_validation::<E>(&type_validations, type_index) else {
        return Err(FullValidationError {
            error: ValidationError::SchemaInconsistency(SchemaInconsistencyError::TypeValidationNotFound(type_index)),
            location,
        })
    };
    match validation {
        TypeValidation::None => {}
        TypeValidation::I8(x) => {
            numeric_validation_match!(x, value, location, I8, I8ValidationError)
        }
        TypeValidation::I16(x) => {
            numeric_validation_match!(x, value, location, I16, I16ValidationError)
        }
        TypeValidation::I32(x) => {
            numeric_validation_match!(x, value, location, I32, I32ValidationError)
        }
        TypeValidation::I64(x) => {
            numeric_validation_match!(x, value, location, I64, I64ValidationError)
        }
        TypeValidation::I128(x) => {
            numeric_validation_match!(x, value, location, I128, I128ValidationError)
        }
        TypeValidation::U8(x) => {
            numeric_validation_match!(x, value, location, U8, U8ValidationError)
        }
        TypeValidation::U16(x) => {
            numeric_validation_match!(x, value, location, U16, U16ValidationError)
        }
        TypeValidation::U32(x) => {
            numeric_validation_match!(x, value, location, U32, U32ValidationError)
        }
        TypeValidation::U64(x) => {
            numeric_validation_match!(x, value, location, U64, U64ValidationError)
        }
        TypeValidation::U128(x) => {
            numeric_validation_match!(x, value, location, U128, U128ValidationError)
        }
        TypeValidation::Custom(_) => {
            // TODO - add this in when we have custom validations
            unreachable!("Unreachable at present")
        }
        _ => return_type_validation_mismatch!(location),
    }
    Ok(())
}

pub fn validate_terminal_value_batch<'de, E: CustomTypeExtension>(
    type_validations: &[SchemaTypeValidation<E>],
    event: TypedLocatedDecoding<TerminalValueBatchRef<'de, E::CustomTraversal>, E::CustomTraversal>,
) -> Result<(), FullValidationError<E::CustomValueKind>> {
    let TypedLocatedDecoding {
        inner: value_batch,
        type_index,
        location,
        ..
    } = event;
    let Some(validation) = resolve_type_validation::<E>(&type_validations, type_index) else {
        return Err(FullValidationError {
            error: ValidationError::SchemaInconsistency(SchemaInconsistencyError::TypeValidationNotFound(type_index)),
            location,
        })
    };
    match validation {
        TypeValidation::None => {}
        TypeValidation::U8(numeric_validation) => {
            let TerminalValueBatchRef::U8(value_batch) = value_batch else {
                return_type_validation_mismatch!(location)
            };
            for byte in value_batch.iter() {
                if !numeric_validation.is_valid(*byte) {
                    return_type_validation_error!(
                        location,
                        TypeValidationError::U8ValidationError {
                            required: *numeric_validation,
                            actual: *byte,
                        }
                    );
                }
            }
        }
        TypeValidation::Custom(_) => {
            // TODO - add this in when we have custom validations
            unreachable!("Unreachable at present")
        }
        _ => return_type_validation_mismatch!(location),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{traversal::LocatedError, *};

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
            Err(LocatedError {
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
}
